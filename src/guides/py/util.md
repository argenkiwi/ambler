# amble util

    amble util <name>

Writes `src/<package>/utils/<name>.py`. Run it from the project root. Nothing is registered: nodes import it by its module path.

## When to extract

Move a `SystemUtils` method's body into `utils/` when it needs a package, is used by more than one node, or is too long to read at a glance. One-liners such as `print` stay in `SystemUtils`.

## Rules for a utility module

- Plain functions. No classes to construct.
- State (a client, a cache) lives in a private module-level name.
- A failure the node must handle is in the return type (`str | None`); an unexpected one raises, and the docstring says what (`Raises: OSError: …`).
- Import nothing from the package's `engine`, `nodes` or `walks`. Utilities know nothing of nodes.
- URLs, paths and other fixed values are named constants at the top of the module.
- Annotate every parameter and return type: pyright runs in strict mode.
- Give each function a docstring.

## Packages

Add what the module needs with `uv add <package>`.

## Connecting it to a node

    from my_app.utils import journal


    class Utils(Protocol):
        def log(self, entry: str) -> None: ...


    class SystemUtils:
        def log(self, entry: str) -> None:
            journal.append(entry)

The node's tests do not change: they pass their own fake and never reach the utility.
