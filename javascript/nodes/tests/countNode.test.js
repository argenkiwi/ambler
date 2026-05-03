import test from "node:test";
import assert from "node:assert/strict";
import countNode from "../countNode.js";

test("countNode should transition to count when random < 0.5", async () => {
  const initialState = { count: 5 };

  const utils = {
    print: () => {},
    sleep: () => Promise.resolve(),
    random: () => 0.4,
  };

  const result = await countNode(
    { onLoop: "count", onStop: "stop" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "count");
  assert.strictEqual(result[1].count, 6);
});

test("countNode should transition to stop when random >= 0.5", async () => {
  const initialState = { count: 5 };

  const utils = {
    print: () => {},
    sleep: () => Promise.resolve(),
    random: () => 0.5,
  };

  const result = await countNode(
    { onLoop: "count", onStop: "stop" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "stop");
  assert.strictEqual(result[1].count, 6);
});
