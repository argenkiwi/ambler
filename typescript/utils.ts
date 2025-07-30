export function sleep(milliseconds: number): void {
    const start = Date.now();
    while (Date.now() - start < milliseconds) {
        // Busy-wait to block execution
    }
}

export function promptForNumber(): number {
    while (true) {
        const input = prompt("Enter a starting number:");
        if (input === null) { // Handle Ctrl+D or cancel
            console.log("Invalid input, please try again.");
            continue;
        }
        const number = parseInt(input, 10);
        if (!isNaN(number)) {
            return number;
        }
        console.log("Invalid number, please try again.");
    }
}
