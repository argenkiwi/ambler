# Ambler

Ambler is a lightweight, type-safe state machine framework implemented in TypeScript and designed to run on Deno. It is used to create "walks"—interactive, node-based execution flows that guide a user through a sequence of steps, decisions, and events.

## Core Concepts

At its heart, Ambler uses a graph-based execution model:

- **Nodes**: Individual steps in a process. Each node is a function that accepts a state and returns the next step in the sequence.
- **Walks**: A complete graph of nodes wired together to form a specific user journey (e.g., a text adventure or a setup wizard).
- **State**: A strongly typed object passed between nodes, allowing the walk to maintain context, accumulate data, and make decisions based on previous actions.
- **Amble**: The execution engine that drives the machine from the starting node through the sequence until a terminal node (returning `null`) is reached.

## Project Structure

The project is organized by the type of logic being implemented:

- `ambler.ts`: The core engine implementation.
- `nodes/`: Contains reusable, decoupled node implementations. Nodes are designed with dependency injection (passing callbacks and utilities) to ensure they are testable and decoupled from the graph structure.
- `walks/`: Contains the actual implementations of executable journeys (walks) where nodes are composed into a graph.
- `specs/`: Plain-language design documents and specifications that describe the expected behavior of a walk before implementation.
- `tests/`: (Found alongside nodes) Unit tests for individual node logic.

## Creating a Walk

To create a new walk, follow these steps:

1.  **Define the State**: Create an interface representing the data that needs to persist throughout the walk.
2.  **Draft a Spec**: (Optional but recommended) Write a description of the desired flow in the `specs/` directory.
3.  **Compose Nodes**: Create a new file in `walks/`. 
    - Define your `initialState`.
    - Map your nodes in a `nodes` record.
    - Use the `node()` helper to wrap node factories. This allows for circular references (e.g., a decision node looping back to a prompt node) without TypeScript circular dependency errors.
4.  **Bind Transitions**: Use callbacks (e.g., `onSuccess`, `onSelect`) provided by the nodes to point to the next node in your graph.

Example of a node connection in a walk:
```typescript
const nodes: Record<string, Nextable<State>> = {
  start: node(() => StartNode.create({ onSuccess: nodes.nextStep })),
  nextStep: node(() => NextNode.create({ onSuccess: nodes.end })),
  end: node(() => node(() => null)) // Terminal node
};
```

## Running Walks

You can run the walks directly using the Deno runtime.

### Running a specific walk
To execute a walk (for example, the `cyoa` walk):

```bash
deno run typescript/walks/cyoa.ts
```

### Development mode
To run the project with file watching enabled (useful during development):

```bash
deno task dev
```

## Testing

Testing is done at the node level. Each node should have a corresponding `.test.ts` file.

```bash
# Run all tests
deno test

# Run a specific node test
deno test typescript/nodes/chatPromptNode.test.ts
```
