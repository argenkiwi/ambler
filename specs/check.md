# Program Specifications

`amble check [--no-cargo]` verifies that the project in the current directory follows the Ambler conventions, then that it is formatted, free of lint warnings and passes its tests.

The walk starts at `INSPECT`. If an argument other than `--no-cargo` is given, it starts at `REPORT` with the usage as the reason.

## Nodes

### Inspect
- Reads the project's manifest, sources and specs and checks them against the structural conventions, printing one line per violation with its file, line and rule.
- If there are no violations, it transitions to `VERIFY`, or terminates if `--no-cargo` was given.
- If there are violations, it records how many and transitions to `REPORT`.
- If the project cannot be read the program fails with that error.

### Verify
- Runs the formatting check, the linter with warnings denied and the tests, in that order, printing each command before running it.
- If all pass, it terminates.
- If one fails, it skips the rest, records which and transitions to `REPORT`.
- If cargo cannot be run the program fails with that error.

### Report
- Prints the recorded reason on standard error and terminates. The program exits 1.

## Shared State

The shared state consists of:
- `message`: why the check failed, if it did.
