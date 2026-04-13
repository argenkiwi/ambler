import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { StoryIntroNode } from "./storyIntroNode.ts";

Deno.test("StoryIntroNode should update state with user inputs", async () => {
  const initialState: StoryIntroNode.State = {
    identity: "",
    placement: "",
    circumstances: "",
  };

  const onIntroComplete = async (state: StoryIntroNode.State) => {
    return null;
  };

  const edges = {
    onIntroComplete: onIntroComplete,
  };

  const utils: StoryIntroNode.Utils = {
    readLine: async (msg: string) => {
      if (msg === "Who is the protagonist? ") return "Arthur";
      if (msg === "Where and when does the story take place? ") return "Camelot, 5th Century";
      if (msg === "What is happening? ") return "The quest for the Holy Grail begins";
      return null;
    },
    print: () => {},
  };

  const node = StoryIntroNode.create(edges, utils);
  const result = await node(initialState);

  if (result === null) {
    throw new Error("Result should not be null");
  }

  const stateAfter = (result as any).state;
  assertEquals(stateAfter.identity, "Arthur");
  assertEquals(stateAfter.placement, "Camelot, 5th Century");
  assertEquals(stateAfter.circumstances, "The quest for the Holy Grail begins");
});

Deno.test("StoryIntroNode should return null if any input is null", async () => {
  const initialState: StoryIntroNode.State = {
    identity: "",
    placement: "",
    circumstances: "",
  };

  const edges = {
    onIntroComplete: async (state: StoryIntroNode.State) => null,
  };

  const utils: StoryIntroNode.Utils = {
    readLine: async (msg: string) => {
      if (msg === "Who is the protagonist? ") return "Arthur";
      if (msg === "Where and when does the story take place? ") return null;
      return "something";
    },
    print: () => {},
  };

  const node = StoryIntroNode.create(edges, utils);
  const result = await node(initialState);
  assertEquals(result, null);
});

Deno.test("StoryIntroNode should trim inputs", async () => {
  const initialState: StoryIntroNode.State = {
    identity: "",
    placement: "",
    circumstances: "",
  };

  const edges = {
    onIntroComplete: async (state: StoryIntroNode.State) => null,
  };

  const utils: StoryIntroNode.Utils = {
    readLine: async (msg: string) => {
      if (msg === "Who is and when does the story take place? ") return "  Arthur  "; // This won't match because of the prompt mismatch potential
      // Let's be careful with the prompts in the mock
      if (msg === "Who is the protagonist? ") return "  Arthur  ";
      if (msg === "Where and when does the story take place? ") return " Camelot ";
      if (msg === "What is happening? ") return " An event  ";
      return null;
    },
    print: () => {},
  };

  const node = StoryIntroNode.create(edges, utils);
  const result = await node(initialState);

  if (result === null) {
    throw new Error("Result should not be null");
  }

  const stateAfter = (result as any).state;
  assertEquals(stateAfter.identity, "Arthur");
  assertEquals(stateAfter.placement, "Camelot");
  assertEquals(stateAfter.circumstances, "An event");
});
