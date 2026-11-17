//! Terminal and standard I/O utilities.

use std::io::{self, Write};

/// Prompts with `message` on stdout, flushes stdout, and reads a line from stdin.
///
/// Returns `None` at EOF. Trailing newlines (`\r`, `\n`) are stripped.
///
/// # Errors
///
/// Returns an [`io::Error`] if stdout cannot be flushed or stdin cannot be read.
pub fn prompt(message: &str) -> io::Result<Option<String>> {
    print!("{message}");
    io::stdout().flush()?;
    let mut input = String::new();
    Ok(match io::stdin().read_line(&mut input)? {
        0 => None,
        _ => Some(input.trim_end_matches(['\r', '\n']).to_string()),
    })
}
