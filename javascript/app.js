const { amble } = require('./ambler');

// Nodes.
const Node = {
    START: 1,
    STEP: 2,
    STOP: 3,
};

function start(state) {
    console.log("Let's count...");
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
    if (node === Node.START) {
        return start(state);
    } else if (node === Node.STEP) {
        return await step(state);
    } else if (node === Node.STOP) {
        return stop(state);
    }
}

(async () => {
    await amble(0, Node.START, direct);
})();
