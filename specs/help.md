# Program Specifications

`amble help [topic]` prints a guide. It also runs when amble is given no verb, and when it is given a verb it does not know.

## Nodes

### Guide
- Initial node.
- Takes the topic from the first argument, or the general guide if there is none.
- If there is a guide for the topic, it prints it on standard output and terminates.
- If there is not, it records that and the list of topics, and transitions to `REPORT`.

### Report
- Prints the recorded reason on standard error and terminates. The program exits 1.

## Shared State

The shared state consists of:
- `args`: the arguments after the verb; the first is the topic.
- `message`: why the verb failed, if it did.
