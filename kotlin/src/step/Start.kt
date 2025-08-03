package step

fun start(state: Int): Pair<Int, CounterLead> {
    print("Enter a starting number (or press Enter for default): ")
    val input = readlnOrNull()
    return when {
        input.isNullOrEmpty() -> {
            println("Using default starting number.")
            state to CounterLead.COUNT
        }

        input.toIntOrNull() != null -> input.toInt() to CounterLead.COUNT
        else -> {
            println("Invalid number, please try again.")
            state to CounterLead.START
        }
    }
}
