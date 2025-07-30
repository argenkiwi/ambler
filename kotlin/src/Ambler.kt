class Next(val run: suspend () -> Next?)

suspend fun amble(initial: Next?) {
    var next = initial
    while (next != null) {
        next = next.run()
    }
}

suspend fun amble(initial: suspend () -> Next) = amble(Next(initial))