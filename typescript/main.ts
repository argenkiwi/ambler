import { Next, amble } from "./ambler.ts";
import { sleep, promptForNumber} from "./utils.ts"

function promptNumberNode(): Next<number> {
    const number = promptForNumber();
    return new Next(stepNode, number);
}

function stepNode(state: number): Next<any> | null {
    console.log(`Count: ${state}`);
    sleep(1000);
    const newState = state + 1;
    if (Math.random() > 0.5) {
        return new Next(stepNode, newState);
    } else {
        return new Next(stopNode, newState);
    }
}

function stopNode(state: number): Next<any> | null {
    console.log(`Stopping count at ${state}.`);
    return null;
}

if (import.meta.main) {
    amble(promptNumberNode());
}
