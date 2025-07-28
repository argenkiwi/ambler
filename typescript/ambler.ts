/**
 * Asynchronously traverses a node-based structure, updating a state value at each step.
 * This is the TypeScript equivalent of the provided Kotlin `amble` function.
 *
 * @param state The initial state.
 * @param node The starting node.
 * @param step An async function that takes the current state and node,
 * and returns a promise resolving to a tuple with the new state
 * and the next node (or null to terminate).
 * @returns A promise that resolves to a tuple containing the final state and a null node.
 */
async function amble<S, N>(
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
