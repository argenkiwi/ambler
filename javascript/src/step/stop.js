export function stop(state, readlineInterface) {
    console.log(`Final count: ${state}`);
    readlineInterface.close();
    return [state, null];
}