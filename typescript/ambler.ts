
export async function amble<S, N>(
  state: S,
  node: N,
  step: (currentState: S, currentNode: N) => Promise<[S, N | null]>
): Promise<[S, N | null]> {
  let currentState: S = state;
  let currentNode: N | null = node;

  while (currentNode !== null) {
    const [newState, nextNode] = await step(currentState, currentNode);
    currentState = newState;
    currentNode = nextNode;
  }

  return [currentState, null];
}
