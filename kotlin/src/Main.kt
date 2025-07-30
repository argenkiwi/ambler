import kotlinx.coroutines.delay
import kotlin.random.Random

fun promptNumberNode(): Next<Int> {
    val number = promptForNumber()
    return Next(::stepNode, number)
}

suspend fun stepNode(state: Int): Next<Int>? {
    println("Count: $state")
    delay(1000)
    val newState = state + 1
    return when {
        Random.nextDouble() > 0.5 -> Next(::stepNode, newState)
        else -> Next(::stopNode, newState)
    }
}

fun stopNode(state: Int): Next<Int>? {
    println("Stopping count at $state.")
    return null
}

suspend fun main() {
    amble(promptNumberNode())
}
