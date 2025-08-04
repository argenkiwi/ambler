export async function amble(state, lead, follow) {
    const resolve = follow(lead);
    const [currentState, nextLead] = await resolve(state);
    if (nextLead === null) {
        return currentState;
    } else {
        return await amble(currentState, nextLead, follow);
    }
}