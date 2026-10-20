/**
 * @typedef {{ count: number }} State
 * @typedef {"onCount" | "onStop"} Edge
 * @typedef {{
 *   print: (msg: string) => void,
 *   sleep: (ms: number) => Promise<void>,
 *   random: () => number,
 * }} Utils
 */

/** @type {Utils} */
const defaultUtils = {
  print: (msg) => console.log(msg),
  sleep: (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
  random: () => Math.random(),
};

/** @type {import("../ambler.js").NodeFactory<State, Edge, Utils, string>} */
export const factory = (
  edges,
  utils = defaultUtils,
) =>
async (state) => {
  utils.print(`Current count: ${state.count}`);
  await utils.sleep(1000);
  const nextState = { ...state, count: state.count + 1 };

  if (utils.random() > 0.5) {
    return [edges.onCount, nextState];
  } else {
    return [edges.onStop, nextState];
  }
};
