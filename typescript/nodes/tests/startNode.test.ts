import { assertEquals } from "@std/assert";
import startNode, { State } from "../startNode.ts";

type ResultType = ReturnType<ReturnType<typeof startNode>>;

Deno.test("startNode should use default count of 0 when input is empty", () => {
  const initialState: State = { count: 0 };

  const utils = {
    print: (_msg: string) => {},
    readLine: (_prompt: string) => "",
  };

  const result = startNode(
    { onSuccess: "count", onError: "start" },
    utils,
  )(initialState) as [string, State];

  assertEquals(result[0], "count");
  assertEquals(result[1].count, 0);
});

Deno.test("startNode should use valid number when input is provided", () => {
  const initialState: State = { count: 0 };

  const utils = {
    print: (_msg: string) => {},
    readLine: (_prompt: string) => "42",
  };

  const result = startNode(
    { onSuccess: "count", onError: "start" },
    utils,
  )(initialState) as [string, State];

  assertEquals(result[0], "count");
  assertEquals(result[1].count, 42);
});

Deno.test("startNode should loop back to start when input is invalid", () => {
  const initialState: State = { count: 0 };

  const utils = {
    print: (_msg: string) => {},
    readLine: (_prompt: string) => "abc",
  };

  const result = startNode(
    { onSuccess: "count", onError: "start" },
    utils,
  )(initialState) as [string, State];

  assertEquals(result[0], "start");
  assertEquals(result[1].count, 0);
});
