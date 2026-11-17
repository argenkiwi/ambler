# Program Specifications

`amble node <name> (--outcomes <a,b> | --terminal) [--fallible] [--state <field>[:<Type>]]...` adds a node skeleton to the project in the current directory.

## Nodes

### Plan Node
- Initial node.
- Reads the node's name, outcomes, whether it can fail and the state it needs from the arguments, and the registered nodes and state traits from the project.
- Works out the node file (its contract, an unimplemented body and one failing test per branch) and the new contents of the nodes module: the node registered in alphabetical order, plus a new state trait for each `--state <field>:<Type>`.
- If everything fits, it transitions to `WRITE`.
- If the arguments are invalid, the node or a new trait already exists, a reused trait is missing or this is not an Ambler project, it records why and transitions to `REPORT`.

### Write
- Writes the planned files, formats the generated node, and prints each path and then what to do next.
- Terminates. If a file cannot be written the program fails with that error.

### Report
- Prints the recorded reason on standard error and terminates. The program exits 1.

## Shared State

The shared state consists of:
- `args`: the arguments after the verb.
- `plan`: the files to write, and the next steps to print afterwards.
- `message`: why the verb failed, if it did.
