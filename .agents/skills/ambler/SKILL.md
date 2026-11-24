---
name: ambler
description: "Builds programs with Ambler, where a program is a walk (a state machine) of small, tested nodes, using the `ambler` command line. Use this whenever the user wants to start an Ambler project, or to add or change a node, step, state, handler, walk, flow, program, utility or spec in one, even if they only describe the behaviour: \"add a step that asks for a name\", \"new program\", \"wire up these nodes\", \"extract this helper\", \"write a spec\", \"add tests for this node\"."
metadata:
  author: leandro
  version: "1.0"
---

# Ambler

The `ambler` command line holds the conventions and does the mechanical work: it writes files in the right place, registers them and generates the wiring. Your part is the design: which outcomes a node has, what state it needs, what its tests prove, where each outcome leads.

Do not write or register nodes, walks or utility modules by hand, and do not work from memory of the conventions. Run the verb, read the guide it points to, then edit what was generated.

| The user wants | Run | Read first |
|---|---|---|
| A new project | `ambler new <name>`, or `ambler new` in an empty folder (`--example` adds a reference walk, `--target ts` makes it TypeScript, `--target py` Python) | `ambler help new` |
| A node, step or handler | `ambler node <name> …` | `ambler help node-design` |
| Tests for a node | nothing to generate | `ambler help testing` |
| A helper shared by nodes, or one that needs a crate or package | `ambler util <name>` | `ambler help util` |
| A walk, flow or program | `ambler walk <name> --nodes … --edge …` | `ambler help walk` |
| A spec | written by hand; `ambler walk` starts it | `ambler help spec` |
| To know the work is done | `ambler check` | `ambler help check` |
| A whole spec implemented by an agent loop, one task per run | `ambler pave <spec> --agent '<command>'` | `ambler help pave` |

`ambler help` with no topic shows the layout and the usual order of work. If the command is missing, install it with `cargo install --git https://github.com/argenkiwi/ambler`.
