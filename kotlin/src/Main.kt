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

suspend fun promptNumberNode(state: Int): Next? {
    val number = promptForNumber()
    return Next { startNode(number) }
}

suspend fun startNode(state: Int): Next? {
    println("Starting count from $state")
    return Next { stepNode(state) }
}

suspend fun stepNode(state: Int): Next? {
    val newState = state + 1
    println("Count: $newState")
    return if (Random.nextDouble() > 0.5) {
        Next { stepNode(newState) }
    } else {
        Next { stopNode(newState) }
    }
}

suspend fun stopNode(state: Int): Next? {
    println("Stopping count.")
    return null
}

suspend fun main() {
    amble { promptNumberNode(0) }
}