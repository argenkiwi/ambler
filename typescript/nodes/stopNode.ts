import { Edges, Node } from "../ambler.ts";

export interface Utils {
  log: (message: string) => void;
}

export type Edge = "onDone";

export function create<S extends { count: number }, K extends string>(
  edges: Edges<Edge, K>,
  utils: Utils = { log: console.log },
): Node<S, K> {
  return (state) => {
    utils.log(`Final count: ${state.count}`);
    utils.log("Counting process terminated.");
    return [edges.onDone, state];
  };
}
