import { Lead } from "../Lead.ts";
import { State } from "../State.ts";

export async function count(state: State): Promise<[State, Lead | null]> {
  console.log(`Current count: ${state.count}`);
  await new Promise((resolve) => setTimeout(resolve, 1000));
  state.count++;

  if (Math.random() < 0.5) {
    return [state, Lead.COUNT];
  } else {
    return [state, Lead.STOP];
  }
}
