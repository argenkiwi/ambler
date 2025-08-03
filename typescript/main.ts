import { Step, amble } from "./ambler.ts";
import { readLines } from "https://deno.land/std@0.198.0/io/mod.ts";

// 1. Define the CounterState interface.
interface CounterState {
  count: number;
}

// 2. Define the CounterLead enum.
enum CounterLead {
  START,
  COUNT,
  STOP,
}

// 3. Implement the Start step.
class Start implements Step<CounterState, CounterLead> {
  async resolve(state: CounterState): Promise<[CounterState, CounterLead | null]> {
    while (true) {
      Deno.stdout.writeSync(new TextEncoder().encode("Enter a starting number (or press Enter for default 0): "));
      const reader = readLines(Deno.stdin);
      const userInput = (await reader.next()).value;

      if (!userInput) {
        state.count = 0;
        console.log(`Starting count from default: ${state.count}`);
        return [state, CounterLead.COUNT];
      }

      const initialCount = parseInt(userInput);
      if (!isNaN(initialCount)) {
        state.count = initialCount;
        console.log(`Starting count from: ${state.count}`);
        return [state, CounterLead.COUNT];
      } else {
        console.log("Invalid input. Please enter a valid number or press Enter.");
      }
    }
  }
}

// 4. Implement the Count step.
class Count implements Step<CounterState, CounterLead> {
  async resolve(state: CounterState): Promise<[CounterState, CounterLead | null]> {
    console.log(`Current count: ${state.count}`);
    await new Promise((resolve) => setTimeout(resolve, 1000));
    state.count++;

    if (Math.random() < 0.5) {
      return [state, CounterLead.COUNT];
    } else {
      return [state, CounterLead.STOP];
    }
  }
}

// 5. Implement the Stop step.
class Stop implements Step<CounterState, CounterLead> {
  async resolve(state: CounterState): Promise<[CounterState, CounterLead | null]> {
    console.log(`Final count: ${state.count}`);
    return [state, null];
  }
}

// 6. Create the main function to run the amble process.
async function main() {
  const initialState: CounterState = { count: 0 };
  const initialLead: CounterLead = CounterLead.START;

  const follow = (lead: CounterLead): Step<CounterState, CounterLead> => {
    switch (lead) {
      case CounterLead.START:
        return new Start();
      case CounterLead.COUNT:
        return new Count();
      case CounterLead.STOP:
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