import { assertEquals } from "@std/assert";
import * as StartNode from "./startNode.ts";

Deno.test("startNode should transition to onSuccess with default count when input is empty", () => {
  const state = { count: 0 };
  const utils: StartNode.Utils = {
    prompt: () => "",
    error: () => {},
  };
  const node = StartNode.create({ onSuccess: "count", onError: "start" }, utils);
  const [next, nextState] = node(state, "start") as [string, { count: number }];
  assertEquals(next, "count");
  assertEquals(nextState.count, 0);
});

Deno.test("startNode should transition to onSuccess with parsed count when input is valid", () => {
  const state = { count: 0 };
  const utils: StartNode.Utils = {
    prompt: () => "42",
    error: () => {},
  };
  const node = StartNode.create({ onSuccess: "count", onError: "start" }, utils);
  const [next, nextState] = node(state, "start") as [string, { count: number }];
  assertEquals(next, "count");
  assertEquals(nextState.count, 42);
});

Deno.test("startNode should transition to onError when input is invalid", () => {
  const state = { count: 10 };
  const utils: StartNode.Utils = {
    prompt: () => "invalid",
    error: () => {},
  };
  const node = StartNode.create({ onSuccess: "count", onError: "start" }, utils);
  const [next, nextState] = node(state, "start") as [string, { count: number }];
  assertEquals(next, "start");
  assertEquals(nextState.count, 10);
});
