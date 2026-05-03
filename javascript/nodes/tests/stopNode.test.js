import test from "node:test";
import assert from "node:assert/strict";
import stopNode from "../stopNode.js";

test("stopNode should terminate with final count", () => {
  const initialState = { count: 10 };

  const utils = {
    print: () => {},
  };

  const result = stopNode(
    { onDone: "done" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], null);
  assert.strictEqual(result[1].count, 10);
});
