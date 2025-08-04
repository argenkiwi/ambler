package core.step

import Step
import core.Lead
import kotlinx.coroutines.delay
import kotlin.random.Random

object Count : Step<Int, Lead> {
    override suspend fun resolve(state: Int): Pair<Int, Lead?> {
        println("Count: $state")
        delay(1000)
        val newState = state + 1
        return newState to when {
            Random.Default.nextDouble() > 0.5 -> Lead.COUNT
            else -> Lead.STOP
        }
    }
}
