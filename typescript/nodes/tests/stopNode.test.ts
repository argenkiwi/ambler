import { assertEquals } from "@std/assert";
import stopNode, { State } from "../stopNode.ts";

Deno.test("stopNode should terminate with final count", () => {
  const initialState: State = { count: 10 };

  const utils = {
    print: (_msg: string) => {},
  };

  const result = stopNode(
    { onDone: "done" },
    utils,
  )(initialState) as [null, State];

  assertEquals(result[0], null);
  assertEquals(result[1].count, 10);
});
