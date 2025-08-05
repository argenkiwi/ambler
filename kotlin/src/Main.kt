import core.Lead
import core.step.count
import core.step.start
import core.step.stop

suspend fun main() {
    amble(0, Lead.START) { lead, state ->
        when (lead) {
            Lead.START -> start(state)
            Lead.COUNT -> count(state)
            Lead.STOP -> stop(state)
        }
    }
}
