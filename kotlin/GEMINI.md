# Ambler Application Guidelines

When building applications using `Ambler.kt`, please adhere to the following guidelines to ensure consistency and maintainability.

> IMPORTANT: Never modify the code in `Ambler.kt`!

## 1. Identify Program Stages (Nodes)

First, break down your program's logic into distinct steps or stages. These will become the "nodes" of your application's flow.

For example, a simple counter application might have the following stages:
- `START`: Initializes the process.
- `STEP`: Performs the counting action.
- `STOP`: Terminates the process.

## 2. Define Nodes

Create an `enum class`, typically named `Node`, to define these stages.

```kotlin
enum class Node {
    START, COUNT, STOP
}
```

## 3. Determine Shared State

Decide what data needs to be passed between the different nodes. This will be your "state" object. It can be a simple type (like an `Int` for a counter) or a more complex data structure (like a `data class` or a `Map`).

In the counter example, the state is an `Int` representing the current count.

## 4. Create Node Functions

For each node, create a corresponding function that takes the current `state` as a parameter. Each function should return a `Pair` containing:
1. The (potentially modified) `state`.
2. An optiona `Node` value that will determine which node to go to next. 

```kotlin
import kotlinx.coroutines.delay
import kotlin.random.Random

enum class Node {
    START, COUNT, STOP
}

fun start(state: Int): Pair<Int, Node> {
    println("Starting from $state...")
    return state to Node.COUNT
}

suspend fun count(state: Int) : Pair<Int, Node>{
    val count = state + 1
    delay(1000)
    println("...$count...")
    return count to when {
        Random.nextBoolean() -> Node.COUNT
        else -> Node.STOP
    }
}

fun stop(state: Int): Pair<Int, Node?> {
    println("...and stop.")
    return state to null
}
```

## 5. Start the Application
 
Finally, in your main execution block, call the `amble` function, passing the initial state, the starting node, and a `step` function. This function acts as the central router for your application. It takes the current `state` and `Node` as input, calls the appropriate node function and returns the updated state and the next `Node` to be called.

```kotlin
import kotlinx.coroutines.runBlocking

fun main() {
    runBlocking {
        amble(0, Node.START) { state, node ->
            when (node) {
                Node.START -> start(state)
                Node.COUNT -> count(state)
                Node.STOP -> stop(state)
            }
        }
    }
}
```
