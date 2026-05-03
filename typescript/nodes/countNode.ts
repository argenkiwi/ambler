import { NodeFactory } from "../ambler.ts";

export interface State {
  count: number;
}

export type Edge = "onLoop" | "onStop";

export type Utils = {
  print: (msg: string) => void;
  sleep: (ms: number) => Promise<void>;
  random: () => number;
};

const defaultUtils: Utils = {
  print: (msg) => console.log(msg),
  sleep: (ms: number) => new Promise((r) => setTimeout(r, ms)),
  random: Math.random,
};

const create: NodeFactory<Edge, Utils, State> = (edges, utils = defaultUtils) => {
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

export default create;
