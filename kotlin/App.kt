package org.ambler

import kotlinx.coroutines.runBlocking
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
