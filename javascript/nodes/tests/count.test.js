import test from "node:test";
import assert from "node:assert/strict";
import { factory } from "../count.js";

test("countNode should increment count and transition to onCount if random > 0.5", async () => {
  const initialState = { count: 5 };

  const utils = {
    print: (_msg) => {},
    sleep: async (_ms) => {},
    random: () => 0.6,
  };

  const result = await factory(
    { onCount: "onCount", onStop: "onStop" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "onCount");
  assert.strictEqual(result[1].count, 6);
});

test("countNode should increment count and transition to onStop if random <= 0.5", async () => {
  const initialState = { count: 10 };

  const utils = {
    print: (_msg) => {},
    sleep: async (_ms) => {},
    random: () => 0.4,
  };

  const result = await factory(
    { onCount: "onCount", onStop: "onStop" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "onStop");
  assert.strictEqual(result[1].count, 11);
});
