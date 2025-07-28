# Ambler Application Guidelines

When building applications using `ambler.go`, please adhere to the following guidelines to ensure consistency and maintainability.

> IMPORTANT: Never modify the code in `ambler.go`!

## 1. Identify Program Stages (Nodes)

First, break down your program's logic into distinct steps or stages. These will become the "nodes" of your application's flow.

For example, a simple counter application might have the following stages:
- `NodeStart`: Initializes the process.
- `NodeStep`: Performs the counting action.
- `NodeStop`: Terminates the process.

## 2. Define Nodes

Create a `const` block, typically named `Node`, to define these stages using `iota`.

```go
type Node int

const (
	NodeStart Node = iota
	NodeStep
	NodeStop
)
```

## 3. Determine Shared State

Decide what data needs to be passed between the different nodes. This will be your "state" object. It can be a simple type (like an integer for a counter) or a more complex data structure (like a `struct` or a custom type).

In the counter example, the state is an integer representing the current count.

## 4. Create Node Functions

For each node, create a corresponding function that takes the current `state` as a parameter. Each function should return a tuple containing:
1. The (potentially modified) `state`.
2. A value that will be used to decide which node to go to next. This can be `nil` if there's only one possible next step.

```go
import (
	"fmt"
	"math/rand"
)

func start(state int) (int, any) {
	fmt.Println("Let's count...")
	return state, nil
}

func step(state int) (int, bool) {
	count := state + 1
	fmt.Printf("...%d...
", count)
	// Return a boolean to decide whether to continue
	return count, rand.Intn(2) == 0
}

func stop(state int) (int, any) {
	fmt.Println("...stop.")
	return state, nil
}
```

## 5. Create the `direct` Function

This function acts as the central router for your application. It takes the current `state` and `node` as input and uses the `ambler.Resolve` function to determine the next step.

The `ambler.Resolve` function takes two arguments:
1. A `struct` containing the result of calling the appropriate node function (the `State` and `Action` fields).
2. A lambda function that takes the `Action` from the node function's result and returns the next `Node` to execute.

```go
import (
	"context"
	"/home/argenkiwi/Code/ambler/go" // Adjust import path as needed
)

func direct(ctx context.Context, state int, node Node) (int, Node, error) {
	switch node {
	case NodeStart:
		// After 'start', always go to 'NodeStep'
		return ambler.Resolve(struct{State int; Action any}{start(state)}, func(_ any) Node { return NodeStep })
	case NodeStep:
		// After 'step', check the boolean to either loop or stop
		return ambler.Resolve(struct{State int; Action bool}{step(state)}, func(shouldContinue bool) Node {
			if shouldContinue {
				return NodeStep
			} else {
				return NodeStop
			}
		})
	case NodeStop:
		// 'NodeStop' is the final node, so we return -1 (or a suitable "end" node)
		return ambler.Resolve(struct{State int; Action any}{stop(state)}, func(_ any) Node { return -1 })
	}
	return state, -1, nil
}
```

## 6. Start the Application

Finally, in your `main` function, call the `ambler.Amble` function, passing the context, initial state, the starting node, and a reference to your `direct` function.

```go
import (
	"context"
	"/home/argenkiwi/Code/ambler/go" // Adjust import path as needed
)

func main() {
	// Start with an initial state of 0 at the NodeStart node
	ambler.Amble(context.Background(), 0, NodeStart, direct)
}
