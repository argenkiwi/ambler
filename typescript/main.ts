import { ambleFrom, Next } from "./ambler.ts";

const promptForNumber = (): Promise<number> => {
    return new Promise((resolve) => {
        const ask = () => {
            const input = prompt("Enter a starting number:");
            if (input === null) {
                resolve(0);
                return;
            }
            const number = parseInt(input, 10);
            if (!isNaN(number)) {
                resolve(number);
            } else {
                console.log("Invalid number, please try again.");
                ask();
            }
        };
        ask();
    });
};

const promptNumberNode = (state: number): Promise<Next | null> => {
    return new Promise(async (resolve) => {
        const number = await promptForNumber();
        resolve(new Next(() => startNode(number)));
    });
};

const startNode = (state: number): Promise<Next | null> => {
    console.log(`Starting count from ${state}`);
    return Promise.resolve(new Next(() => stepNode(state)));
};

const stepNode = (state: number): Promise<Next | null> => {
    const newState = state + 1;
    console.log(`Count: ${newState}`);
    if (Math.random() > 0.5) {
        return Promise.resolve(new Next(() => stepNode(newState)));
    } else {
        return Promise.resolve(new Next(() => stopNode(newState)));
    }
};

const stopNode = (state: number): Promise<Next | null> => {
    console.log("Stopping count.");
    return Promise.resolve(null);
};

if (import.meta.main) {
    ambleFrom(() => promptNumberNode(0));
}
