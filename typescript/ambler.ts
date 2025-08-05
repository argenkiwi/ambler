export async function amble<S, L>(state: S, lead: L, follow: (lead: L, state: S) => Promise<[S, L | null]>): Promise<S> {
  let currentLead: L | null = lead;
  let currentState: S = state;

  while (currentLead !== null) {
    const [newState, nextLead] = await follow(currentLead, currentState);
    currentState = newState;
    currentLead = nextLead;
  }

  return currentState;
}
