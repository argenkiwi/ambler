/**
 * Factory for the Stop node.
 *
 * @param {Object} edges Map of transitions.
 * @param {Object} [utils] Utilities.
 * @returns {function} The node function.
 */
const create = (edges, utils = defaultUtils) => {
  return (state) => {
    utils.print(`Final count: ${state.count}`);
    return [null, state];
  };
};

const defaultUtils = {
  print: (msg) => console.log(msg),
};

export default create;
