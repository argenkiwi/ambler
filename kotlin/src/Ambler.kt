interface Step<S, L> {
    suspend fun resolve(state: S): Pair<S, L?>
}

tailrec suspend fun <S, L> amble(state: S, lead: L, follow: (L) -> Step<S, L>): S {
    val (state, lead) = follow(lead).resolve(state)
    return when (lead) {
        null -> state
        else -> amble(state, lead, follow)
    }
}
