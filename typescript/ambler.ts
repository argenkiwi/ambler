export interface StepFunc<S, L> {
  (state: S): Promise<[S, L | null]>
}

export async function amble<S, L>(state: S, lead: L, follow: (lead: L) => StepFunc<S, L>): Promise<S> {
  let currentLead: L | null = lead;
  let currentState: S = state;

  while (currentLead !== null) {
    const step = follow(currentLead);
    const [newState, nextLead] = await step(currentState);
    currentState = newState;
    currentLead = nextLead;
  }

  return currentState;
}
