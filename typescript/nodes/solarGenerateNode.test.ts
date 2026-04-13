import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { SolarGenerateNode } from "./solarGenerateNode.ts";

Deno.test("solarGenerateNode should generate a story successfully", async () => {
  const initialState: SolarGenerateNode.State = {
    ollamaHost: "http://localhost:11434",
    selectedModel: "llama3:latest",
    solarPrompt: "A community rebuilding a solar array.",
    generatedStory: "",
  };

  const mockStory = "Once upon a time in a solarpunk future...";
  let printed: string[] = [];

  const utils: SolarGenerateNode.Utils = {
    generateStory: async () => mockStory,
    print: (msg: string) => {
      printed.push(msg);
    },
  };

  const edges = {
    onGenerateComplete: async (state: SolarGenerateNode.State) => {
      assertEquals(state.generatedStory, mockStory);
      return null;
    },
  };

  const result = await SolarGenerateNode.create(edges, utils)(initialState);

  assertEquals(printed.includes("\nGenerating your solarpunk story... (this may take a moment)"), true);
  assertEquals(printed.includes("\n--- GENERATED STORY ---"), true);
  assertEquals(printed.includes(mockStory), true);
  assertEquals(printed.includes("\n--- END OF STORY ---"), true);

  if (result) {
    await result.run();
  } else {
    throw new Error("Expected Next object, but got null");
  }
});

Deno.test("solarGenerateNode should handle generation error", async () => {
  const initialState: SolarGenerateNode.State = {
    ollamaHost: "http://localhost:11434",
    selectedModel: "llama3:latest",
    solarPrompt: "A community rebuilding a solar array.",
    generatedStory: "",
  };

  const errorMsg = "Connection refused";
  let printed: string[] = [];

  const utils: SolarGenerateNode.Utils = {
    generateStory: async () => {
      throw new Error(errorMsg);
    },
    print: (msg: string) => {
      printed.push(msg);
    },
  };

  const edges = {
    onGenerateComplete: async (state: SolarGenerateNode.State) => {
      return null;
    },
  };

  const result = await SolarGenerateNode.create(edges, utils)(initialState);

  assertEquals(result, null);
  assertEquals(printed.some(msg => msg.includes(`Error generating story: Error: ${errorMsg}`)), true);
});
