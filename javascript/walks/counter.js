import { ambler } from "../ambler.js";
import startNode from "../nodes/startNode.js";
import countNode from "../nodes/countNode.js";
import stopNode from "../nodes/stopNode.js";
import { fileURLToPath } from "node:url";
import process from "node:process";

const amble = ambler((bind) => ({
  start: bind(startNode, { onSuccess: "count", onError: "start" }),
  count: bind(countNode, { onLoop: "count", onStop: "stop" }),
  stop:  bind(stopNode, { onDone: null }),
}));

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  let nodeId = "start";
  let state = {
    count: 0,
  };

  while (nodeId) {
    const next = amble(nodeId, state);
    [nodeId, state] = await next;
  }
}
