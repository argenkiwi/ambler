/**
 * The result of a node's execution: the next node ID (or `null` to stop) and the updated state.
 *
 * @template S The machine's state type.
 * @template {string} K The union of valid node identifier strings.
 * @typedef {[key: K | null, state: S]} Next
 */

/**
 * A processing step in the state machine.
 *
 * Receives the current state and returns a {@link Next} tuple (or a Promise resolving to one).
 *
 * @template S The machine's state type.
 * @template {string} K The union of valid node identifier strings.
 * @typedef {(state: S) => Next<S, K> | Promise<Next<S, K>>} Node
 */

/**
 * A factory function that creates a {@link Node}.
 *
 * It decouples the node's internal logic from the specific node IDs (edges) and
 * external dependencies (utils) used in a specific walk.
 *
 * @template S The state type required by this node.
 * @template {string} E The union of valid edge names for this node.
 * @template U The type of the utilities object provided to the node.
 * @template {string} N The union of valid node identifier strings in the walk.
 * @typedef {(edges: Record<E, N | null>, utils?: U) => Node<S, N>} NodeFactory
 */

/**
 * Creates a state machine runner from a map of node factories.
 *
 * Each factory is called once on first use; the resulting node is cached for subsequent calls.
 *
 * @template S The machine's state type.
 * @template {string} K The union of valid node identifier strings.
 * @param {Record<K, () => Node<S, K>>} nodes A map of node IDs to factory functions that produce {@link Node}s.
 * @returns {(nodeId: K, state: S) => Next<S, K> | Promise<Next<S, K>>} A runner `(nodeId, state) => Next`.
 * @throws {Error} If `nodeId` is not present in `nodes`.
 */
export function ambler(nodes) {
  /** @type {Partial<Record<K, Node<S, K>>>} */
  const loaded = {};
  return (nodeId, state) => {
    if (!(nodeId in nodes)) throw new Error(`Node not found: ${nodeId}`);
    loaded[nodeId] ??= nodes[nodeId]();
    return loaded[nodeId](state);
  };
}
