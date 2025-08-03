package step

fun stop(state: Int): Pair<Int, CounterLead?> {
    println("Stopping count at $state.")
    return state to null
}
