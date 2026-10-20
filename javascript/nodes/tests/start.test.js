import test from "node:test";
import assert from "node:assert/strict";
import { factory } from "../start.js";

test("startNode should transition to onSuccess with 0 if input is empty", async () => {
  const initialState = { count: 0 };

  const utils = {
    readLine: (_msg) => "",
    print: (_msg) => {},
  };

  const result = await factory(
    { onSuccess: "NEXT", onError: "ERROR" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "NEXT");
  assert.strictEqual(result[1].count, 0);
});

test("startNode should transition to onSuccess with input number", async () => {
  const initialState = { count: 0 };

  const utils = {
    readLine: (_msg) => "42",
    print: (_msg) => {},
  };

  const result = await factory(
    { onSuccess: "NEXT", onError: "ERROR" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "NEXT");
  assert.strictEqual(result[1].count, 42);
});

test("startNode should transition to onError if input is invalid", async () => {
  const initialState = { count: 123 };

  const utils = {
    readLine: (_msg) => "abc",
    print: (_msg) => {},
  };

  const result = await factory(
    { onSuccess: "NEXT", onError: "ERROR" },
    utils,
  )(initialState);

  assert.strictEqual(result[0], "ERROR");
  assert.strictEqual(result[1].count, 123);
});
