import { NodeFactory } from "../ambler.ts";

export interface State {
  count: number;
}

export type Edge = "onSuccess" | "onError";

export type Utils = {
  print: (msg: string) => void;
  readLine: (prompt: string) => string | null;
};

const defaultUtils: Utils = {
  print: (msg) => console.log(msg),
  readLine: (_prompt: string) => "",
};

const create: NodeFactory<Edge, Utils, State> = (edges, utils = defaultUtils) => {
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

export default create;
