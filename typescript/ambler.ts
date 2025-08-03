export interface Step<S, L> {
  resolve(state: S): Promise<[S, L | null]>;
}

export class Next<S, L> implements Step<S, L> {
  private delegate: (state: S) => Promise<[S, L | null]>;

  constructor(delegate: (state: S) => Promise<[S, L | null]>) {
    this.delegate = delegate;
  }

  async resolve(state: S): Promise<[S, L | null]> {
    return this.delegate(state);
  }
}

export async function amble<S, L>(state: S, lead: L, follow: (lead: L) => Step<S, L>): Promise<S> {
  let currentLead: L | null = lead;
  let currentState: S = state;

  while (currentLead !== null) {
    const [newState, nextLead] = await follow(currentLead).resolve(currentState);
    currentState = newState;
    currentLead = nextLead;
  }

  return currentState;
}