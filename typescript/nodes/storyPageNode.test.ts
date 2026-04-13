import { assertEquals, assertStringIncludes } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { StoryPageNode } from "./storyPageNode.ts";

Deno.test("StoryPageNode should append story page and proceed to onDecisionRequired if story continues", async () => {
  const initialState: StoryPageNode.State = {
    selectedModel: "llama3",
    ollamaHost: "http://localhost:11434",
    identity: "Arthur",
    placement: "Camelot",
    circumstances: "Quest begins",
    storyPages: ["Once upon a time."],
    currentPage: 1,
  };

  const onDecisionRequired = async (state: StoryPageNode.State) => null;
  const onPageComplete = async (state: StoryPageNode.State) => null;

  const edges = {
    onPageComplete: onPageComplete,
    onDecisionRequired: onDecisionRequired,
  };

  const chatResponse = "You walk into the forest.\n1. [ ] Go left\n2. [ ] Go right";
  
  const utils: StoryPageNode.Utils = {
    chat: async () => chatResponse,
    readLine: async () => null,
    print: () => {},
  };

  const node = StoryPageNode.create(edges, utils);
  const result = await node(initialState);

  if (result === null) {
    throw new Error("Result should not be null");
  }

  const stateAfter = (result as any).state;
  assertEquals(stateAfter.storyPages, ["Once upon a time.", chatResponse]);
  assertEquals(stateAfter.currentPage, 2);
  assertEquals((result as any).nextFunc, onDecisionRequired);
});

Deno.test("StoryPageNode should proceed to onPageComplete if story ends with 'The End'", async () => {
  const initialState: StoryPageNode.State = {
    selectedModel: "llama3",
    ollamaHost: "http://localhost:11434",
    identity: "Arthur",
    placement: "Camelot",
    circumstances: "Quest begins",
    storyPages: ["Once upon a time."],
    currentPage: 1,
  };

  const onPageComplete = async (state: StoryPageNode.State) => null;
  const onDecisionRequired = async (state: StoryPageNode.State) => null;

  const edges = {
    onPageComplete: onPageComplete,
    onDecisionRequired: onDecisionRequired,
  };

  const chatResponse = "The quest ends here. The End";
  
  const utils: StoryPageNode.Utils = {
    chat: async () => chatResponse,
    readLine: async () => null,
    print: () => {},
  };

  const node = StoryPageNode.create(edges, utils);
  const result = await node(initialState);

  if (result === null) {
    throw new Error("Result should not be null");
  }

  const stateAfter = (result as any).state;
  assertEquals(stateAfter.storyPages, ["Once upon a time.", chatResponse]);
  assertEquals(stateAfter.currentPage, 2);
  assertEquals((result as any).nextFunc, onPageComplete);
});

Deno.test("StoryPageNode should print the page content", async () => {
  const initialState: StoryPageNode.State = {
    selectedModel: "llama3",
    ollamaHost: "http://localhost:11434",
    identity: "Arthur",
    placement: "Camelot",
    circumstances: "Quest begins",
    storyPages: [],
    currentPage: 1,
  };

  const edges = {
    onPageComplete: async (state: StoryPageNode.State) => null,
    onDecisionRequired: async (state: StoryPageNode.State) => null,
  };

  const chatResponse = "New Page Content";
  let printedContent = "";
  
  const utils: StoryPageNode.Utils = {
    chat: async () => chatResponse,
    readLine: async () => null,
    print: (msg) => { printedContent = msg; },
  };

  const node = StoryPageNode.create(edges, utils);
  await node(initialState);

  assertStringIncludes(printedContent, "New Page Content");
});
