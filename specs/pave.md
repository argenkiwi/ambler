# Program Specifications

`amble pave <spec> [--agent '<command>'] [--max-iterations <n>] [--max-repairs <n>] [--dry-run]` implements `specs/<spec>.md` in the project in the current directory by running a coding agent in a loop, one narrow task per run: create or finish a node, wire the walk, or repair a failing check. It commits a checkpoint after every passing check and stops once the walk exists and the check passes.

## Nodes

### Plan Pave
- Initial node.
- Reads the spec name, the agent command (from `--agent`, or else the `AMBLE_AGENT` environment variable), the limits and whether this is a dry run from the arguments, and where to keep the check log and, for a Rust project, the build output.
- If everything fits, it transitions to `CHECK_PROJECT`.
- If the arguments are invalid, no agent command is given outside a dry run, the spec does not exist, this is not an Ambler project of the target or not a git repository, it records why and transitions to `REPORT`.

### Check Project
- Runs `amble check`, keeping its output in the log, and records whether it passed and the end of its output.
- If it passes, it transitions to `CHECKPOINT`.
- If it fails, it prints the last lines of the output and transitions to `CHOOSE_TASK`.
- If the check cannot be run the program fails with that error.

### Checkpoint
- Commits every change as a checkpoint named after the last task, if there are any, and resets the repair count. In a dry run it only says what it would commit.
- Transitions to `CHOOSE_TASK`. If git fails the program fails with that error.

### Choose Task
- Works out the next task from the spec's nodes and the project: finish a node that still has unfinished code, create the next missing node if the check passes, wire the walk once every node is done, or else repair the failing check.
- If the walk exists and the check passes, it prints that the spec is done and terminates.
- If the check has failed after as many repairs in a row as allowed, it transitions to `ROLL_BACK`.
- If it already rolled back once and is still failing, the same task was chosen three times in a row while the check passed, or the iteration limit is reached, it records why and transitions to `REPORT`.
- Otherwise it records the task and transitions to `BRIEF_AGENT`.
- If the spec cannot be read the program fails with that error.

### Roll Back
- Stashes every change since the last checkpoint, naming the task that failed, and records that the one retry is used. In a dry run it only says what it would stash.
- Transitions to `CHECK_PROJECT`. If git fails the program fails with that error.

### Brief Agent
- Writes a prompt holding only what the task needs, in the target's files, tools and terms: the node's section of the spec and the shared state, where the existing state is declared, or the whole spec for the walk, with where to read each node's outcomes; or the check output and the uncommitted changes for a repair.
- In a dry run it prints the prompt and terminates.
- Otherwise it runs the agent with the prompt, counts a repair if the check was failing, and transitions to `CHECK_PROJECT`. An agent that exits with an error is noted and the check decides.
- If a file cannot be read or the agent cannot be started the program fails with that error.

### Report
- Prints the recorded reason on standard error and terminates. The program exits 1.

## Shared State

The shared state consists of:
- `target`: the language of the project.
- `args`: the arguments after the verb.
- `paving`: the settings, the last check result, the current task and the counters that bound the loop.
- `message`: why the verb failed, if it did.
