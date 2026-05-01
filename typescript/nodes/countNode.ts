import { Edges, Node } from "../ambler.ts";

export interface Utils {
  log: (message: string) => void;
  sleep: (ms: number) => Promise<void>;
  random: () => number;
}

export type Edge = "onCount" | "onStop";

export function create<S extends { count: number }, K extends string>(
  edges: Edges<Edge, K>,
  utils: Utils = {
    log: console.log,
    sleep: (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
    random: Math.random,
  },
): Node<S, K> {
  return async (state) => {
    utils.log(`Current count: ${state.count}`);

    await utils.sleep(1000);

    const nextState = { ...state, count: state.count + 1 };

    const stop = utils.random() > 0.7;

    if (stop) {
      return [edges.onStop, nextState];
    }

    return [edges.onCount, nextState];
  };
}
