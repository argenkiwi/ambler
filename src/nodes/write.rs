use crate::Node;
use crate::nodes::HasPlan;
use crate::scaffold::Change;
use crate::scaffold::target::Target;
use crate::utils::{fs, process};
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not write {path}: {source}")]
    Io { path: String, source: io::Error },
}

pub trait Utils {
    fn write(&self, path: &str, contents: &str) -> io::Result<()>;
    fn symlink(&self, path: &str, target: &str) -> io::Result<()>;
    /// Formats `target`'s source files at `paths` in place, if it can.
    fn format(&self, target: Target, paths: &[String]);
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn write(&self, path: &str, contents: &str) -> io::Result<()> {
        fs::write(path, contents)
    }

    fn symlink(&self, path: &str, target: &str) -> io::Result<()> {
        fs::symlink(path, target)
    }

    fn format(&self, target: Target, paths: &[String]) {
        process::format(target, paths);
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Applies the plan: writes its files, formats the generated ones and prints what to do next.
/// Terminal: it has no outgoing edges.
#[derive(Clone, Debug, Default)]
pub struct Write<U = SystemUtils> {
    utils: U,
}

impl Write {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Write<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Write<U>
where
    S: HasPlan + Send,
    U: Utils + Sync,
{
    type Outcome = ();
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<(), Error> {
        let plan = state.plan();
        let mut generated: Vec<(Target, Vec<String>)> = Vec::new();
        for change in &plan.changes {
            let path = change.path();
            match change {
                Change::Write {
                    contents, format, ..
                } => {
                    if let Some(target) = *format {
                        match generated.iter_mut().find(|(known, _)| *known == target) {
                            Some((_, paths)) => paths.push(path.to_string()),
                            None => generated.push((target, vec![path.to_string()])),
                        }
                    }
                    self.utils.write(path, contents)
                }
                Change::Symlink { target, .. } => self.utils.symlink(path, target),
            }
            .map_err(|source| Error::Io {
                path: path.to_string(),
                source,
            })?;
            self.utils.print(&format!("wrote {path}"));
        }
        for (target, paths) in &generated {
            self.utils.format(*target, paths);
        }
        if !plan.hint.is_empty() {
            self.utils.print(&plan.hint);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::Plan;
    use crate::testing::{Fake, State};

    fn state(changes: Vec<Change>, hint: &str) -> State {
        State {
            plan: Plan {
                changes,
                hint: hint.to_string(),
            },
            ..State::default()
        }
    }

    #[tokio::test]
    async fn writes_every_change_formats_generated_code_and_prints_the_hint() {
        let fake = Fake::default();
        let mut state = state(
            vec![
                Change::rust("src/nodes/ask.rs", "pub struct Ask;"),
                Change::write("src/nodes/mod.rs", "pub mod ask;\n"),
                Change::Symlink {
                    path: ".claude/skills".to_string(),
                    target: "../.agents/skills".to_string(),
                },
            ],
            "Next: write the tests.",
        );

        Write::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(
            fake.file("src/nodes/ask.rs").as_deref(),
            Some("pub struct Ask;")
        );
        assert_eq!(
            fake.file("src/nodes/mod.rs").as_deref(),
            Some("pub mod ask;\n")
        );
        assert_eq!(
            fake.file(".claude/skills").as_deref(),
            Some("-> ../.agents/skills")
        );
        assert_eq!(*fake.formatted.lock().unwrap(), ["src/nodes/ask.rs"]);
        assert_eq!(
            fake.printed(),
            [
                "wrote src/nodes/ask.rs",
                "wrote src/nodes/mod.rs",
                "wrote .claude/skills",
                "Next: write the tests.",
            ]
        );
    }

    #[tokio::test]
    async fn stops_at_the_first_file_it_cannot_write() {
        let fake = Fake {
            unwritable: Some("src/nodes/mod.rs"),
            ..Fake::default()
        };
        let mut state = state(
            vec![
                Change::write("src/nodes/mod.rs", "pub mod ask;\n"),
                Change::write("src/nodes/ask.rs", "pub struct Ask;"),
            ],
            "Next.",
        );

        let result = Write::with_utils(&fake).run(&mut state).await;

        assert!(matches!(result, Err(Error::Io { path, .. }) if path == "src/nodes/mod.rs"));
        assert_eq!(fake.file("src/nodes/ask.rs"), None);
        assert!(fake.printed().is_empty());
    }
}
