import kotlinx.coroutines.delay
import kotlin.random.Random

fun promptForNumber(): Int {
    while (true) {
        print("Enter a starting number: ")
        val input = readlnOrNull()
        if (input != null) {
            val number = input.toIntOrNull()
            if (number != null) {
                return number
            }
        }
        println("Invalid number, please try again.")
    }
}

fun promptNumberNode(): Next? {
    val number = promptForNumber()
    return Next { startNode(number) }
}

fun startNode(state: Int): Next? {
    println("Starting count from $state")
    return Next { stepNode(state) }
}

suspend fun stepNode(state: Int): Next? {
    val newState = state + 1
    println("Count: $newState")
    delay(1000)
    return when {
        Random.nextDouble() > 0.5 -> Next { stepNode(newState) }
        else -> Next { stopNode() }
    }
}

fun stopNode(): Next? {
    println("Stopping count.")
    return null
}

suspend fun main() {
    amble { promptNumberNode() }
}
