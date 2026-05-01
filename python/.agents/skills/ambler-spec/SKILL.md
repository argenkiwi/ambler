name: ambler-spec
description: Creates or updates a Markdown specification file for an Ambler walk in the specs/ directory. Use this whenever a user wants to document, plan, or describe a walk — including "write a spec", "document this walk", or "plan out the steps" — even before any code exists.
metadata:
  author: leandro
  version: "1.0-py"

# Ambler Specification (Python)

This skill guides you in creating a Markdown specification file (`specs/<walk-name>.md`) for an Ambler walk.

## Instructions

### 1. Identify the Walk's Name and Purpose

- The file should be named `specs/<name>.md`.

### 2. Determine the Shared State

- Describe it under a `## Shared State` heading.

### 3. Map the Steps (Nodes)

- Identify all nodes in the walk.
- For each node, create a `### <Node Name>` subsection under `## Steps`.

### 4. Format the Markdown

Follow the exact format of `specs/counter.md`.

## Guidelines

- **Node name casing**: Use Title Case for headings (`### Count`) and backtick-quoted ALL_CAPS for references (`` `COUNT` ``).
- **Consistency**: If `walks/<name>.py` already exists, ensure the spec reflects it.
- **No extra sections**: Stick to `# Program Specifications`, `## Shared State`, and `## Steps`.
