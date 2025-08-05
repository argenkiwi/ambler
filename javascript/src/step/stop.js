export function stop(state) {
    console.log(`Final count: ${state}`);
    return [state, null];
}