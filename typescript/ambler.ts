/**
 * Represents the result of a step: the new state and the next step.
 * This is equivalent to Kotlin's `Pair<S, Step<S>?>`.
 */
type StepResult<S> = [S, Step<S> | null];

/**
 * An interface for a single step in the state machine.
 * This is equivalent to Kotlin's `sealed interface Step<S>`.
 */
export interface Step<S> {
  run(state: S): Promise<StepResult<S>>;
}

/**
 * Executes the state machine, stepping through nodes until a `null` node is returned.
 * This function uses a `while` loop, which is the idiomatic TypeScript/JavaScript
 * equivalent of Kotlin's `tailrec`.
 * @param initialState The starting state.
 * @param initialStep The first step to execute.
 * @returns The final state and a `null` step.
 */
export async function amble<S>(initialState: S, initialStep: Step<S>): Promise<StepResult<S>> {
  let currentState = initialState;
  let currentNode: Step<S> | null = initialStep;

  // Loop continues as long as there is a valid next step
  while (currentNode) {
    const [newState, nextNode] = await currentNode.run(currentState);
    currentState = newState;
    currentNode = nextNode;
  }

  return [currentState, null];
}
