import core.Lead
import core.step.count
import core.step.start
import core.step.stop

suspend fun main() {
    amble(0, Lead.START) {
        when (it) {
            Lead.START -> Next(::start)
            Lead.COUNT -> Next(::count)
            Lead.STOP -> Next(::stop)
        }
    }
}
