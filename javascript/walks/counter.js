import { ambler } from "../ambler.js";
import * as startNode from "../nodes/start.js";
import * as countNode from "../nodes/count.js";
import * as stopNode from "../nodes/stop.js";
import { fileURLToPath } from "node:url";
import process from "node:process";

/**
 * @typedef {{ count: number }} State
 * @typedef {"START" | "COUNT" | "STOP"} NodeId
 */

/** @type {ReturnType<typeof ambler<State, NodeId>>} */
const amble = ambler({
  START: () => startNode.factory({ onSuccess: "COUNT", onError: "START" }),
  COUNT: () => countNode.factory({ onCount: "COUNT", onStop: "STOP" }),
  STOP: () => stopNode.factory({ onDone: null }),
});

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  /** @type {NodeId | null} */
  let nodeId = "START";
  /** @type {State} */
  let state = {
    count: 0,
  };

  while (nodeId) {
    const next = amble(nodeId, state);
    [nodeId, state] = next instanceof Promise ? await next : next;
  }
}
