import { ambleFrom, Next } from './ambler.js';
import { createInterface } from 'readline';

const rl = createInterface({
  input: process.stdin,
  output: process.stdout
});

const promptForNumber = () => {
  return new Promise((resolve) => {
    rl.question('Enter a starting number: ', (input) => {
      const number = parseInt(input, 10);
      if (!isNaN(number)) {
        resolve(number);
      } else {
        console.log('Invalid number, please try again.');
        promptForNumber().then(resolve);
      }
    });
  });
};

const promptNumberNode = async (_state) => {
  const number = await promptForNumber();
  return new Next(() => startNode(number));
};

const startNode = (state) => {
  console.log(`Starting count from ${state}`);
  return Promise.resolve(new Next(() => stepNode(state)));
};

const stepNode = (state) => {
  const newState = state + 1;
  console.log(`Count: ${newState}`);
  if (Math.random() > 0.5) {
    return Promise.resolve(new Next(() => stepNode(newState)));
  } else {
    return Promise.resolve(new Next(() => stopNode(newState)));
  }
};

const stopNode = (_state) => {
  console.log('Stopping count.');
  rl.close();
  return Promise.resolve(null);
};

ambleFrom(() => promptNumberNode(0));