sealed interface Step<S> {
    suspend operator fun invoke(state: S): Pair<S, Step<S>?>
}

tailrec suspend fun <S> amble(state: S, node: Step<S>): Pair<S, Step<S>?> {
    val (state, node) = node(state)
    return when (node) {
        null -> state to null
        else -> amble(state, node)
    }
}
