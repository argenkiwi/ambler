import { Lead } from "../lead.js";

export function start(state, readlineInterface) {
    return new Promise((resolve) => {
        readlineInterface.question("Enter a starting number (or press Enter for default 0): ", (input) => {
            if (input === "") {
                console.log(`Starting count from default: ${state}`);
                resolve([state, Lead.Count]);
            } else {
                const number = parseInt(input, 10);
                if (!isNaN(number)) {
                    console.log(`Starting count from: ${number}`);
                    resolve([number, Lead.Count]);
                } else {
                    console.log("Invalid input. Please, try again.");
                    resolve([state, Lead.Start]);
                }
            }
        });
    });
}