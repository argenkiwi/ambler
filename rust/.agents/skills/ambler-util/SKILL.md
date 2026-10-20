---
name: ambler-util
description: Creates or extracts utility modules into the src/utils/ directory of an Ambler Rust project. Use this whenever a node's DefaultUtils needs external crates, contains logic reusable across multiple nodes, or has implementations too complex to inline — even if the user just says "add a util", "extract this helper", "share this function", "create a utility", or "this code is getting too long for the node". When a user creates a node that pulls in a new crate or has complex async logic, proactively suggest using this skill.
metadata:
  author: leandro
  version: "2.0-rust"
---

# Ambler Util (Rust)

Follow these steps to create or extract a utility module in the `src/utils/` directory.

## When to create a utility

Create a `src/utils/<name>.rs` file when a node's `DefaultUtils` contains:

1. **External dependencies** — any crate not already in `Cargo.toml` (e.g., `reqwest`, `serde_json`)
2. **Reusable logic** — code that is or could be used by more than one node
3. **Complex implementations** — functions with significant logic (error handling, retries, client caching, multi-step operations) that clutter the node file

Simple one-liners like `println!("{message}")` or `rand::random::<f64>()` belong in `DefaultUtils` directly — no need to extract them.

---

## 1. Gather requirements

Before writing, determine:

- **Utility name**: What capability does this module provide? (e.g., `ollama_chat`, `file_writer`, `markdown_parser`). File will be named `src/utils/<name>.rs` (`snake_case`).
- **Functions to export**: What are the signatures? What arguments and return types?
- **Dependencies**: Does it require extra crates? Which ones and which features?
- **Nodes that will use it**: Which node(s) will call this utility from their `DefaultUtils`?

If any of the above is unclear, ask the user before writing code.

---

## 2. Create `src/utils/<name>.rs`

Export functions that implement the side-effectful work. Keep the module focused on a single capability.

```rust
// Example: src/utils/journal.rs
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

/// File that journal entries are appended to.
const JOURNAL_PATH: &str = "journal.log";

// Stateful resources live here and stay private to the module.
static WRITE_LOCK: Mutex<()> = Mutex::const_new(());

/// Appends `entry` as a new line to the journal file, creating the file if needed.
///
/// Returns `false` if the entry could not be written.
pub async fn append(entry: String) -> bool {
    let _guard = WRITE_LOCK.lock().await;
    write_line(&entry).await.is_ok()
}

async fn write_line(entry: &str) -> std::io::Result<()> {
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(JOURNAL_PATH)
        .await?;
    file.write_all(format!("{entry}\n").as_bytes()).await
}
```

Register the module:

- Add `pub mod <name>;` to `src/utils/mod.rs` (create it if it doesn't exist).
- If this is the first utility, also add `pub mod utils;` to `src/lib.rs`, next to `pub mod nodes;`.

### Rules

- **Export only functions** — never export clients, connections or `static`s. Callers interact through function calls with clear signatures.
- **Keep state private** — if the utility needs to cache a client or serialise access, use a private `static` (e.g., `std::sync::OnceLock` or `tokio::sync::Mutex::const_new`), never a `pub` one.
- **Match the node's `Utils` trait** — return the same type the consuming node's `Utils` method declares, so `DefaultUtils` is a one-line delegation.
- **Take owned arguments in async functions** — a node's async util returns `BoxFuture<'static, T>`, so the utility's future must not borrow from the caller. Take `String` rather than `&str` (or clone before the `async` block).
- **No Ambler imports** — utility files must not import from `crate::Node`, `crate::Ambler` or `crate::nodes`. They are plain Rust helpers.
- **Extract URL and config constants** — don't inline magic strings like API base URLs or file paths. Pull them into named `const` items at the top of the file so they're easy to find and change.

### Doc comments

Every exported function gets a `///` doc comment. This helps both humans reading the code and agents using the utility in future nodes understand what it does without having to trace the implementation.

```rust
/// Fetches the current temperature for a city from the OpenWeather API.
///
/// Returns the temperature in Celsius, or `None` if the request failed.
pub async fn fetch_weather(city: String, api_key: String) -> Option<f64> {
```

Keep it brief — a one-sentence summary, then a line on what it returns. Name parameters in backticks where they need explaining. Skip a `# Errors` or `# Panics` section unless the function has a documented, meaningful contract.

---

## 3. Update `Cargo.toml` (if using extra crates)

If the utility needs a crate that isn't already a dependency, add it with Cargo:

```bash
cargo add reqwest --features json
```

This updates `[dependencies]` in `Cargo.toml`. Check `Cargo.toml` first — the crate may already be listed (`tokio` with `full`, `futures` and `rand` are there by default).

---

## 4. Connect to the node

Import the utility in the node file and use it in `DefaultUtils`. The `Utils` trait stays unchanged — only the implementation on `DefaultUtils` changes.

```rust
use crate::utils::journal;
use futures::future::BoxFuture;

pub trait Utils: Send + Sync {
    fn log(&self, entry: &str) -> BoxFuture<'static, bool>;
}

pub struct DefaultUtils;

impl Utils for DefaultUtils {
    fn log(&self, entry: &str) -> BoxFuture<'static, bool> {
        Box::pin(journal::append(entry.to_string()))
    }
}
```

Tests still inject a `MockUtils` and never touch the utility — that's the point of the `Utils` trait.

---

## 5. Checklist before finishing

- [ ] `src/utils/<name>.rs` exists and exports the required functions.
- [ ] `pub mod <name>;` is in `src/utils/mod.rs`, and `pub mod utils;` is in `src/lib.rs`.
- [ ] All extra crates are listed in `Cargo.toml`.
- [ ] Exported function return types match the node's `Utils` methods.
- [ ] Async utility functions take owned arguments so their futures are `'static`.
- [ ] Stateful resources (clients, caches, locks) are private `static`s in the util module.
- [ ] URL/config constants are extracted to named `const` items.
- [ ] Every exported function has a `///` doc comment.
- [ ] The node's `DefaultUtils` delegates to the utility function.
- [ ] No `crate::Node`, `crate::Ambler` or `crate::nodes` imports appear in the utility file.
- [ ] Node tests still use mocked utils — they need no changes (`cargo test`).
