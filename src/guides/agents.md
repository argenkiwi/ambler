# ambler agents

How coding agents learn to work in an Ambler project. Three places, each with one job, and none of them restates the conventions:

- This command holds the reference. `ambler help <topic>` is the only place a convention is written down, and it ships with the binary that enforces it, so the two cannot drift apart.
- `AGENTS.md` holds behaviour: a few standing rules that are always in the agent's context, each pointing at a guide. `CLAUDE.md` is one line that imports it.
- `.agents/skills/ambler/SKILL.md` is a router: it triggers on requests such as "add a step" and maps each to a verb and a guide. Installed globally, it also tells an agent in an empty folder to start with `ambler new`.

`ambler new` writes all three, and links `.claude/skills` to `.agents/skills`.

For a project that was not created by `ambler new`, add the text below to its `AGENTS.md`, and copy the skill from ambler's repository if you want the triggers. There is no verb for this: it is done once per project.

---

