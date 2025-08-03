package core.step

import core.Lead
import kotlinx.coroutines.delay
import kotlin.random.Random

suspend fun count(state: Int): Pair<Int, Lead> {
    println("Count: $state")
    delay(1000)
    val newState = state + 1
    return newState to when {
        Random.Default.nextDouble() > 0.5 -> Lead.COUNT
        else -> Lead.STOP
    }
}
