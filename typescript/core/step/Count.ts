import { Step } from "../../ambler.ts";
import { Lead } from "../Lead.ts";
import { State } from "../State.ts";


// 4. Implement the Count step.
export class Count implements Step<State, Lead> {
  async resolve(state: State): Promise<[State, Lead | null]> {
    console.log(`Current count: ${state.count}`);
    await new Promise((resolve) => setTimeout(resolve, 1000));
    state.count++;

    if (Math.random() < 0.5) {
      return [state, Lead.COUNT];
    } else {
      return [state, Lead.STOP];
    }
  }
}
