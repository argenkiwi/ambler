---
name: ambler
description: "Builds this Ambler project, where a program is a walk (a state machine) of small, tested nodes, using the project's own `<cli>` command line. Use this whenever the user wants to add or change a node, step, state, handler, walk, flow, program, utility or spec, even if they only describe the behaviour: \"add a step that asks for a name\", \"wire up these nodes\", \"extract this helper\", \"write a spec\", \"add tests for this node\"."
metadata:
  author: leandro
  version: "2.0"
---

# Ambler

`<cli>` is this project's own command line. It holds the conventions and does the mechanical work: it writes files in the right place, registers them and generates the wiring. Your part is the design: which outcomes a node has, what state it needs, what its tests prove, where each outcome leads.

Do not write or register nodes, walks or utility modules by hand, and do not work from memory of the conventions. Run the verb, read the guide it points to, then edit what was generated.

| The user wants | Run | Read first |
|---|---|---|
| A node, step or handler | `<cli> node <name> …` | `<cli> help node-design` |
| Tests for a node | nothing to generate | `<cli> help testing` |
| A helper shared by nodes, or one that needs a crate or package | `<cli> util <name>` | `<cli> help util` |
| A walk, flow or program | `<cli> walk <name> --nodes … --edge …` | `<cli> help walk` |
| A spec | written by hand; `<cli> walk` starts it | `<cli> help spec` |
| To know the work is done | `<cli> check` | `<cli> help check` |

`<cli> help` with no topic shows the layout and the usual order of work. If the command is missing, run `<setup>`.
