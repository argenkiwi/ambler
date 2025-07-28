export function amble(state, node, step) {
    let currentNode = node;
    let currentState = state;

    while (currentNode) {
        [currentState, currentNode] = step(currentState, currentNode);
    }

    return [currentState, null];
}
