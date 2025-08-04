
import { Count } from "./core/step/Count.ts";
import { Lead } from "./core/Lead.ts";
import { Start } from "./core/step/Start.ts";
import { Stop } from "./core/step/Stop.ts";
import { amble, Step } from "./ambler.ts";
import { State } from "./core/State.ts";

async function main() {
  const initialState: State = { count: 0 };
  const initialLead: Lead = Lead.START;

  const follow = (lead: Lead): Step<State, Lead> => {
    switch (lead) {
      case Lead.START:
        return new Start();
      case Lead.COUNT:
        return new Count();
      case Lead.STOP:
        return new Stop();
      default:
        throw new Error(`Unknown lead: ${lead}`);
    }
  };

  await amble(initialState, initialLead, follow);
}

if (import.meta.main) {
  main();
}
