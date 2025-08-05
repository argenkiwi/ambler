import { Lead } from "../Lead.ts";

export function stop(state: number): [number, Lead | null] {
  console.log(`Final count: ${state}`);
  return [state, null];
}
