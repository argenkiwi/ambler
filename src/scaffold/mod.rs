//! Pure text generation and analysis: source in, source out, no I/O.
//!
//! Nodes decide what to do; the functions here work out what the files should say. What does not
//! depend on the language is here; each [`Target`](target::Target) has its own module.

pub mod args;
pub mod context;
pub mod lint;
pub mod names;
pub mod pave;
pub mod rust;
pub mod target;
pub mod ts;
pub mod walk;

use target::Target;

pub use lint::{Violation, lint};

/// One thing to do to the file system. Paths are relative to the current directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// Write `contents` to `path`, creating its folders. `format` names the target whose
    /// formatter runs on it afterwards.
    Write {
        path: String,
        contents: String,
        format: Option<Target>,
    },
    /// Make `path` a symbolic link to `target`.
    Symlink { path: String, target: String },
}

impl Change {
    /// A file written as is.
    #[must_use]
    pub fn write(path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self::Write {
            path: path.into(),
            contents: contents.into(),
            format: None,
        }
    }

    /// A generated source file, run through `target`'s formatter once written.
    #[must_use]
    pub fn source(target: Target, path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self::Write {
            path: path.into(),
            contents: contents.into(),
            format: Some(target),
        }
    }

    /// A generated Rust file, to be run through rustfmt once written.
    #[must_use]
    pub fn rust(path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self::source(Target::Rust, path, contents)
    }

    /// The path this change touches.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::Write { path, .. } | Self::Symlink { path, .. } => path,
        }
    }
}

/// What a verb is about to do, and what to tell the caller once it is done.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub changes: Vec<Change>,
    /// Next steps, printed after the changes are applied.
    pub hint: String,
}
