class Next<S>(
    private val next: suspend (S) -> Next<S>?,
    private val state: S
) {
    suspend operator fun invoke() = next(state)
}

suspend fun amble(initial: Next<*>) {
    var next: Next<*>? = initial
    while (next != null) {
        next = next()
    }
}
