import kotlinx.coroutines.delay
import kotlin.random.Random

fun start(state: Int): Next<Int> {
    print("Enter a starting number (or press Enter for default): ")
    val input = readlnOrNull()

    return when {
        input.isNullOrEmpty() -> {
            println("Using default starting number.")
            Next(::count, state)
        }
        input.toIntOrNull() != null -> {
            Next(::count, input.toInt())
        }
        else -> {
            println("Invalid number, please try again.")
            Next(::start, state)
        }
    }
}

suspend fun count(state: Int): Next<Int>? {
    println("Count: $state")
    delay(1000)
    val newState = state + 1
    return when {
        Random.nextDouble() > 0.5 -> Next(::count, newState)
        else -> Next(::stop, newState)
    }
}

fun stop(state: Int): Next<Int>? {
    println("Stopping count at $state.")
    return null
}

suspend fun main() {
    amble(::start, 0)
}
