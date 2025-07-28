## Guidelines for Python Ambler Applications

### State and Nodes

- **State**: The state of the application can be any Python object.
- **Nodes**: Nodes should be represented using a Python `Enum` for type safety and clarity.

### Node Functions

- Each node should have a corresponding function that takes the current state as input and returns a tuple containing the new state and the next node to transition to. If the next node is `None`, the `amble` function will terminate.

### Dispatching

- A dictionary should be used to map nodes to their corresponding functions. This provides a clean and efficient way to dispatch to the correct function based on the current node.