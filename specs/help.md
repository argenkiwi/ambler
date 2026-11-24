# Program Specifications

`ambler help [topic]` prints a guide. It also runs when ambler is given no verb, and when it is given a verb it does not know.

## Nodes

### Guide
- Initial node.
- Takes the topic from the first argument, or the general guide if there is none.
- If there is a guide for the topic, it prints the one for the project's target (the shared one if the target has none of its own) on standard output and terminates.
- If there is not, it records that and the list of topics, and transitions to `REPORT`.

### Report
- Prints the recorded reason on standard error and terminates. The program exits 1.

## Shared State

The shared state consists of:
- `target`: the language of the project, which picks the guides.
- `args`: the arguments after the verb; the first is the topic.
- `message`: why the verb failed, if it did.
