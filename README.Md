
# MATS (Me At The School) Package Manager

MATS is a simple and lightweight package manager designed for Linux environments. It allows you to easily package and run applications—whether they are single executables or directory-based apps.

## Features

- **Fast loading**: Minimal compression for quick access.
- **Flexibility**: Supports both single executables and directory-based applications.
- **Security**: Automatically generates SHA256 hashes for integrity verification.
- **Permissions handling**: Proper executable permissions for ELF binaries.
- **Simple metadata**: Metadata is stored in an easy-to-read format.

---

## Installation

To install and build MATS, follow these steps:

1. Clone the repository:
   ```bash
   git clone https://github.com/kleeedolinux/mats
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

---

## Command Guide

### Basic Commands

- **Show help**:
  ```bash
  mats help
  ```
  (Displays help and usage information.)

- **Create a package**:
  ```bash
  mats pack <output> <source>
  ```
  (Package your app. `<output>` is the package name, and `<source>` is the file or directory you're packaging.)

- **Run a package**:
  ```bash
  mats run <package>
  ```
  (Run a packaged app.)

### Creating Packages

#### 1. Single Executable
If your app is just a single executable file, create a `metadata.txt` in the same directory with the following content:
```yaml
name: MyApplication
version: 1.0.0
author: Your Name
executable: program
```
Then, run:
```bash
mats pack app.mats ./program
```

#### 2. Directory with Multiple Files
For apps with multiple files, create a directory structure like this:
```
myapp/
├── metadata.txt
├── program
├── lib/
└── config/
```
Then package it:
```bash
mats pack app.mats ./myapp
```

---

## Running Applications

Once you've packaged your app, you can run it like this:
```bash
mats run app.mats
```

---

## Examples

### 1. Package Firefox
Create `metadata.txt`:
```bash
echo "name: Firefox
version: 123.0
author: Mozilla
executable: firefox" > metadata.txt
```
Then package it:
```bash
mats pack firefox.mats ./firefox
```

### 2. Package an AppImage (e.g., VSCode)
Create `metadata.txt`:
```bash
echo "name: VSCode
version: 1.85.0
author: Microsoft
executable: code.AppImage" > metadata.txt
```
Then package it:
```bash
mats pack vscode.mats ./code.AppImage
```

### 3. Package a Directory-Based App
Create the directory:
```bash
mkdir myapp
cp program myapp/
cp -r libs myapp/
echo "name: MyApp
version: 1.0.0
author: Developer
executable: program" > myapp/metadata.txt
```
Then package it:
```bash
mats pack myapp.mats ./myapp
```

---

## Notes

- **File extension**: MATS will automatically add the `.mats` extension if not provided.
- **SHA256 checksums**: Automatically generated for security.
- **Executable permissions**: MATS handles proper permissions for ELF binaries.
- **metadata.txt**: Must be in YAML format and contain accurate details.

---

## Security Features

MATS provides several built-in security measures:

- **SHA256 hash verification**: Ensures the integrity of your executables.
- **Timestamps**: Each package includes a timestamp for tracking.
- **Executable permissions**: Automatically fixed for ELF binaries.
- **Isolated execution**: Applications run in a safe, isolated environment.

---

## File Formats

### `metadata.txt` (YAML)

This file contains essential details about your app:
```yaml
name: <app-name>
version: <version>
author: <author>
executable: <executable-name>
```

### `<name>.sha256`

This file contains the SHA256 checksum for verification:
```yaml
sha256: <hash>
date: <timestamp>
name: <app-name>
exec-file: <executable-name>
author: <author>
```

---

## Common Issues

### 1. **"Executable not found"**
- Ensure the executable name in `metadata.txt` matches the actual file.
- Check that the executable is in the correct location.

### 2. **"metadata.txt not found"**
- Ensure `metadata.txt` is in the correct location:
  - For single executables: it should be next to the executable.
  - For directory-based apps: place `metadata.txt` at the root of the directory.

### 3. **"Permission denied"**
- Ensure the executable has proper permissions.
- Run the command with the appropriate user permissions if needed.

---

## Package Structure

A `.mats` package includes:
- **`metadata.txt`**: Contains package information.
- **Application files**: The app’s executables and any other required files.
- **SHA256 checksum file**: Ensures package integrity.
