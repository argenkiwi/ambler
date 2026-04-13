import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { SolarSaveNode } from "./solarSaveNode.ts";

Deno.test("solarSaveNode should save the story when user inputs 'y'", async () => {
  const initialState: SolarSaveNode.State = {
    generatedStory: "The sun shone brightly over the verdant domes.",
  };

  let printed: string[] = [];
  let savedContent: string | undefined;
  const utils: SolarSaveNode.Utils = {
    saveToFile: async (content) => {
      savedContent = content;
      return true;
    },
    readLine: async () => "y",
    print: (msg) => {
      printed.push(msg);
    },
  };

  let saveCompleteCalled = false;
  const edges: SolarSaveNode.Edges<SolarSaveNode.State> = {
    onSaveComplete: (state) => {
      saveCompleteCalled = true;
      assertEquals(state.generatedStory, initialState.generatedStory);
    },
  };

  const result = await SolarSaveNode.create(edges, utils)(initialState);

  assertEquals(savedContent, initialState.generatedStory);
  assertEquals(saveCompleteCalled, true);
  assertEquals(result, null);
});

Deno.test("solarSaveNode should not save the story when user inputs 'n'", async () => {
  const initialState: SolarSaveNode.State = {
    generatedStory: "The sun shone brightly over the verdant domes.",
  };

  let printed: string[] = [];
  let saveAttempted = false;
  const utils: SolarSaveNode.Utils = {
    saveToFile: async () => {
      saveAttempted = true;
      return true;
    },
    readLine: async () => "n",
    print: (msg: string) => {
      printed.push(msg);
    },
  };

  let saveCompleteCalled = false;
  const edges: SolarSaveNode.Edges<SolarSaveNode.State> = {
    onSaveComplete: () => {
      saveCompleteCalled = true;
    },
  };

  const result = await SolarSaveNode.create(edges, utils)(initialState);

  assertEquals(saveAttempted, false);
  assertEquals(saveCompleteCalled, true);
  assertEquals(printed.includes("Story not saved."), true);
  assertEquals(result, null);
});

Deno.test("solarSaveNode should notify user if saving fails", async () => {
  const initialState: SolarSaveNode.State = {
    generatedStory: "The sun shone brightly over the verdant domes.",
  };

  let printed: string[] = [];
  const utils: SolarSaveNode.Utils = {
    saveToFile: async () => false,
    readLine: async () => "y",
    print: (msg) => {
      printed.push(msg);
    },
  };

  const edges: SolarSaveNode.Edges<SolarSaveNode.State> = {
    onSaveComplete: () => {},
  };

  const result = await SolarSaveNode.create(edges, utils)(initialState);

  assertEquals(printed.includes("Failed to save the story."), true);
  assertEquals(result, null);
});
