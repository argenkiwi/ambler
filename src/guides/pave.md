# amble pave

    amble pave <spec> [--agent '<command>'] [--max-iterations 30] [--max-repairs 3] [--dry-run]

Implements `specs/<spec>.md` by running a coding agent in a loop, one small task per run, so the agent only ever sees one narrow piece of work. That suits agents with small context windows, local models included. pave decides what to do next from the project; the agent only does that one thing.

Run it from the root of an Ambler project in a git repository, once the spec is written (`amble help spec`).

- `--agent 'claude -p'`: the command that runs one agent turn. pave adds the prompt as its last argument. If it is not given, the `AMBLE_AGENT` environment variable is used. There is no default.
- `--max-iterations`: how many agent runs at most.
- `--max-repairs`: how many failed checks in a row before rolling back.
- `--dry-run`: run the check, print the next task's prompt and stop. Nothing is committed and no agent runs.
- `AMBLE_PAVE_TARGET`: in a Rust project, the `CARGO_TARGET_DIR` for pave's own checks (default `target/amble-pave`), so they don't overwrite another toolchain's `target/`, a container's for example.

## Each iteration

1. Run `amble check`. If it passes, commit a checkpoint if anything changed. If it fails, the output goes into the next prompt.
2. Pick the task. The spec's `###` headings, in snake_case, are the node names, in order:
   - a node file that still has unfinished code: finish that node;
   - a missing node, while the check passes: create it;
   - every node done and no walk yet: wire the walk, with `--nodes` filled in and each node's outcomes listed;
   - the check failing for any other reason: repair it.
3. Run the agent with a prompt that holds only what the task needs. A node's prompt has its own section of the spec, the shared state and the state the project's nodes already have.

pave stops with exit 0 once the walk (`src/walks/<spec>.rs`, `walks/<spec>.ts` or `src/<package>/walks/<spec>.py`) exists and the check passes.

## Safeguards

- Checkpoints: a commit after every passing check, so `HEAD` is always a known-good state.
- Resuming: run the same command again. Unfinished nodes and walks are finished, not started over.
- Repair budget: after `--max-repairs` failed checks in a row, pave runs `git stash push -u` to get back to the last checkpoint (nothing is reset), then retries once. If that fails too, it exits 1.
- No progress: if the same task is picked three times in a row while the check passes, it exits 1.
- The full output of the last check is in `.git/amble-pave-check.log`.

## In a container

When the agent runs in a container, run pave there too, so the agent and the checks share a toolchain. Install the `amble` command in the image, set `AMBLE_PAVE_TARGET=/workspace/target` for a Rust project (or wherever the agent builds) and make sure git has an identity to commit with.
