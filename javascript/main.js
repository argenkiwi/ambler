import { Next, amble } from "./ambler.js";
import { createInterface } from 'readline';

const rl = createInterface({
  input: process.stdin,
  output: process.stdout
});

function sleep(milliseconds) {
    const start = Date.now();
    while (Date.now() - start < milliseconds) {
        // Busy-wait to block execution
    }
}

function promptForNumber(callback) {
    rl.question("Enter a starting number: ", (input) => {
        const number = parseInt(input, 10);
        if (!isNaN(number)) {
            callback(number);
        } else {
            console.log("Invalid number, please try again.");
            promptForNumber(callback);
        }
    });
}

function promptNumberNode() {
    promptForNumber((number) => {
        amble(new Next(stepNode, number));
    });
}

function stepNode(state) {
    console.log(`Count: ${state}`);
    sleep(1000);
    const newState = state + 1;
    if (Math.random() > 0.5) {
        return new Next(stepNode, newState);
    } else {
        return new Next(stopNode, newState);
    }
}

function stopNode(state) {
    console.log(`Stopping count at ${state}.`);
    rl.close();
    return null;
}

promptNumberNode();
