package step

import kotlinx.coroutines.delay
import kotlin.random.Random

suspend fun count(state: Int): Pair<Int, CounterLead> {
    println("Count: $state")
    delay(1000)
    val newState = state + 1
    return newState to when {
        Random.Default.nextDouble() > 0.5 -> CounterLead.COUNT
        else -> CounterLead.STOP
    }
}
