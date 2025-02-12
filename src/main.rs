use anyhow::Result;
use serde::Deserialize;
use std::fs::{self, File};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use thiserror::Error;

mod zipper;

#[derive(Debug, Deserialize)]
struct Metadata {
    name: String,
    version: String,
    author: String,
    executable: String,
}

#[derive(Error, Debug)]
pub enum MatsError {
    #[error("Metadata file not found in archive")]
    MetadataNotFound,
    #[error("Failed to extract files")]
    ExtractionError,
    #[error("Failed to execute application")]
    ExecutionError,
    #[error("Invalid metadata format")]
    InvalidMetadata,
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Executable not found in archive: {0}")]
    ExecutableNotFound(String),
}

struct MatsLoader {
    temp_dir: TempDir,
}

fn is_elf_file(path: &Path) -> Result<bool> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    use std::io::Read;
    if file.read_exact(&mut magic).is_ok() {
        Ok(magic == [0x7f, 0x45, 0x4c, 0x46])
    } else {
        Ok(false)
    }
}

impl MatsLoader {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        println!("Created temporary directory at: {}", temp_dir.path().display());
        Ok(Self { temp_dir })
    }

    fn load_archive(&self, archive_path: &Path) -> Result<Metadata> {
        if !archive_path.exists() {
            return Err(MatsError::FileNotFound(
                archive_path.to_string_lossy().to_string()
            ).into());
        }

        println!("Opening archive: {}", archive_path.display());
        let file = File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let metadata = self.extract_metadata(&mut archive)?;
        println!("Found metadata for executable: {}", metadata.executable);

        if archive.by_name(&metadata.executable).is_err() {
            return Err(anyhow::anyhow!(
                "Executable '{}' not found in archive", 
                metadata.executable
            ));
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = self.temp_dir.path().join(file.name());
            println!("Extracting: {} to {}", file.name(), outpath.display());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;

                #[cfg(unix)]
                {
                    if file.name() == metadata.executable || is_elf_file(&outpath)? {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = fs::metadata(&outpath)?.permissions();
                        perms.set_mode(0o755);
                        fs::set_permissions(&outpath, perms)?;
                        println!("Set executable permissions for: {}", outpath.display());
                    }
                }
            }
        }

        Ok(metadata)
    }

    fn extract_metadata(&self, archive: &mut zip::ZipArchive<File>) -> Result<Metadata> {
        let metadata_file = archive
            .by_name("metadata.txt")
            .map_err(|_| MatsError::MetadataNotFound)?;
        
        serde_yaml::from_reader(metadata_file)
            .map_err(|_| MatsError::InvalidMetadata.into())
    }

    fn execute(&self, metadata: &Metadata) -> Result<()> {
        let executable_path = self.temp_dir.path().join(&metadata.executable);
        println!("Executing: {}", executable_path.display());
        
        if !executable_path.exists() {
            return Err(MatsError::FileNotFound(
                executable_path.to_string_lossy().to_string()
            ).into());
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&executable_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&executable_path, perms)?;
        }

        let status = Command::new(&executable_path)
            .current_dir(self.temp_dir.path())
            .spawn()
            .map_err(|e| {
                println!("Exec failed with error: {}", e);
                MatsError::ExecutionError
            })?
            .wait()?;

        if !status.success() {
            return Err(MatsError::ExecutionError.into());
        }

        Ok(())
    }
}

fn print_help() {
    println!("MATS (Me At The School) Package Manager");
    println!("\nUSAGE:");
    println!("  mats <COMMAND> [OPTIONS]");
    println!("\nCOMMANDS:");
    println!("  pack    Create a MATS package");
    println!("  run     Run a MATS package");
    println!("  help    Show this help message");
    println!("\nPACK OPTIONS:");
    println!("  mats pack <output.mats> <source-dir>");
    println!("  mats pack <output.mats> <executable-file>");
    println!("\nRUN OPTIONS:");
    println!("  mats run <package.mats>");
    println!("\nEXAMPLES:");
    println!("  mats pack myapp.mats myapp/");
    println!("  mats pack firefox.mats firefox");
    println!("  mats run myapp.mats");
    println!("\nNOTE: source-dir must contain metadata.txt with format:");
    println!("  name: Application Name");
    println!("  version: 1.0.0");
    println!("  author: Author Name");
    println!("  executable: program");
}

fn ensure_mats_extension(path: &str) -> String {
    if !path.ends_with(".mats") {
        format!("{}.mats", path)
    } else {
        path.to_string()
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 || args[1] == "help" || args[1] == "--help" || args[1] == "-h" {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "run" => {
            if args.len() != 3 {
                println!("Usage: mats run <package.mats>");
                return Ok(());
            }
            let loader = MatsLoader::new()?;
            let metadata = loader.load_archive(Path::new(&args[2]))?;
            
            println!("Loading application: {} v{}", metadata.name, metadata.version);
            println!("Author: {}", metadata.author);
            
            loader.execute(&metadata)?;
        }
        "pack" => {
            if args.len() != 4 {
                println!("Usage: mats pack <output.mats> <source-path>");
                println!("\nNote: source-path must contain metadata.txt");
                return Ok(());
            }

            let output_path = ensure_mats_extension(&args[2]);
            let source_path = Path::new(&args[3]);
            if !source_path.exists() {
                println!("Error: Source path does not exist: {}", args[3]);
                return Ok(());
            }

            let zipper = zipper::MatsZipper::new(&output_path, &args[3])?;
            zipper.create_archive()?;
        }
        _ => {
            println!("Unknown command: {}", args[1]);
            println!("Use 'mats help' for usage information");
        }
    }

    Ok(())
} 