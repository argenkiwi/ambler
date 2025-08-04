import core.Lead
import core.step.Count
import core.step.Start
import core.step.Stop

suspend fun main() {
    amble(0, Lead.START) {
        when (it) {
            Lead.START -> Start
            Lead.COUNT -> Count
            Lead.STOP -> Stop
        }
    }
}
