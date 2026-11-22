# Program Specifications

`amble util <name>` adds an empty utility module to the project in the current directory.

## Nodes

### Plan Util
- Initial node.
- Reads the module name from the arguments and the registered utilities from the project, and works out the new module file and the utilities module with it registered in alphabetical order.
- In a TypeScript project it works out only the module file in `utils/`: nothing is registered.
- If the name is valid and free, it transitions to `WRITE`.
- If the name is missing or invalid, the module exists or this is not an Ambler project, it records why and transitions to `REPORT`.

### Write
- Writes the planned files and prints each path and then what to do next.
- Terminates. If a file cannot be written the program fails with that error.

### Report
- Prints the recorded reason on standard error and terminates. The program exits 1.

## Shared State

The shared state consists of:
- `target`: the language of the project.
- `args`: the arguments after the verb.
- `plan`: the files to write, and the next steps to print afterwards.
- `message`: why the verb failed, if it did.
