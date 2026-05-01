name: ambler-util
description: Creates or extracts utility modules into the utils/ directory of an Ambler project. Use this whenever a node's DefaultUtils needs external dependencies, contains logic reusable across multiple nodes, or has implementations too complex to inline.
metadata:
  author: leandro
  version: "1.0-rust"
---

# Ambler Util (Rust)

Follow these steps to create or extract a utility module in the `src/utils/` directory.

## 1. Gather requirements

Before writing, determine:

- **Utility name**: What capability does this module provide? File will be named `src/utils/<name>.rs`.
- **Functions to export**: What are the signatures?
- **Dependencies**: Does it require extra crates? Add them to `Cargo.toml`.

---

## 2. Create `src/utils/<name>.rs`

Export functions or structs that implement the side-effectful work.

```rust
// Example: src/utils/logger.rs

pub fn log_to_file(message: &str, path: &str) {
    // Implementation...
}
```

### Rules

- **Naming**: Use `snake_case` for filenames and functions.
- **Registration**: Add `pub mod <name>;` to `src/utils/mod.rs`.
- **Documentation**: Add doc comments (`///`) to exported functions.

---

## 3. Connect to the node

Import the utility in the node file and use it in `DefaultUtils`.

```rust
use crate::utils::logger;

impl Utils for DefaultUtils {
    fn log(&self, message: &str) {
        logger::log_to_file(message, "log.txt");
    }
}
```

---

## 4. Checklist before finishing

- [ ] `src/utils/<name>.rs` exists.
- [ ] `src/utils/mod.rs` is updated.
- [ ] `Cargo.toml` is updated with any new dependencies.
- [ ] The node's `DefaultUtils` delegates to the utility.
