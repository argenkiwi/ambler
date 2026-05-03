/**
 * Creates a single-step executor for a node registry.
 * The setup callback receives a `bind` function; nodes are instantiated lazily
 * on their first execution rather than upfront.
 *
 * @param {function} setup A callback that registers nodes via `bind(factory, edges, utils?)`.
 * @returns {function} A function that executes one node step and returns a Next tuple [nextNodeId, state].
 */
export function ambler(setup) {
  /**
   * Binds a node factory to its edge map, deferring instantiation until first execution.
   *
   * @param {function} factory The node factory function.
   * @param {Object} edges Map of edge names to next node IDs.
   * @param {Object} [utils] Optional utilities for the node.
   * @returns {function} A node function.
   */
  function bind(factory, edges, utils) {
    let node;
    return (state, key) => {
      if (!node) {
        node = factory(edges, utils);
      }
      return node(state, key);
    };
  }

  const nodes = setup(bind);

  return (nodeId, state) => {
    const node = nodes[nodeId];
    if (!node) {
      throw new Error(`Node not found: ${nodeId}`);
    }
    return node(state, nodeId);
  };
}
