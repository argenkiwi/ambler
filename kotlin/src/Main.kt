import kotlinx.coroutines.delay
import kotlin.random.Random

fun start(state: Int): Next<Int> {
    val number = promptForNumber()
    return Next(::count, number)
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
