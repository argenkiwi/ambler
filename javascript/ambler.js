export async function amble(state, lead, follow) {
    const [currentState, nextLead] = await follow(lead, state);
    if (nextLead === null) {
        return currentState;
    } else {
        return await amble(currentState, nextLead, follow);
    }
}
