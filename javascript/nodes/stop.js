/**
 * @typedef {{ count: number }} State
 * @typedef {"onDone"} Edge
 * @typedef {{ print: (msg: string) => void }} Utils
 */

/** @type {Utils} */
const defaultUtils = {
  print: (msg) => console.log(msg),
};

/** @type {import("../ambler.js").NodeFactory<State, Edge, Utils, string>} */
export const factory = (
  edges,
  utils = defaultUtils,
) =>
(state) => {
  utils.print(`Final count: ${state.count}`);
  return [edges.onDone, state];
};
