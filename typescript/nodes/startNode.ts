import { Edges, Node } from "../ambler.ts";

export interface Utils {
  prompt: (message: string) => string | null;
  error: (message: string) => void;
}

export type Edge = "onSuccess" | "onError";

export function create<S extends { count: number }, K extends string>(
  edges: Edges<Edge, K>,
  utils: Utils = { prompt: globalThis.prompt.bind(globalThis), error: console.error },
): Node<S, K> {
  return (state) => {
    const input = utils.prompt("Enter a starting number (default 0):");

    if (input === null || input.trim() === "") {
      return [edges.onSuccess, { ...state, count: 0 }];
    }

    const count = parseInt(input, 10);
    if (isNaN(count)) {
      utils.error("Invalid number entered.");
      return [edges.onError, state];
    }

    return [edges.onSuccess, { ...state, count }];
  };
}
