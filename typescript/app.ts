import { amble, resolve } from "./ambler.ts";

// Nodes.
enum Node {
    PROMPT_NUMBER,
    Start,
    Step,
    Stop,
}

async function promptNumber(state: number): Promise<[number, null]> {
    const buf = new Uint8Array(100);
    await Deno.stdout.write(new TextEncoder().encode("Enter a starting number: "));
    const n = <number>await Deno.stdin.read(buf);
    const text = new TextDecoder().decode(buf.subarray(0, n)).trim();
    const num = parseInt(text);
    if (isNaN(num)) {
        console.log("Invalid number. Starting with 0.");
        return [0, null];
    }
    return [num, null];
}

function start(state: number): [number, null] {
    console.log(`Let's count from ${state}...`);
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
        case Node.PROMPT_NUMBER:
            return resolve(await promptNumber(state), () => Node.Start);
        case Node.Start:
            return resolve(start(state), () => Node.Step);
        case Node.Step:
            return resolve(step(state), (shouldContinue) => (shouldContinue ? Node.Step : Node.Stop));
        case Node.Stop:
            return resolve(stop(state), () => null);
    }
}

(async () => {
    await amble(0, Node.PROMPT_NUMBER, direct);
})();
