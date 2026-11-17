//! File system access, relative to the current directory.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

/// Whether anything exists at `path`.
#[must_use]
pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Reads the file at `path`.
///
/// # Errors
///
/// Returns the I/O error if the file is missing or cannot be read as UTF-8.
pub fn read(path: &str) -> io::Result<String> {
    std::fs::read_to_string(path)
}

/// Writes `contents` to `path`, creating the folders above it.
///
/// # Errors
///
/// Returns the I/O error if a folder or the file cannot be written.
pub fn write(path: &str, contents: &str) -> io::Result<()> {
    create_parent(path)?;
    std::fs::write(path, contents)
}

/// Makes `path` a symbolic link to `target`, creating the folders above it.
///
/// # Errors
///
/// Returns the I/O error if the link cannot be created, or if the platform has no symbolic links.
pub fn symlink(path: &str, target: &str) -> io::Result<()> {
    create_parent(path)?;
    #[cfg(unix)]
    return std::os::unix::fs::symlink(target, path);
    #[cfg(windows)]
    return std::os::windows::fs::symlink_dir(target, path);
    #[cfg(not(any(unix, windows)))]
    return Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!("cannot link {path} to {target} on this platform"),
    ));
}

fn create_parent(path: &str) -> io::Result<()> {
    match Path::new(path).parent() {
        Some(parent) if !parent.as_os_str().is_empty() => std::fs::create_dir_all(parent),
        _ => Ok(()),
    }
}

/// The files `amble check` looks at: `Cargo.toml`, every `.rs` under `src/` and every `.md`
/// under `specs/`, keyed by `/`-separated path.
///
/// Folders that do not exist are skipped: it is the linter's job to say what is missing.
///
/// # Errors
///
/// Returns the I/O error if a folder or file that exists cannot be read.
pub fn read_project() -> io::Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    if exists("Cargo.toml") {
        files.insert("Cargo.toml".to_string(), read("Cargo.toml")?);
    }
    collect("src", "rs", &mut files)?;
    collect("specs", "md", &mut files)?;
    Ok(files)
}

fn collect(folder: &str, extension: &str, files: &mut BTreeMap<String, String>) -> io::Result<()> {
    if !Path::new(folder).is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = format!("{folder}/{}", entry.file_name().to_string_lossy());
        if entry.file_type()?.is_dir() {
            collect(&path, extension, files)?;
        } else if Path::new(&path).extension().is_some_and(|e| e == extension) {
            let contents = read(&path)?;
            files.insert(path, contents);
        }
    }
    Ok(())
}
