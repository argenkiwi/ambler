import { Next, amble } from "./ambler.ts";
import { sleep } from "./utils.ts";

function start(state: number): Next<number> {
    const input = prompt("Enter a starting number (or press Enter for default):");

    if (input === null || input === "") {
        console.log("Using default starting number.");
        return new Next(count, state);
    } else {
        const number = parseInt(input, 10);
        if (!isNaN(number)) {
            return new Next(count, number);
        } else {
            console.log("Invalid number, please try again.");
            return new Next(start, state);
        }
    }
}

function count(state: number): Next<any> | null {
    console.log(`Count: ${state}`);
    sleep(1000);
    const newState = state + 1;
    if (Math.random() > 0.5) {
        return new Next(count, newState);
    } else {
        return new Next(stop, newState);
    }
}

function stop(state: number): Next<any> | null {
    console.log(`Stopping count at ${state}.`);
    return null;
}

if (import.meta.main) {
    amble(start, 0);
}
