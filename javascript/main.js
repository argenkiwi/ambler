import { amble } from "./ambler.js";
import { Lead } from "./src/lead.js";
import { start } from "./src/step/start.js";
import { count } from "./src/step/count.js";
import { stop } from "./src/step/stop.js";
import { createInterface } from 'readline';

async function main() {
    const rl = createInterface({
        input: process.stdin,
        output: process.stdout
    });

    const initialState = 0;
    const initialLead = Lead.Start;

    await amble(initialState, initialLead, async (lead, state) => {
        switch (lead) {
            case Lead.Start:
                return await start(state, rl);
            case Lead.Count:
                return await count(state);
            case Lead.Stop:
                return stop(state);
            default:
                throw new Error(`Unknown lead: ${lead}`);
        }
    });

    rl.close();
}

main();
