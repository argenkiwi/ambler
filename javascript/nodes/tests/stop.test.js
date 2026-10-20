import test from "node:test";
import assert from "node:assert/strict";
import { factory } from "../stop.js";

test("stopNode should print final count and call onDone", async () => {
  const initialState = { count: 15 };
  let capturedMessage;

  const utils = {
    print: (msg) => {
      capturedMessage = msg;
    },
  };

  const result = await factory({ onDone: null }, utils)(initialState);

  assert.strictEqual(result[0], null);
  assert.strictEqual(result[1].count, 15);
  assert.strictEqual(capturedMessage, "Final count: 15");
});
