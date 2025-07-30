interface Step<S> {
    suspend operator fun invoke(state: S): Pair<S, Step<S>?>
}

tailrec suspend fun <S> amble(state: S, step: Step<S>): Pair<S, Step<S>?> {
    val (state, nextStep) = step(state)
    return when (nextStep) {
        null -> state to null
        else -> amble(state, nextStep)
    }
}
