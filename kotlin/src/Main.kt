import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlin.random.Random

class Start : Step<Int> {
    override suspend fun invoke(state: Int): Pair<Int, Step<Int>?> {
        println("Starting from $state...")
        return state to Count()
    }
}

class Count() : Step<Int> {
    override suspend fun invoke(state: Int): Pair<Int, Step<Int>?> {
        val count = state + 1
        delay(1000)
        println("...$count...")
        return count to when {
            Random.nextBoolean() -> this
            else -> Stop()
        }
    }
}

class Stop() : Step<Int> {
    override suspend fun invoke(state: Int): Pair<Int, Step<Int>?> {
        println("...and stop.")
        return state to null
    }
}

fun main() {
    runBlocking { amble(0, Start()) }
}
