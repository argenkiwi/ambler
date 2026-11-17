# Program Specifications

`amble new <name> [--example]` creates a new Ambler project in a folder named after it.

## Nodes

### Plan Project
- Initial node.
- Reads the project name and the `--example` switch from the arguments and works out every file of the project, with the counter walk if the example was asked for.
- If the arguments are valid and no folder with that name exists, it transitions to `WRITE`.
- If the name is missing or invalid, a flag is unknown or the folder exists, it records why and transitions to `REPORT`.

### Write
- Writes the planned files and links, creating folders as needed, and prints each path and then what to do next.
- Terminates. If a file cannot be written the program fails with that error.

### Report
- Prints the recorded reason on standard error and terminates. The program exits 1.

## Shared State

The shared state consists of:
- `args`: the arguments after the verb.
- `plan`: the files and links to write, and the next steps to print afterwards.
- `message`: why the verb failed, if it did.
