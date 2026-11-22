# Program Specifications

`amble walk <name> --nodes <start,next,...> [--edge <node>.<Outcome>=<node|halt>]... [--no-runner]` wires existing nodes into a new walk in the project in the current directory.

## Nodes

### Plan Walk
- Initial node.
- Reads the walk's name, its nodes (the first is the start) and the edges from the arguments, and each node's outcomes, error, state traits and utils from the project.
- Works out the walk file, with the given edges wired and the others left to be written, and the walks module with it registered. It adds the walk's spec if there is none, and a runner unless `--no-runner` was given: the main binary if the project has none, otherwise a binary named after the walk.
- In a TypeScript project it reads each node's edges, state and utils instead, and works out the walk file, its wiring test in `walks/tests/` and its spec; nothing is registered, a node whose only edge is `onDone` ends the walk unless an edge is given for it, and the runner is a `package.json` script named after the walk. If the nodes give a state field different types, it records why and transitions to `REPORT`.
- If everything fits, it transitions to `WRITE`.
- If the arguments are invalid, a node does not exist yet, an edge names an unknown node or outcome, the walk or its runner already exists or this is not an Ambler project, it records why and transitions to `REPORT`.

### Write
- Writes the planned files, formats the generated walk, and prints each path and then what to do next.
- Terminates. If a file cannot be written the program fails with that error.

### Report
- Prints the recorded reason on standard error and terminates. The program exits 1.

## Shared State

The shared state consists of:
- `target`: the language of the project.
- `args`: the arguments after the verb.
- `plan`: the files to write, and the next steps to print afterwards.
- `message`: why the verb failed, if it did.
