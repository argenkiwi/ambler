import { amble } from './ambler.js';
import  readline from 'node:readline';

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

// Nodes.
const Node = {
    PROMPT_NUMBER: 0,
    START: 1,
    STEP: 2,
    STOP: 3,
};

async function promptNumber(state) {
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

async function step(state) {
    const count = state + 1;
    await new Promise(resolve => setTimeout(resolve, 1000));
    console.log(`...${count}...`);
    return [count, Math.random() < 0.5 ? Node.STEP : Node.STOP];
}

function stop(state) {
    console.log("...stop.");
    return [state, null];
}

// Flow.
async function direct(state, node) {
    if (node === Node.PROMPT_NUMBER) {
        return await promptNumber(state);
    } else if (node === Node.START) {
        return start(state);
    } else if (node === Node.STEP) {
        return await step(state);
    } else if (node === Node.STOP) {
        rl.close();
        return stop(state);
    }
}

(async () => {
    await amble(0, Node.PROMPT_NUMBER, direct);
})();
