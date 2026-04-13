import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { OllamaDiscoverNode } from "./ollamaDiscoverNode.ts";

Deno.test("ollamaDiscoverNode should find host from candidate list", async () => {
  const initialState: OllamaDiscoverNode.State = {
    ollamaHost: "",
  };

  let printed: string[] = [];
  const utils: OllamaDiscoverNode.Utils = {
    tryHost: async (host: string) => host === "http://127.0.0.1:11434", // Simulate localhost found
    readLine: async () => null,
    print: (msg: string) => {
      printed.push(msg);
    },
  };

  const edges = {
    onDiscovered: async (state: OllamaDiscoverNode.State) => {
      assertEquals(state.ollamaHost, "http://127.0.0.1:11434");
      return null;
    },
  };

  const result = await OllamaDiscoverNode.create(edges, utils)(initialState);

  assertEquals(printed.includes("Searching for Ollama server..."), true);
  assertEquals(printed.includes("Found Ollama server at http://127.0.0.1:11434"), true);
  
  if (result) {
    await result.run();
  } else {
    throw new Error("Expected Next object, but got null");
  }
});

Deno.test("ollamaDiscoverNode should ask for input if no candidate host found", async () => {
  const initialState: OllamaDiscoverNode.State = {
    ollamaHost: "",
  };

  let printed: string[] = [];
  let input: string | null = "http://192.168.1.5:11434";
  const utils: OllamaDiscoverNode.Utils = {
    tryHost: async () => false,
    readLine: async () => input,
    print: (msg: string) => {
      printed.push(msg);
    },
  };

  const edges = {
    onDiscovered: async (state: OllamaDiscoverNode.State) => {
      assertEquals(state.ollamaHost, "http://192.168.1.5:11434");
      return null;
    },
  };

  const result = await OllamaDiscoverNode.create(edges, utils)(initialState);

  assertEquals(printed.includes("No Ollama server found automatically."), true);
  
  if (result) {
    await result.run();
  } else {
    throw new Error("Expected Next object, but got null");
  }
});
