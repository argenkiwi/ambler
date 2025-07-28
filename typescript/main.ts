import { amble } from "./ambler.ts";
import { BufReader } from "https://deno.land/std@0.224.0/io/buf_reader.ts";

/**
 * Represents the different nodes or stages of the application.
 */
enum Node {
  PROMPT_NUMBER,
  START,
  STEP,
  STOP,
}

/**
 * The shared state of the application, representing the current count.
 */
type State = number;

/**
 * Prompts the user for a number and transitions to the START node.
 * @param state The current state.
 * @returns A promise that resolves to the new state and the next node.
 */
const promptNumber = async (state: State): Promise<[State, Node | null]> => {
  console.log("Enter a starting number:");
  const bufReader = new BufReader(Deno.stdin);
  const result = await bufReader.readLine();

  if (!result) {
    return [state, Node.STOP];
  }

  const line = new TextDecoder().decode(result.line);
  const number = parseInt(line, 10);

  if (!isNaN(number)) {
    return [number, Node.START];
  } else {
    console.log("Invalid number. Please try again.");
    return [state, Node.PROMPT_NUMBER];
  }
};

/**
 * Prints the starting message and transitions to the STEP node.
 * @param state The current state.
 * @returns A promise that resolves to the new state and the next node.
 */
const start = async (state: State): Promise<[State, Node | null]> => {
  console.log(`Starting count from ${state}.`);
  return [state, Node.STEP];
};

/**
 * Increments the counter, prints it, and randomly decides whether to continue or stop.
 * @param state The current state.
 * @returns A promise that resolves to the new state and the next node.
 */
const step = async (state: State): Promise<[State, Node | null]> => {
  const newState = state + 1;
  console.log(`Count: ${newState}`);
  // Randomly decide whether to continue or stop
  const nextNode = Math.random() < 0.8 ? Node.STEP : Node.STOP;
  return [newState, nextNode];
};

/**
 * Prints the final message and terminates the application.
 * @param state The current state.
 * @returns A promise that resolves to the final state and a null node.
 */
const stop = async (state: State): Promise<[State, Node | null]> => {
  console.log("Stopping count.");
  return [state, null];
};

const nodeFunctions = {
  [Node.PROMPT_NUMBER]: promptNumber,
  [Node.START]: start,
  [Node.STEP]: step,
  [Node.STOP]: stop,
};

/**
 * The main step function that executes the logic for the current node.
 * @param currentState The current state.
 * @param currentNode The current node.
 * @returns A promise that resolves to the new state and the next node.
 */
const stepFunction = (
  currentState: State,
  currentNode: Node
): Promise<[State, Node | null]> => {
  return nodeFunctions[currentNode](currentState);
};

// Start the application
amble(0, Node.PROMPT_NUMBER, stepFunction);

