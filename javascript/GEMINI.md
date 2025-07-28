## Guidelines for JavaScript Ambler Applications

### State and Nodes

- **State**: The state of the application can be any JavaScript object.
- **Nodes**: Nodes should be represented using a frozen object (`Object.freeze`) for clarity and to prevent accidental modification.

### Node Functions

- Each node should have a corresponding function that takes the current state as input and returns a tuple (array) containing the new state and the next node to transition to. If the next node is `null`, the `amble` function will terminate.

### Dispatching

- An object literal should be used to map nodes to their corresponding functions. This provides a clean and efficient way to dispatch to the correct function based on the current node.