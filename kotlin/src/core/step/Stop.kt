package core.step

import core.Lead

fun stop(state: Int): Pair<Int, Lead?> {
    println("Stopping count at $state.")
    return state to null
}

