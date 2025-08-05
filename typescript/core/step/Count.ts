import { Lead } from "../Lead.ts";

export async function count(state: number): Promise<[number, Lead | null]> {
  console.log(`Current count: ${state}`);
  await new Promise((resolve) => setTimeout(resolve, 1000));
  state++;

  if (Math.random() < 0.5) {
    return [state, Lead.COUNT];
  } else {
    return [state, Lead.STOP];
  }
}
