package core.step

import Step
import core.Lead

object Start : Step<Int, Lead> {
    override suspend fun resolve(state: Int): Pair<Int, Lead?> {
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
}
