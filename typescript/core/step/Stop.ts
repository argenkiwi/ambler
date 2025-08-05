import { Lead } from "../Lead.ts";

export async function stop(state: number): Promise<[number, Lead | null]> {
  console.log(`Final count: ${state}`);
  return [state, null];
}
