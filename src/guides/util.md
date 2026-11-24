# ambler util

    ambler util <name>

Writes `src/utils/<name>.rs` and registers it in `src/utils/mod.rs`. Run it from the project root.

## When to extract

Move a `SystemUtils` method's implementation into `src/utils/` when it needs an external crate, is used by more than one node, or is too long to read at a glance. One-liners such as `println!` stay in `SystemUtils`.

## Rules for a utility module

- Export functions only. No structs to construct, no traits.
- State (a client, a lock, a cache) lives in a private `static`: `OnceLock`, `tokio::sync::Mutex::const_new`.
- Return exactly the type the node's `Utils` method returns, so `SystemUtils` delegates in one line.
- Anything that can fail returns a `Result`, not a `bool` or an `Option`.
- Async functions return `Send` futures.
- Import nothing from `crate::nodes`, `crate::walks` or the engine. Utilities know nothing of nodes.
- URLs, paths and other fixed values are named `const`s.
- Document every exported function with `///`, with an `# Errors` section if it can fail.

## Crates

Add what the module needs with `cargo add <crate>`. `tokio` is present with the `time`, `macros` and `rt-multi-thread` features; add others the same way, for example `cargo add tokio --features fs,io-util`.

## Connecting it to a node

    use crate::utils::journal;

    pub trait Utils {
        fn log(&self, entry: &str) -> impl Future<Output = std::io::Result<()>> + Send;
    }

    impl Utils for SystemUtils {
        fn log(&self, entry: &str) -> impl Future<Output = std::io::Result<()>> + Send {
            journal::append(entry)
        }
    }

The node's tests do not change: they mock `Utils` and never reach the utility.
