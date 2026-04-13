import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { StoryDecisionNode } from "./storyDecisionNode.ts";

Deno.test("storyDecisionNode should correctly select an option and update the page", async () => {
  const lastPage = "You arrive at the solar array.\n1. [ ] Check the panels.\n2. [ ] Survey the perimeter.";
  const initialState: StoryDecisionNode.State = {
    storyPages: [lastPage],
  };

  let printed: string[] = [];
  let input: string | null = "1";

  const utils: StoryDecisionNode.Utils = {
    readLine: async () => input,
    print: (msg: string) => {
      printed.push(msg);
    },
  };

  const edges = {
    onDecisionMade: async (state: StoryDecisionNode.State) => {
      const updatedPage = state.storyPages[0];
      assertEquals(updatedPage.includes(". [x] Check the panels."), true);
      assertEquals(updatedPage.includes(". [ ] Survey the perimeter."), true);
      return null;
    },
  };

  const result = await StoryDecisionNode.create(edges, utils)(initialState);

  if (result) {
    await result.run();
  } else {
    throw new Error("Expected Next object, but got null");
  }
});

Deno.test("storyDecisionNode should handle invalid input and retry", async () => {
  const lastPage = "Choices:\n1. [ ] Option A\n2. [ ] Option B";
  const initialState: StoryDecisionNode.State = {
    storyPages: [lastPage],
  };

  let printed: string[] = [];
  let inputCount = 0;
  const inputs: (string | null)[] = ["invalid", "3", "1"];

  const utils: StoryDecisionNode.Utils = {
    readLine: async () => {
      const val = inputs[inputCount++];
      return val;
    },
    print: (msg: string) => {
      printed.push(msg);
    },
  };

  const edges = {
    onDecisionMade: async (state: StoryDecisionNode.State) => {
      assertEquals(state.storyPages[0].includes(". [x] Option A"), true);
      return null;
    },
  };

  const result = await StoryDecisionNode.create(edges, utils)(initialState);

  assertEquals(printed.includes("Invalid selection. Please try again."), true);
  assertEquals(inputCount, 3); // "invalid", "3", "1"

  if (result) {
    await result.run();
  } else {
    throw new Error("Expected Next object, but got null");
  }
});

Deno.test("storyDecisionNode should return null if no pages exist", async () => {
  const initialState: StoryDecisionNode.State = {
    storyPages: [],
  };

  const utils: StoryDecisionNode.Utils = {
    readLine: async () => "1",
    print: () => {},
  };

  const edges = {
    onDecisionMade: async () => null,
  };

  const result = await StoryDecisionNode.create(edges, utils)(initialState);
  assertEquals(result, null);
});

Deno.test("storyDecisionNode should return null if input is null", async () => {
  const lastPage = "1. [ ] Option A";
  const initialState: StoryDecisionNode.State = {
    storyPages: [lastPage],
  };

  const utils: StoryDecisionNode.Utils = {
    readLine: async () => null,
    print: () => {},
  };

  const edges = {
    onDecisionMade: async () => null,
  };

  const result = await StoryDecisionNode.create(edges, utils)(initialState);
  assertEquals(result, null);
});
