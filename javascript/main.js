import { amble } from "./ambler.js";
import { Lead } from "./src/lead.js";
import { start } from "./src/step/start.js";
import { count } from "./src/step/count.js";
import { stop } from "./src/step/stop.js";
import { createInterface } from 'readline';

const rl = createInterface({
    input: process.stdin,
    output: process.stdout
});

async function main() {
    const initialState = 0;
    const initialLead = Lead.Start;

    const follow = (lead) => {
        switch (lead) {
            case Lead.Start:
                return (state) => start(state, rl);
            case Lead.Count:
                return count;
            case Lead.Stop:
                return (state) => stop(state, rl);
            default:
                throw new Error(`Unknown lead: ${lead}`);
        }
    };

    await amble(initialState, initialLead, follow);
}

main();