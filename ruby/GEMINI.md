# Ambler Application Guidelines

When building applications using `ambler.rb`, please adhere to the following guidelines to ensure consistency and maintainability.

> IMPORTANT: Never modify the code in `ambler.rb`!

## 1. Identify Program Stages (Nodes)

First, break down your program's logic into distinct steps or stages. These will become the "nodes" of your application's flow.

For example, a simple counter application might have the following stages:
- `START`: Initializes the process.
- `STEP`: Performs the counting action.
- `STOP`: Terminates the process.

## 2. Define Nodes

Create a `module`, typically named `Node`, to define these stages as constants.

```ruby
module Node
  START = 1
  STEP = 2
  STOP = 3
end
```

## 3. Determine Shared State

Decide what data needs to be passed between the different nodes. This will be your "state" object. It can be a simple type (like an integer for a counter) or a more complex data structure (like a `Hash` or a custom class).

In the counter example, the state is an integer representing the current count.

## 4. Create Node Functions

For each node, create a corresponding function that takes the current `state` as a parameter. Each function should return an `Array` containing:
1. The (potentially modified) `state`.
2. A value that will be used to decide which node to go to next. This can be `nil` if there's only one possible next step.

```ruby
def start(state)
  puts "Let's count..."
  [state, nil]
end

def step(state)
  count = state + 1
  puts "...#{count}..."
  # Return a boolean to decide whether to continue
  [count, [true, false].sample]
end

def stop(state)
  puts "...stop."
  [state, nil]
end
```

## 5. Create the `direct` Function

This function acts as the central router for your application. It takes the current `state` and `node` as input and uses the `resolve` function to determine the next step.

The `resolve` function takes two arguments:
1. The result of calling the appropriate node function (the `[state, value]` array).
2. A `lambda` function that takes the `value` from the node function's result and returns the next `Node` to execute.

```ruby
require_relative 'ambler'

def direct(state, node)
  case node
  when Node::START
    # After 'start', always go to 'STEP'
    resolve(start(state), ->(_) { Node::STEP })
  when Node::STEP
    # After 'step', check the boolean to either loop or stop
    resolve(step(state), ->(should_continue) { should_continue ? Node::STEP : Node::STOP })
  when Node::STOP
    # 'stop' is the final node, so we return nil
    resolve(stop(state), ->(_) { nil })
  end
end
```

## 6. Start the Application

Finally, in your main execution block, call the `amble` function, passing the initial state, the starting node, and a reference to your `direct` function.

```ruby
require_relative 'ambler'

# ... (Node definitions and functions)

amble(0, Node::START, method(:direct))
```
