import { assertEquals } from "@std/assert";
import * as CountNode from "./countNode.ts";

Deno.test("countNode should increment count and transition to onCount when random <= 0.7", async () => {
  const state = { count: 5 };
  const utils: CountNode.Utils = {
    log: () => {},
    sleep: () => Promise.resolve(),
    random: () => 0.5, // 0.5 <= 0.7 is true, so stop = false
  };
  const node = CountNode.create({ onCount: "count", onStop: "stop" }, utils);
  const [next, nextState] = await node(state, "count");
  assertEquals(next, "count");
  assertEquals(nextState.count, 6);
});

Deno.test("countNode should increment count and transition to onStop when random > 0.7", async () => {
  const state = { count: 5 };
  const utils: CountNode.Utils = {
    log: () => {},
    sleep: () => Promise.resolve(),
    random: () => 0.8, // 0.8 > 0.7 is true, so stop = true
  };
  const node = CountNode.create({ onCount: "count", onStop: "stop" }, utils);
  const [next, nextState] = await node(state, "count");
  assertEquals(next, "stop");
  assertEquals(nextState.count, 6);
});
