interface Step<S, N> {
    suspend fun resolve(state: S): Pair<S, N?>
}

class Next<S, T>(private val delegate: suspend (S) -> Pair<S, T?>) : Step<S, T> {
    override suspend fun resolve(state: S) = delegate(state)
}

tailrec suspend fun <S, L> amble(state: S, lead: L, follow: (L) -> Step<S, L>): S {
    val (state, lead) = follow(lead).resolve(state)
    return when (lead) {
        null -> state
        else -> amble(state, lead, follow)
    }
}
