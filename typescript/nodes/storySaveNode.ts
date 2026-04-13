import fs from "node:fs/promises";
import { Next, Nextable } from "../ambler.ts";

export namespace StorySaveNode {
  export interface State {
    storyPages: string[];
  }

  export type Edges<S extends State> = {
    onSaveComplete: Nextable<S>;
  };

  export type Utils = {
    saveFile: (filename: string, content: string) => Promise<void>;
    print: (msg: string) => void;
  };

  const defaultUtils: Utils = {
    saveFile: async (filename, content) => {
      const dir = "cyoa";
      await fs.mkdir(dir, { recursive: true });
      await fs.writeFile(`${dir}/${filename}.md`, content);
    },
    print: (msg: string) => console.log(msg),
  };

  export function create<S extends State>(
    edges: Edges<S>,
    utils: Utils = defaultUtils,
  ): Nextable<S> {
    return async (state: S): Promise<Next<S> | null> => {
      const fullStory = state.storyPages.join("\n\n");
      const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
      const filename = `story-${timestamp}`;

      try {
        await utils.saveFile(filename, fullStory);
        utils.print(`Story saved to cyoa/${filename}.md`);
      } catch (err) {
        utils.print(`Error saving story: ${err}`);
      }

      return new Next(edges.onSaveComplete, state);
    };
  }
}
