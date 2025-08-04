import { Lead } from "../lead.js";

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export async function count(state) {
    console.log(`Current count: ${state}`);
    await sleep(1000);
    state++;
    if (Math.random() > 0.5) {
        return [state, Lead.Count];
    } else {
        return [state, Lead.Stop];
    }
}