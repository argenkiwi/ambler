import fs from "node:fs";
import process from "node:process";

/**
 * @typedef {{ count: number }} State
 * @typedef {"onSuccess" | "onError"} Edge
 * @typedef {{
 *   readLine: (msg: string) => string | null,
 *   print: (msg: string) => void,
 * }} Utils
 */

/**
 * Prints `msg` and synchronously reads one line from stdin.
 * Returns `null` if stdin is closed before any input is read.
 *
 * @param {string} msg
 * @returns {string | null}
 */
const readStdinLine = (msg) => {
  process.stdout.write(msg);
  const bytes = [];
  const buf = Buffer.alloc(1);
  while (true) {
    let n;
    try {
      n = fs.readSync(0, buf, 0, 1, null);
    } catch (err) {
      if (err.code === "EAGAIN") continue;
      if (err.code === "EOF") n = 0;
      else throw err;
    }
    if (n === 0) {
      return bytes.length ? Buffer.from(bytes).toString() : null;
    }
    if (buf[0] === 0x0a) return Buffer.from(bytes).toString().replace(/\r$/, "");
    bytes.push(buf[0]);
  }
};

/** @type {Utils} */
const defaultUtils = {
  readLine: readStdinLine,
  print: (msg) => console.log(msg),
};

/** @type {import("../ambler.js").NodeFactory<State, Edge, Utils, string>} */
export const factory = (
  edges,
  utils = defaultUtils,
) =>
(state) => {
  const input = utils.readLine("Enter a starting number: ");

  if (input === null || input === "") {
    return [edges.onSuccess, { ...state, count: 0 }];
  }

  const n = parseInt(input);
  if (isNaN(n)) {
    utils.print("Error: Invalid input. Please enter a number.");
    return [edges.onError, state];
  }

  return [edges.onSuccess, { ...state, count: n }];
};
