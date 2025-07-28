export function resolve<S, A, D>(result: [S, A], direct: (a: A) => D | null): [S, D | null] {
    const [state, action] = result;
    return [state, direct(action)];
}

export async function amble<S, E>(state: S, edge: E, follow: (state: S, edge: E) => Promise<[S, E | null]>): Promise<[S, E | null]> {
    let currentEdge: E | null = edge;
    let currentState: S = state;

    while (currentEdge) {
        [currentState, currentEdge] = await follow(currentState, currentEdge);
    }

    return [currentState, null];
}
