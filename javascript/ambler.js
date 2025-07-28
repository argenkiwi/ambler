export async function amble(state, node, step) {
    let currentNode = node;
    let currentState = state;

    while (currentNode) {
        [currentState, currentNode] = await step(currentState, currentNode);
    }

    return [currentState, null];
}
