import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { StorySaveNode } from "./storySaveNode.ts";

Deno.test("StorySaveNode should save the story and proceed to onSaveComplete", async () => {
  const initialState: StorySaveNode.State = {
    storyPages: ["Page 1", "Page 2"],
  };

  const onSaveComplete = async (state: StorySaveNode.State) => null;

  const edges = {
    onSaveComplete: onSaveComplete,
  };

  let capturedFilename = "";
  let capturedContent = "";
  let printedMsg = "";

  const utilsMock: StorySaveNode.Utils = {
    saveFile: async (filename, content) => {
      capturedFilename = filename;
      capturedContent = content;
    },
    print: (msg: string) => {
      printedMsg = msg;
    },
  };

  const node = StorySaveNode.create(edges, utilsMock);
  const result = await node(initialState);

  if (result === null) {
    throw new Error("Result should not be null");
  }

  assertEquals(capturedContent, "Page 1\n\nPage 2");
  assertEquals(capturedFilename.startsWith("story-"), true);
  assertEquals(printedMsg.includes("Story saved to"), true);
  assertEquals((result as any).nextFunc, onSaveComplete);
});

Deno.test("StorySaveNode should handle errors during saving", async () => {
  const initialState: StorySaveNode.State = {
    storyPages: ["Page 1"],
  };

  const onSaveComplete = async (state: StorySaveNode.State) => null;

  const edges = {
    onSaveComplete: onSaveComplete,
  };

  let printedMsg = "";
  const utils: StorySaveNode.Utils = {
    saveFile: async () => {
      throw new Error("Disk full");
    },
    print: (msg: string) => {
      printedMsg = msg;
    },
  };

  const node = StorySaveNode.create(edges, utils);
  const result = await node(initialState);

  if (result === null) {
    throw new Error("Result should not be null");
  }

  assertEquals(printedMsg.includes("Error saving story: Error: Disk full"), true);
  assertEquals((result as any).nextFunc, onSaveComplete);
});
