tailrec suspend fun <S, L> amble(state: S, lead: L, follow: suspend (L, S) -> Pair<S, L?>): S {
    val (state, lead) = follow(lead, state)
    return when (lead) {
        null -> state
        else -> amble(state, lead, follow)
    }
}
