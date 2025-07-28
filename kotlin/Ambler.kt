package org.ambler

tailrec suspend fun <S, N> amble(
    state: S,
    node: N,
    step: suspend (S, N) -> Pair<S, N?>
): Pair<S, N?> {
    val (state, node) = step(state, node)
    return when (node) {
        null -> state to null
        else -> amble(state, node, step)
    }
}
