import { Next, amble } from "./ambler.js";
import { createInterface } from 'readline';

const rl = createInterface({
  input: process.stdin,
  output: process.stdout
});

function sleep(milliseconds) {
    const start = Date.now();
    while (Date.now() - start < milliseconds) {}
}

function start(state) {
    return new Promise((resolve) => {
        rl.question("Enter a starting number (or press Enter for default): ", (input) => {
            if (input === "") {
                console.log("Using default starting number.");
                resolve(new Next(count, state));
            } else {
                const number = parseInt(input, 10);
                if (!isNaN(number)) {
                    resolve(new Next(count, number));
                } else {
                    console.log("Invalid number, please try again.");
                    resolve(new Next(start, state));
                }
            }
        });
    });
}

function count(state) {
    console.log(`Count: ${state}`);
    sleep(1000);
    const newState = state + 1;
    if (Math.random() > 0.5) {
        return new Next(count, newState);
    } else {
        return new Next(stop, newState);
    }
}

function stop(state) {
    console.log(`Stopping count at ${state}.`);
    rl.close();
    return null;
}

async function main() {
    let next = new Next(start, 0);
    while (next) {
        const result = await next.run();
        next = result;
    }
}

main();
