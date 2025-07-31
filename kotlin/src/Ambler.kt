class Next<S>(
    private val next: suspend (S) -> Next<S>?,
    private val state: S
) {
    suspend operator fun invoke() = next(state)
}

suspend fun <S> amble(next: (S) -> Next<S>?, state: S) {
    var next = next(state)
    while (next != null) {
        next = next()
    }
}
