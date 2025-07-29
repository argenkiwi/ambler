import { amble, type Step } from "./ambler.ts";

// A helper function for delays, equivalent to Kotlin's `delay`
const delay = (ms: number): Promise<void> => new Promise(resolve => setTimeout(resolve, ms));

// --- State Machine Step Definitions ---
// These constant objects are equivalent to Kotlin's `data object`s.
const Start: Step<number> = {
  async run(state) {
    console.log(`Starting from ${state}...`);
    // Returns the same state and transitions to the Count step
    return [state, Count];
  },
};

const Count: Step<number> = {
  async run(state) {
    const count = state + 1;
    await delay(1000);
    console.log(`...${count}...`);
    // Randomly decide whether to continue counting or stop
    const nextStep = Math.random() < 0.5 ? Count : Stop;
    return [count, nextStep];
  },
};

const Stop: Step<number> = {
  async run(state) {
    console.log(`...and stop.`);
    // Returns the final state and `null` to end the process
    return [state, null];
  },
};

// --- Main Execution ---
if (import.meta.main) {
  await amble(0, Start);
}
