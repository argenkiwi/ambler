package core.step

import core.Lead

fun start(state: Int): Pair<Int, Lead?> {
    print("Enter a starting number (or press Enter for default): ")
    val input = readlnOrNull()
    return when {
        input.isNullOrEmpty() -> {
            println("Using default starting number.")
            state to Lead.COUNT
        }

        input.toIntOrNull() != null -> input.toInt() to Lead.COUNT
        else -> {
            println("Invalid number, please try again.")
            state to Lead.START
        }
    }
}

