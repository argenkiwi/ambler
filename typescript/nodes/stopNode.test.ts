import { assertEquals } from "@std/assert";
import * as StopNode from "./stopNode.ts";

Deno.test("stopNode should transition to onDone (null)", () => {
  const state = { count: 10 };
  const utils: StopNode.Utils = {
    log: () => {},
  };
  const node = StopNode.create({ onDone: null }, utils);
  const [next, nextState] = node(state, "stop") as [null, { count: number }];
  assertEquals(next, null);
  assertEquals(nextState.count, 10);
});
