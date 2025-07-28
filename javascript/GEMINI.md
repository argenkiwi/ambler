# Ambler Application Guidelines

When building applications using `ambler.js`, please adhere to the following guidelines to ensure consistency and maintainability.

> IMPORTANT: Never modify the code in `ambler.js`!

## 1. Identify Program Stages (Nodes)

First, break down your program's logic into distinct steps or stages. These will become the "nodes" of your application's flow.

For example, a simple counter application might have the following stages:
- `START`: Initializes the process.
- `STEP`: Performs the counting action.
- `STOP`: Terminates the process.

## 2. Define Nodes

Create an `object`, typically named `Node`, to define these stages as constants.

```javascript
const Node = {
    START: 1,
    STEP: 2,
    STOP: 3,
};
```

## 3. Determine Shared State

Decide what data needs to be passed between the different nodes. This will be your "state" object. It can be a simple type (like a `number` for a counter) or a more complex data structure (like an `object` or a custom class).

In the counter example, the state is a number representing the current count.

## 4. Create Node Functions

For each node, create a corresponding function that takes the current `state` as a parameter. Each function should return an `Array` containing:
1. The (potentially modified) `state`.
2. A value that will be used to decide which node to go to next. This can be `null` if there's only one possible next step.

```javascript
const Node = {
    START: 1,
    STEP: 2,
    STOP: 3,
};

function start(state) {
    console.log("Let's count...");
    return [state, Node.STEP];
}

async function step(state) {
    const count = state + 1;
    await new Promise(resolve => setTimeout(resolve, 1000));
    console.log(`...${count}...`);
    return [count, Math.random() < 0.5 ? Node.STEP : Node.STOP];
}

function stop(state) {
    console.log("...stop.");
    return [state, null];
}
```

## 5. Start the Application

Finally, in your main execution block, call the `amble` function, passing the initial state, the starting node, and a `step` function. This function acts as the central router for your application. It takes the current `state` and `Node` as input, calls the appropriate node function and returns the updated state and the next `Node` to be called.

```javascript
const { amble } = require('./ambler');

// ... (Node definitions and functions)

async function direct(state, node) {
    if (node === Node.START) {
        return start(state);
    } else if (node === Node.STEP) {
        return await step(state);
    } else if (node === Node.STOP) {
        return stop(state);
    }
}

(async () => {
    await amble(0, Node.START, direct);
})();
```
