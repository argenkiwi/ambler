import step.CounterLead
import step.count
import step.start
import step.stop

suspend fun main() {
    amble(0, CounterLead.START) {
        when (it) {
            CounterLead.START -> Next(::start)
            CounterLead.COUNT -> Next(::count)
            CounterLead.STOP -> Next(::stop)
        }
    }
}
