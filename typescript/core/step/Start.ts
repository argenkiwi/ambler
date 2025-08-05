import { readLines } from "https://deno.land/std@0.198.0/io/mod.ts";
import { Lead } from "../Lead.ts";

export async function start(state: number): Promise<[number, Lead | null]> {
  while (true) {
    Deno.stdout.writeSync(new TextEncoder().encode("Enter a starting number (or press Enter for default 0): "));
    const reader = readLines(Deno.stdin);
    const userInput = (await reader.next()).value;

    if (!userInput) {
      console.log(`Starting count from default: ${state}`);
      return [state, Lead.COUNT];
    }

    const initialCount = parseInt(userInput);
    if (!isNaN(initialCount)) {
      state = initialCount;
      console.log(`Starting count from: ${state}`);
      return [state, Lead.COUNT];
    } else {
      console.log("Invalid input. Please enter a valid number or press Enter.");
    }
  }
}
