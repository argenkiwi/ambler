
import { count } from "./core/step/Count.ts";
import { Lead } from "./core/Lead.ts";
import { start } from "./core/step/Start.ts";
import { stop } from "./core/step/Stop.ts";
import { amble, StepFunc } from "./ambler.ts";
import { State } from "./core/State.ts";

async function main() {
  const initialState: State = { count: 0 };
  const initialLead: Lead = Lead.START;

  const follow = (lead: Lead): StepFunc<State, Lead> => {
    switch (lead) {
      case Lead.START:
        return start;
      case Lead.COUNT:
        return count;
      case Lead.STOP:
        return stop;
      default:
        throw new Error(`Unknown lead: ${lead}`);
    }
  };

  await amble(initialState, initialLead, follow);
}

if (import.meta.main) {
  main();
}
