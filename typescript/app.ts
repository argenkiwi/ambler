import { amble, resolve } from "./ambler";

// Nodes.
enum Node {
    Start,
    Step,
    Stop,
}

function start(state: number): [number, null] {
    console.log("Let's count...");
    return [state, null];
}

function step(state: number): [number, boolean] {
    const count = state + 1;
    console.log(`...${count}...`);
    return [count, Math.random() < 0.5];
}

function stop(state: number): [number, null] {
    console.log("...stop.");
    return [state, null];
}

// Flow.
async function direct(state: number, node: Node): Promise<[number, Node | null]> {
    switch (node) {
        case Node.Start:
            return resolve(start(state), () => Node.Step);
        case Node.Step:
            return resolve(step(state), (shouldContinue) => (shouldContinue ? Node.Step : Node.Stop));
        case Node.Stop:
            return resolve(stop(state), () => null);
    }
}

(async () => {
    await amble(0, Node.Start, direct);
})();
