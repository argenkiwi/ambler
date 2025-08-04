package core.step

import Step
import core.Lead

object Stop : Step<Int, Lead> {
    override suspend fun resolve(state: Int): Pair<Int, Lead?> {
        println("Stopping count at $state.")
        return state to null
    }
}
