import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { SolarPromptNode } from "./solarPromptNode.ts";

Deno.test("solarPromptNode should capture prompt successfully", async () => {
  const initialState: SolarPromptNode.State = {
    solarPrompt: "",
  };

  const promptText = "A garden in the sky.";
  let printed: string[] = [];

  const utils: SolarPromptNode.Utils = {
    readLine: async () => promptText,
    print: (msg: string) => {
      printed.push(msg);
    },
  };

  const edges = {
    onPromptComplete: async (state: SolarPromptNode.State) => {
      assertEquals(state.solarPrompt, promptText);
      return null;
    },
  };

  const result = await SolarPromptNode.create(edges, utils)(initialState);

  assertEquals(printed.includes("\n--- Solar Prompt Input ---"), true);
  
  if (result) {
    await result.run();
  } else {
    throw new Error("Expected Next object, but got null");
  }
});

Deno.test("solarPromptNode should return null if input is null", async () => {
  const initialState: SolarPromptNode.State = {
    solarPrompt: "",
  };

  const utils: SolarPromptNode.Utils = {
    readLine: async () => null,
    print: (msg: string) => {},
  };

  const edges = {
    onPromptComplete: async (state: SolarPromptNode.State) => {
      return null;
    },
  };

  const result = await SolarPromptNode.create(edges, utils)(initialState);

  assertEquals(result, null);
});
