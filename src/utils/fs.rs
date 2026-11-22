//! File system access, relative to the current directory.

use crate::scaffold::target::Target;
use std::collections::BTreeMap;
use std::io;
use std::path::Path;

/// Whether anything exists at `path`.
#[must_use]
pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// The name of the current folder, if it has one.
#[must_use]
pub fn current_folder_name() -> Option<String> {
    std::env::current_dir()
        .ok()?
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

/// The names of the entries in the folder at `path`.
///
/// # Errors
///
/// Returns the I/O error if the folder cannot be read.
pub fn entries(path: &str) -> io::Result<Vec<String>> {
    std::fs::read_dir(path)?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect()
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

/// The files `amble check` looks at, keyed by `/`-separated path: every `.md` under `specs/`
/// and, for Rust, `Cargo.toml` and every `.rs` under `src/`; for TypeScript, `package.json`,
/// `ambler.ts` and every `.ts` under `nodes/`, `walks/` and `utils/`.
///
/// Files and folders that do not exist are skipped: it is the linter's job to say what is
/// missing.
///
/// # Errors
///
/// Returns the I/O error if a folder or file that exists cannot be read.
pub fn read_project(target: Target) -> io::Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    let (roots, folders, extension): (&[&str], &[&str], &str) = match target {
        Target::Rust => (&["Cargo.toml"], &["src"], "rs"),
        Target::TypeScript => (
            &["package.json", "ambler.ts"],
            &["nodes", "walks", "utils"],
            "ts",
        ),
    };
    for root in roots {
        if exists(root) {
            files.insert((*root).to_string(), read(root)?);
        }
    }
    for folder in folders {
        collect(folder, extension, &mut files)?;
    }
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
