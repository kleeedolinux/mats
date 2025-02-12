use anyhow::Result;
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

#[derive(Debug, Deserialize)]
struct Metadata {
    name: String,
    version: String,
    author: String,
    executable: String,
}

pub struct MatsZipper {
    output_path: PathBuf,
    source_path: PathBuf,
}

impl MatsZipper {
    pub fn new<P: AsRef<Path>>(output_path: P, source_path: P) -> Result<Self> {
        Ok(Self {
            output_path: output_path.as_ref().to_path_buf(),
            source_path: source_path.as_ref().to_path_buf(),
        })
    }

    fn read_metadata(&self) -> Result<Metadata> {
        let metadata_path = if self.source_path.is_dir() {
            self.source_path.join("metadata.txt")
        } else {
            PathBuf::from("metadata.txt")
        };

        if !metadata_path.exists() {
            return Err(anyhow::anyhow!(
                "metadata.txt not found in {}",
                if self.source_path.is_dir() { "directory" } else { "current directory" }
            ));
        }

        let mut file = File::open(metadata_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_yaml::from_str(&contents)?)
    }

    fn generate_sha256_info(&self, metadata: &Metadata) -> Result<String> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        let exec_path = if self.source_path.is_dir() {
            self.source_path.join(&metadata.executable)
        } else {
            self.source_path.clone()
        };

        let mut file = File::open(&exec_path)?;
        std::io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();

        Ok(format!(
            "sha256: {}\ndate: {}\nname: {}\nexec-file: {}\nauthor: {}\n",
            hex::encode(hash),
            now,
            metadata.name,
            metadata.executable,
            metadata.author
        ))
    }

    pub fn create_archive(&self) -> Result<()> {
        let metadata = self.read_metadata()?;
        
        let sha256_info = self.generate_sha256_info(&metadata)?;
        let sha256_filename = format!("{}.sha256", metadata.name);
        std::fs::write(&sha256_filename, &sha256_info)?;

        let file = File::create(&self.output_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Stored);

        if self.source_path.is_dir() {
            self.add_directory_to_zip(&mut zip, options)?;
        } else {
            self.add_file_to_zip(&mut zip, options)?;
        }

        zip.start_file(&sha256_filename, options)?;
        zip.write_all(sha256_info.as_bytes())?;

        zip.finish()?;
        println!("Created MATS archive for '{}' v{}", metadata.name, metadata.version);
        println!("SHA256 info written to {}", sha256_filename);
        Ok(())
    }

    fn add_directory_to_zip(&self, zip: &mut ZipWriter<File>, options: FileOptions) -> Result<()> {
        for entry in WalkDir::new(&self.source_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path.strip_prefix(&self.source_path)?
                .to_string_lossy()
                .into_owned();

            if path.is_file() && !name.is_empty() {
                zip.start_file(name, options)?;
                let mut file = File::open(path)?;
                std::io::copy(&mut file, zip)?;
            } else if path.is_dir() && !name.is_empty() {
                zip.add_directory(name, options)?;
            }
        }
        Ok(())
    }

    fn add_file_to_zip(&self, zip: &mut ZipWriter<File>, options: FileOptions) -> Result<()> {
        zip.start_file("metadata.txt", options)?;
        let mut metadata_file = File::open("metadata.txt")?;
        std::io::copy(&mut metadata_file, zip)?;

        let file_name = self.source_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid source path"))?
            .to_string_lossy()
            .into_owned();

        zip.start_file(&file_name, options)?;
        let mut source_file = File::open(&self.source_path)?;
        std::io::copy(&mut source_file, zip)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_create_single_file_archive() -> Result<()> {
        let temp_dir = tempdir()?;
        let test_executable = temp_dir.path().join("test.elf");
        let test_content = b"\x7fELF..."; // Mock ELF header
        fs::write(&test_executable, test_content)?;

        let output_path = temp_dir.path().join("test.mats");
        let zipper = MatsZipper::new(
            &output_path,
            &test_executable,
        )?;

        zipper.create_archive()?;
        assert!(output_path.exists());
        Ok(())
    }

    #[test]
    fn test_create_directory_archive() -> Result<()> {
        let temp_dir = tempdir()?;
        let test_app_dir = temp_dir.path().join("testapp");
        fs::create_dir(&test_app_dir)?;

        // Create test files including an ELF executable
        let exec_name = "myapp.elf";
        fs::write(
            test_app_dir.join(exec_name),
            b"\x7fELF...", // Mock ELF header
        )?;
        fs::write(
            test_app_dir.join("config.txt"),
            b"some configuration",
        )?;

        let output_path = temp_dir.path().join("testapp.mats");
        let zipper = MatsZipper::new(
            &output_path,
            &test_app_dir,
        )?;

        zipper.create_archive()?;
        assert!(output_path.exists());
        Ok(())
    }
} 