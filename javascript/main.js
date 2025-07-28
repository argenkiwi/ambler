import { amble } from './ambler.js';
import readline from 'node:readline';

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

// Nodes.
const Node = Object.freeze({
    PROMPT_NUMBER: 0,
    START: 1,
    STEP: 2,
    STOP: 3,
});

function promptNumber(state) {
    return new Promise(resolve => {
        rl.question("Enter a starting number: ", (answer) => {
            const num = parseInt(answer);
            if (isNaN(num)) {
                console.log("Invalid number. Please try again.");
                resolve([state, Node.PROMPT_NUMBER]); // Stay in PROMPT_NUMBER
            } else {
                resolve([num, Node.START]); // Go to START with the parsed number
            }
        });
    });
}

function start(state) {
    console.log(`Let's count from ${state}...`);
    return [state, Node.STEP];
}

function step(state) {
    const count = state + 1;
    // await new Promise(resolve => setTimeout(resolve, 1000)); // Removed await
    console.log(`...${count}...`);
    return [count, Math.random() < 0.5 ? Node.STEP : Node.STOP];
}

function stop(state) {
    console.log("...stop.");
    return [state, null];
}

// Flow.
const nodeFunctions = {
    [Node.PROMPT_NUMBER]: promptNumber,
    [Node.START]: start,
    [Node.STEP]: step,
    [Node.STOP]: (state) => {
        rl.close();
        return stop(state);
    }
};

function direct(state, node) {
    return nodeFunctions[node](state);
}

(() => {
    amble(0, Node.PROMPT_NUMBER, direct);
})();
