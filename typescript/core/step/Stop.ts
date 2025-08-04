import { Step } from "../../ambler.ts";
import { State } from "../State.ts";
import { Lead } from "../Lead.ts";

export class Stop implements Step<State, Lead> {
  async resolve(state: State): Promise<[State, Lead | null]> {
    console.log(`Final count: ${state.count}`);
    return [state, null];
  }
}
