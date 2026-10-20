import test from "node:test";
import assert from "node:assert/strict";
import startNode from "../startNode.js";

test("startNode should use default count of 0 when input is empty", () => {
  const initialState = { count: 0 };

  const utils = {
    print: () => {},
    readLine: () => "",
  };

  const result = startNode(
    { onSuccess: "count", onError: "start" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "count");
  assert.strictEqual(result[1].count, 0);
});

test("startNode should use valid number when input is provided", () => {
  const initialState = { count: 0 };

  const utils = {
    print: () => {},
    readLine: () => "42",
  };

  const result = startNode(
    { onSuccess: "count", onError: "start" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "count");
  assert.strictEqual(result[1].count, 42);
});

test("startNode should loop back to start when input is invalid", () => {
  const initialState = { count: 0 };

  const utils = {
    print: () => {},
    readLine: () => "abc",
  };

  const result = startNode(
    { onSuccess: "count", onError: "start" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "start");
  assert.strictEqual(result[1].count, 0);
});
