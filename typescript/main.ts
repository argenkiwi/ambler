
import { count } from "./core/step/Count.ts";
import { Lead } from "./core/Lead.ts";
import { start } from "./core/step/Start.ts";
import { stop } from "./core/step/Stop.ts";
import { amble } from "./ambler.ts";

async function main() {
  const initialState = 0;
  const initialLead: Lead = Lead.START;

  await amble<number, Lead>(initialState, initialLead, async (lead, state) => {
    switch (lead) {
      case Lead.START:
        return await start(state);
      case Lead.COUNT:
        return await count(state);
      case Lead.STOP:
        return stop(state);
      default:
        throw new Error(`Unknown lead: ${lead}`);
    }
  });
}

if (import.meta.main) {
  main();
}
