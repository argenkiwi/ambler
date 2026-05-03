import { assertEquals } from "@std/assert";
import countNode, { State } from "../countNode.ts";

Deno.test("countNode should transition to count when random < 0.5", async () => {
  const initialState: State = { count: 5 };

  const utils = {
    print: (_msg: string) => {},
    sleep: (_ms: number) => Promise.resolve(),
    random: () => 0.4,
  };

  const result = await countNode(
    { onLoop: "count", onStop: "stop" },
    utils,
  )(initialState) as [string, State];

  assertEquals(result[0], "count");
  assertEquals(result[1].count, 6);
});

Deno.test("countNode should transition to stop when random >= 0.5", async () => {
  const initialState: State = { count: 5 };

  const utils = {
    print: (_msg: string) => {},
    sleep: (_ms: number) => Promise.resolve(),
    random: () => 0.5,
  };

  const result = await countNode(
    { onLoop: "count", onStop: "stop" },
    utils,
  )(initialState) as [string, State];

  assertEquals(result[0], "stop");
  assertEquals(result[1].count, 6);
});
