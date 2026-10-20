/**
 * Factory for the Start node.
 *
 * @param {Object} edges Map of transitions.
 * @param {Object} [utils] Utilities.
 * @returns {function} The node function.
 */
const create = (edges, utils = defaultUtils) => {
  return (state) => {
    const input = utils.readLine("Enter starting number: ");
    if (input === null) {
      utils.print("No input received. Using default count of 0.");
      return [edges.onSuccess, { ...state, count: 0 }];
    }
    const trimmed = input.trim();

    if (trimmed === "") {
      utils.print("Using default count of 0.");
      return [edges.onSuccess, { ...state, count: 0 }];
    }

    const num = Number(trimmed);

    if (Number.isNaN(num)) {
      utils.print("Invalid input. Please enter a number.");
      return [edges.onError, { ...state }];
    }

    utils.print(`Starting count: ${num}`);
    return [edges.onSuccess, { ...state, count: num }];
  };
};

const defaultUtils = {
  print: (msg) => console.log(msg),
  readLine: (msg) => prompt(msg),
};

export default create;
