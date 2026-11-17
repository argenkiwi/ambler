//! Test doubles shared by the node and walk tests: one fake for every node's `Utils`.

use crate::nodes::{
    guide, inspect, plan_node, plan_project, plan_util, plan_walk, report, verify, write,
};
use std::collections::BTreeMap;
use std::io;
use std::sync::Mutex;

pub(crate) use crate::walks::State;

/// An in-memory file system, terminal and cargo.
#[derive(Default)]
pub(crate) struct Fake {
    pub files: Mutex<BTreeMap<String, String>>,
    pub printed: Mutex<Vec<String>>,
    pub eprinted: Mutex<Vec<String>>,
    /// The paths handed to the formatter.
    pub formatted: Mutex<Vec<String>>,
    /// A path that cannot be written.
    pub unwritable: Option<&'static str>,
    /// Whether reading the project fails.
    pub unreadable: bool,
    /// The cargo subcommand that fails, or `"missing"` if cargo cannot be run at all.
    pub failing: Option<&'static str>,
    /// The name of the current folder, if it has one.
    pub folder: Option<&'static str>,
}

impl Fake {
    pub fn with_files(files: &[(&str, &str)]) -> Self {
        let files = files
            .iter()
            .map(|(path, contents)| ((*path).to_string(), (*contents).to_string()))
            .collect();
        Self {
            files: Mutex::new(files),
            ..Self::default()
        }
    }

    /// The contents of the file at `path`; a link reads as `-> <target>`.
    pub fn file(&self, path: &str) -> Option<String> {
        self.files.lock().unwrap().get(path).cloned()
    }

    pub fn paths(&self) -> Vec<String> {
        self.files.lock().unwrap().keys().cloned().collect()
    }

    pub fn printed(&self) -> Vec<String> {
        self.printed.lock().unwrap().clone()
    }

    pub fn eprinted(&self) -> Vec<String> {
        self.eprinted.lock().unwrap().clone()
    }

    /// Whether a file is at `path`, or `path` is a folder with files in it.
    fn exists(&self, path: &str) -> bool {
        let folder = format!("{path}/");
        self.files
            .lock()
            .unwrap()
            .keys()
            .any(|file| file == path || file.starts_with(&folder))
    }

    fn read(&self, path: &str) -> io::Result<String> {
        self.file(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.to_string()))
    }

    fn write(&self, path: &str, contents: &str) -> io::Result<()> {
        if self.unwritable == Some(path) {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), contents.to_string());
        Ok(())
    }

    fn print(&self, message: &str) {
        self.printed.lock().unwrap().push(message.to_string());
    }
}

impl plan_project::Utils for &Fake {
    fn exists(&self, path: &str) -> bool {
        Fake::exists(self, path)
    }

    fn current_folder(&self) -> Option<String> {
        self.folder.map(str::to_string)
    }

    fn entries(&self, path: &str) -> io::Result<Vec<String>> {
        let prefix = if path == "." {
            String::new()
        } else {
            format!("{path}/")
        };
        let mut entries: Vec<String> = self
            .paths()
            .iter()
            .filter_map(|file| file.strip_prefix(&prefix))
            .map(|rest| rest.split('/').next().unwrap_or(rest).to_string())
            .collect();
        entries.dedup();
        Ok(entries)
    }
}

impl plan_node::Utils for &Fake {
    fn exists(&self, path: &str) -> bool {
        Fake::exists(self, path)
    }
    fn read(&self, path: &str) -> io::Result<String> {
        Fake::read(self, path)
    }
}

impl plan_util::Utils for &Fake {
    fn exists(&self, path: &str) -> bool {
        Fake::exists(self, path)
    }
    fn read(&self, path: &str) -> io::Result<String> {
        Fake::read(self, path)
    }
}

impl plan_walk::Utils for &Fake {
    fn exists(&self, path: &str) -> bool {
        Fake::exists(self, path)
    }
    fn read(&self, path: &str) -> io::Result<String> {
        Fake::read(self, path)
    }
}

impl write::Utils for &Fake {
    fn write(&self, path: &str, contents: &str) -> io::Result<()> {
        Fake::write(self, path, contents)
    }
    fn symlink(&self, path: &str, target: &str) -> io::Result<()> {
        Fake::write(self, path, &format!("-> {target}"))
    }
    fn format(&self, paths: &[String]) {
        self.formatted.lock().unwrap().extend_from_slice(paths);
    }
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
}

impl report::Utils for &Fake {
    fn eprint(&self, message: &str) {
        self.eprinted.lock().unwrap().push(message.to_string());
    }
}

impl inspect::Utils for &Fake {
    fn read_project(&self) -> io::Result<BTreeMap<String, String>> {
        if self.unreadable {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(self.files.lock().unwrap().clone())
    }
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
}

impl verify::Utils for &Fake {
    fn cargo(&self, args: &[&str]) -> io::Result<bool> {
        match self.failing {
            Some("missing") => Err(io::ErrorKind::NotFound.into()),
            failing => Ok(failing != args.first().copied()),
        }
    }
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
}

impl guide::Utils for &Fake {
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
}
