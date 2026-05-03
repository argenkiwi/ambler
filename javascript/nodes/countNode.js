/**
 * Factory for the Count node.
 *
 * @param {Object} edges Map of transitions.
 * @param {Object} [utils] Utilities.
 * @returns {function} The node function.
 */
const create = (edges, utils = defaultUtils) => {
  return async (state) => {
    utils.print(`Count: ${state.count}`);
    await utils.sleep(1000);

    const nextState = { ...state, count: state.count + 1 };

    // Randomly transition: roughly 50/50
    if (utils.random() < 0.5) {
      return [edges.onLoop, nextState];
    } else {
      return [edges.onStop, nextState];
    }
  };
};

const defaultUtils = {
  print: (msg) => console.log(msg),
  sleep: (ms) => new Promise((r) => setTimeout(r, ms)),
  random: Math.random,
};

export default create;
