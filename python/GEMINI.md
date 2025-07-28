# Ambler Application Guidelines

When building applications using `ambler.py`, please adhere to the following guidelines to ensure consistency and maintainability.

> IMPORTANT: Never modify the code in `ambler.py`!

## 0. Specifications

The specifications of the program should be described in SPECS.md. If the file does not exist, create one reflecting the user request and ask the user to confirm they are happy with it before proceeding.

## 1. Identify Program Stages (Nodes)

First, break down your program's logic into distinct steps or stages. These will become the "nodes" of your application's flow.

For example, a simple counter application might have the following stages:
- `START`: Initializes the process.
- `STEP`: Performs the counting action.
- `STOP`: Terminates the process.

## 2. Define Nodes

Create a class, typically named `Node`, to define these stages as constants.

```python
class Node:
    START = 1
    STEP = 2
    STOP = 3
```

## 3. Determine Shared State

Decide what data needs to be passed between the different nodes. This will be your "state" object. It can be a simple type (like an integer for a counter) or a more complex data structure (like a dictionary or a custom class).

In the counter example, the state is an integer representing the current count.

## 4. Create Node Functions

For each node, create a corresponding function that takes the current `state` as a parameter. Each function should return a tuple containing:
1. The (potentially modified) `state`.
2. A value that will be used to decide which node to go to next. This can be `None` if there's only one possible next step.

```python
import random
import asyncio

class Node:
    START = 1
    STEP = 2
    STOP = 3

def start(state):
    print("Let's count...")
    return state, Node.STEP

async def step(state):
    count = state + 1
    await asyncio.sleep(1)
    print(f"...{count}...")
    # Return a boolean to decide whether to continue
    return count, Node.STEP if random.choice([True, False]) else Node.STOP

def stop(state):
    print("...stop.")
    return state, None
```

## 5. Start the Application

Finally, in your main execution block, call the `amble` function, passing the initial state, the starting node, and a `step` function. This function acts as the central router for your application. It takes the current `state` and `Node` as input, calls the appropriate node function and returns the updated state and the next `Node` to be called.

```python
from ambler import amble
import asyncio

# ... (Node definitions and functions)

async def direct(state, node):
    if node == Node.START:
        return start(state)
    elif node == Node.STEP:
        return await step(state)
    elif node == Node.STOP:
        return stop(state)

async def main():
    await amble(0, Node.START, direct)

if __name__ == "__main__":
    asyncio.run(main())
```
