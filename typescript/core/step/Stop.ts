import { State } from "../State.ts";
import { Lead } from "../Lead.ts";

export async function stop(state: State): Promise<[State, Lead | null]> {
  console.log(`Final count: ${state.count}`);
  return [state, null];
}
