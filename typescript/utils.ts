export function sleep(milliseconds: number): void {
    const start = Date.now();
    while (Date.now() - start < milliseconds) {
        // Busy-wait to block execution
    }
}
