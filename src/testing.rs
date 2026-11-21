//! Test doubles shared by the node and walk tests: one fake for every node's `Utils`.

use crate::nodes::{
    brief_agent, check_project, checkpoint, choose_task, guide, inspect, plan_node, plan_pave,
    plan_project, plan_util, plan_walk, report, roll_back, verify, write,
};
use std::collections::{BTreeMap, VecDeque};
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
    /// An argument of the check command that fails (`clippy`, `test`, `lint`), or `"missing"` if
    /// the checks cannot be run at all. For pave:
    /// `"check"`, `"git"` or `"agent"` cannot be run, `"agent-exit"` runs and exits 1.
    pub failing: Option<&'static str>,
    /// The name of the current folder, if it has one.
    pub folder: Option<&'static str>,
    /// The environment variables that are set.
    pub env: Vec<(&'static str, &'static str)>,
    /// The git folder, if the current directory is in a repository.
    pub git_dir: Option<&'static str>,
    /// What each `amble check` run says, in order: whether it passes, and its output. Once
    /// they run out, the check passes with no output.
    pub checks: Mutex<VecDeque<(bool, &'static str)>>,
    /// Whether the working tree has uncommitted changes.
    pub dirty: Mutex<bool>,
    /// The git commands run: `commit <subject>`, `stash <message>`.
    pub git: Mutex<Vec<String>>,
    /// The files each agent run writes, in order.
    pub agent_writes: Mutex<VecDeque<Vec<(&'static str, String)>>>,
    /// The commands and prompts the agent was run with.
    pub prompts: Mutex<Vec<(String, String)>>,
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

    fn eprint(&self, message: &str) {
        self.eprinted.lock().unwrap().push(message.to_string());
    }

    pub fn git(&self) -> Vec<String> {
        self.git.lock().unwrap().clone()
    }

    pub fn prompts(&self) -> Vec<(String, String)> {
        self.prompts.lock().unwrap().clone()
    }

    /// Fails like a git command that could not be run, if git is the failing program.
    fn git_runs(&self) -> io::Result<()> {
        match self.failing {
            Some("git") => Err(io::ErrorKind::NotFound.into()),
            _ => Ok(()),
        }
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
    fn entries(&self, path: &str) -> io::Result<Vec<String>> {
        plan_project::Utils::entries(self, path)
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
    fn format(&self, _target: crate::scaffold::target::Target, paths: &[String]) {
        self.formatted.lock().unwrap().extend_from_slice(paths);
    }
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
}

impl report::Utils for &Fake {
    fn eprint(&self, message: &str) {
        Fake::eprint(self, message);
    }
}

impl plan_pave::Utils for &Fake {
    fn exists(&self, path: &str) -> bool {
        Fake::exists(self, path)
    }
    fn env(&self, name: &str) -> Option<String> {
        self.env
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| (*value).to_string())
    }
    fn git_dir(&self) -> Option<String> {
        self.git_dir.map(str::to_string)
    }
}

impl check_project::Utils for &Fake {
    fn run_check(&self, log: &str, _target: &str) -> io::Result<bool> {
        if self.failing == Some("check") {
            return Err(io::ErrorKind::NotFound.into());
        }
        let (passed, output) = self
            .checks
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or((true, ""));
        Fake::write(self, log, output)?;
        Ok(passed)
    }
    fn read(&self, path: &str) -> io::Result<String> {
        Fake::read(self, path)
    }
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
    fn eprint(&self, message: &str) {
        Fake::eprint(self, message);
    }
}

impl checkpoint::Utils for &Fake {
    fn dirty(&self) -> io::Result<bool> {
        self.git_runs()?;
        Ok(*self.dirty.lock().unwrap())
    }
    fn commit(&self, subject: &str, _body: &str) -> io::Result<()> {
        self.git_runs()?;
        *self.dirty.lock().unwrap() = false;
        self.git.lock().unwrap().push(format!("commit {subject}"));
        Ok(())
    }
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
}

impl choose_task::Utils for &Fake {
    fn read(&self, path: &str) -> io::Result<String> {
        Fake::read(self, path)
    }
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
}

impl roll_back::Utils for &Fake {
    fn stash(&self, message: &str) -> io::Result<()> {
        self.git_runs()?;
        *self.dirty.lock().unwrap() = false;
        self.git.lock().unwrap().push(format!("stash {message}"));
        Ok(())
    }
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
}

impl brief_agent::Utils for &Fake {
    fn read(&self, path: &str) -> io::Result<String> {
        Fake::read(self, path)
    }
    fn status(&self) -> io::Result<String> {
        self.git_runs()?;
        Ok(if *self.dirty.lock().unwrap() {
            " M src/lib.rs\n".to_string()
        } else {
            String::new()
        })
    }
    fn agent(&self, command: &str, prompt: &str) -> io::Result<bool> {
        if self.failing == Some("agent") {
            return Err(io::ErrorKind::NotFound.into());
        }
        self.prompts
            .lock()
            .unwrap()
            .push((command.to_string(), prompt.to_string()));
        let writes = self.agent_writes.lock().unwrap().pop_front();
        for (path, contents) in writes.unwrap_or_default() {
            Fake::write(self, path, &contents)?;
            *self.dirty.lock().unwrap() = true;
        }
        Ok(self.failing != Some("agent-exit"))
    }
    fn print(&self, message: &str) {
        Fake::print(self, message);
    }
}

impl inspect::Utils for &Fake {
    fn read_project(
        &self,
        _target: crate::scaffold::target::Target,
    ) -> io::Result<BTreeMap<String, String>> {
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
    fn run(&self, _program: &str, args: &[&str]) -> io::Result<bool> {
        match self.failing {
            Some("missing") => Err(io::ErrorKind::NotFound.into()),
            Some(failing) => Ok(!args.contains(&failing)),
            None => Ok(true),
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
