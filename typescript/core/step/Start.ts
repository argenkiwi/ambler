import { readLines } from "https://deno.land/std@0.198.0/io/mod.ts";
import { Step } from "../../ambler.ts";
import { State } from "../State.ts";
import { Lead } from "../Lead.ts";

export class Start implements Step<State, Lead> {
  async resolve(state: State): Promise<[State, Lead | null]> {
    while (true) {
      Deno.stdout.writeSync(new TextEncoder().encode("Enter a starting number (or press Enter for default 0): "));
      const reader = readLines(Deno.stdin);
      const userInput = (await reader.next()).value;

      if (!userInput) {
        state.count = 0;
        console.log(`Starting count from default: ${state.count}`);
        return [state, Lead.COUNT];
      }

      const initialCount = parseInt(userInput);
      if (!isNaN(initialCount)) {
        state.count = initialCount;
        console.log(`Starting count from: ${state.count}`);
        return [state, Lead.COUNT];
      } else {
        console.log("Invalid input. Please enter a valid number or press Enter.");
      }
    }
  }
}
