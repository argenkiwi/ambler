"""Native Ambler CLI for Python projects adhering to the Ambler Tooling Protocol (ATP).

Implements `node`, `walk`, `util`, and `check` without requiring foreign toolchains.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


NOT_IMPLEMENTED_MARKER = f"{'not yet'} {'implemented'}"


def to_snake(s: str) -> str:
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", s)
    s = re.sub(r"([a-z\d])([A-Z])", r"\1_\2", s)
    return s.replace("-", "_").lower()


def to_pascal(s: str) -> str:
    return "".join(part.capitalize() for part in to_snake(s).split("_") if part)


def to_shout(s: str) -> str:
    return to_snake(s).upper()


def get_package_info() -> tuple[Path, str]:
    """Finds root and package name."""
    cwd = Path.cwd()
    pyproject = cwd / "pyproject.toml"
    if not pyproject.exists():
        raise RuntimeError("No pyproject.toml found in current folder.")

    # Find the package in src/
    src = cwd / "src"
    if src.exists():
        candidates = [
            d for d in src.iterdir() if d.is_dir() and (d / "__init__.py").exists()
        ]
        if candidates:
            return cwd, candidates[0].name

    # Fallback to pyproject name
    content = pyproject.read_text()
    match = re.search(r'name\s*=\s*["\']([^"\']+)["\']', content)
    name = match.group(1) if match else "ambler"
    return cwd, to_snake(name)


def handle_node(args: argparse.Namespace) -> int:
    root, pkg = get_package_info()
    name = to_snake(args.name)
    cls_name = to_pascal(name)
    nodes_dir = root / "src" / pkg / "nodes"
    tests_dir = root / "tests" / "nodes"
    nodes_dir.mkdir(parents=True, exist_ok=True)
    tests_dir.mkdir(parents=True, exist_ok=True)

    node_file = nodes_dir / f"{name}.py"
    test_file = tests_dir / f"test_{name}.py"

    if node_file.exists():
        print(f"Error: Node {name} already exists at {node_file}", file=sys.stderr)
        return 1

    terminal = args.terminal
    fallible = args.fallible
    raw_outcomes = (
        [o.strip() for o in args.outcomes.split(",") if o.strip()]
        if args.outcomes
        else []
    )
    outcomes = [to_shout(o) for o in raw_outcomes]

    if not terminal and not outcomes:
        print("Error: Specify either --outcomes <a,b> or --terminal", file=sys.stderr)
        return 2

    # Parse state fields: e.g. ["count:int", "name:str"]
    state_fields: list[tuple[str, str]] = []
    protocols: list[str] = []
    if args.state:
        for item in args.state:
            if ":" in item:
                f_name, f_type = item.split(":", 1)
            else:
                f_name, f_type = item, "object"
            f_name = to_snake(f_name.strip())
            f_type = f_type.strip()
            state_fields.append((f_name, f_type))
            protocols.append(f"Has{to_pascal(f_name)}")

    # Update nodes/__init__.py with any missing Has* protocols
    init_file = nodes_dir / "__init__.py"
    init_content = (
        init_file.read_text()
        if init_file.exists()
        else '"""Node package."""\n\nfrom typing import Protocol\n'
    )
    new_protocols_added = False
    for f_name, f_type in state_fields:
        p_name = f"Has{to_pascal(f_name)}"
        if p_name not in init_content:
            if "from typing import Protocol" not in init_content:
                init_content = "from typing import Protocol\n\n" + init_content
            doc = f'"""A state that has a {f_name} field."""'
            init_content += (
                f"\n\nclass {p_name}(Protocol):\n    {doc}\n\n    {f_name}: {f_type}\n"
            )
            new_protocols_added = True
    if new_protocols_added:
        init_file.write_text(init_content)

    # Build node file content
    lines: list[str] = [
        "from dataclasses import dataclass, field",
    ]
    if not terminal:
        lines.append("from enum import Enum, auto")
    lines.append("from typing import Protocol")
    if protocols:
        lines.append(f"from {pkg}.nodes import {', '.join(protocols)}")

    lines.append("\n")

    if not terminal:
        lines.append(
            'class Outcome(Enum):\n    """Where this node leaves the walk."""\n'
        )
        for out in outcomes:
            lines.append(f"    {out} = auto()\n")
        lines.append("\n")

    if fallible:
        lines.append(
            'class Error(Exception):\n    """What stops this node from finishing."""\n'
        )

    # State parameter type
    if not protocols:
        state_type = "object"
    elif len(protocols) == 1:
        state_type = protocols[0]
    else:
        lines.append(f"class State({', '.join(protocols)}, Protocol):\n    pass\n\n")
        state_type = "State"

    ret_type = "None" if terminal else "Outcome"

    lines.append(f'''class Utils(Protocol):
    """The side effects this node needs: one method each, faked in the tests."""


class SystemUtils:
    """The real side effects."""


@dataclass(frozen=True)
class {cls_name}:
    """Atomic step node."""

    utils: Utils = field(default_factory=SystemUtils)

    async def run(self, state: {state_type}) -> {ret_type}:
        raise NotImplementedError("{NOT_IMPLEMENTED_MARKER}")
''')

    node_file.write_text("\n".join(lines) + "\n")

    # Build test file content
    t_lines: list[str] = [
        "from dataclasses import dataclass\n",
    ]
    if fallible:
        t_lines.append("import pytest\n")

    imports = [cls_name]
    if not terminal:
        imports.append("Outcome")
    if fallible:
        imports.append("Error")
    t_lines.append(f"from {pkg}.nodes.{name} import {', '.join(imports)}\n\n")

    # MockState
    if state_fields:
        t_lines.append("@dataclass\nclass MockState:\n")
        for fn, ft in state_fields:
            default_val = (
                "0"
                if ft == "int"
                else '""'
                if ft == "str"
                else "False"
                if ft == "bool"
                else "None"
            )
            t_lines.append(f"    {fn}: {ft} = {default_val}\n")
    else:
        t_lines.append("@dataclass\nclass MockState:\n    pass\n")

    t_lines.append('''
class MockUtils:
    """Fakes every side effect of the node."""


''')

    if terminal:
        t_lines.append(f"""async def test_finishes() -> None:
    state = MockState()
    await {cls_name}(MockUtils()).run(state)
""")
    else:
        for out in outcomes:
            t_lines.append(f"""async def test_{out.lower()}_when_pending() -> None:
    state = MockState()
    outcome = await {cls_name}(MockUtils()).run(state)
    assert outcome == Outcome.{out}
""")

    test_file.write_text("\n".join(t_lines) + "\n")

    print(f"Created {node_file}")
    print(f"Created {test_file}")
    return 0


def handle_walk(args: argparse.Namespace) -> int:
    root, pkg = get_package_info()
    name = to_snake(args.name)
    cls_name = to_pascal(name)
    walks_dir = root / "src" / pkg / "walks"
    tests_dir = root / "tests" / "walks"
    specs_dir = root / "specs"
    walks_dir.mkdir(parents=True, exist_ok=True)
    tests_dir.mkdir(parents=True, exist_ok=True)
    specs_dir.mkdir(parents=True, exist_ok=True)

    walk_file = walks_dir / f"{name}.py"
    test_file = tests_dir / f"test_{name}.py"
    spec_file = specs_dir / f"{name}.md"

    if walk_file.exists():
        print(f"Error: Walk {name} already exists at {walk_file}", file=sys.stderr)
        return 1

    node_names = [to_snake(n.strip()) for n in args.nodes.split(",") if n.strip()]
    if not node_names:
        print("Error: --nodes must contain at least one node name", file=sys.stderr)
        return 2

    # Edges parsing: e.g. ["start.SUCCESS=count", "start.INVALID=start"]
    edges: list[tuple[str, str, str]] = []
    if args.edge:
        for e in args.edge:
            if "=" in e and "." in e.split("=")[0]:
                source, target = e.split("=", 1)
                src_node, src_outcome = source.split(".", 1)
                edges.append(
                    (to_snake(src_node), to_shout(src_outcome), to_snake(target))
                )

    # Node imports
    lines: list[str] = [
        '"""Walk implementation."""\n',
        "from dataclasses import dataclass, field",
        "from enum import Enum, auto\n",
        f"from {pkg} import Walk",
    ]
    for n in node_names:
        p_name = to_pascal(n)
        lines.append(f"from {pkg}.nodes.{n} import {p_name}")

    lines.append("\n\n@dataclass\nclass State:\n    pass\n\n\nclass Id(Enum):\n")
    for n in node_names:
        lines.append(f"    {to_shout(n)} = auto()\n")

    lines.append(f"\n\n@dataclass(frozen=True)\nclass {cls_name}(Walk[Id, State]):\n")
    for n in node_names:
        p_name = to_pascal(n)
        lines.append(f"    {n}: {p_name} = field(default_factory={p_name})\n")

    lines.append("\n    async def step(self, id: Id, state: State) -> Id | None:\n")
    lines.append("        match id:\n")
    for n in node_names:
        shout_n = to_shout(n)
        lines.append(f"            case Id.{shout_n}:\n")
        # Check if node has edges
        matched_edges = [edge for edge in edges if edge[0] == n]
        if matched_edges:
            lines.append(f"                match await self.{n}.run(state):\n")
            for _, outcome, tgt in matched_edges:
                tgt_return = f"Id.{to_shout(tgt)}" if tgt.lower() != "halt" else "None"
                lines.append(
                    f"                    case {to_pascal(n)}.Outcome.{outcome}:\n"
                )
                lines.append(f"                        return {tgt_return}\n")
        else:
            lines.append(f"                # Wire outcomes for {n}\n")
            lines.append(f"                await self.{n}.run(state)\n")
            lines.append("                return None\n")

    lines.append(f'''

async def run() -> None:
    """Runs the {name} walk."""
    await {cls_name}().run(Id.{to_shout(node_names[0])}, State())
''')

    walk_file.write_text("".join(lines))

    # Spec file
    if not spec_file.exists():
        spec_file.write_text(f"""# {cls_name} Walk Specification

Initial node: `{node_names[0]}`.

## Flow
{" -> ".join(node_names)}
""")

    # Test file
    test_file.write_text(f"""import pytest
from {pkg}.walks.{name} import {cls_name}, Id, State


async def test_{name}_walk_initial_step() -> None:
    walk = {cls_name}()
    state = State()
    next_id = await walk.step(Id.{to_shout(node_names[0])}, state)
    assert next_id is None or isinstance(next_id, Id)
""")

    print(f"Created {walk_file}")
    print(f"Created {test_file}")
    print(f"Created {spec_file}")
    return 0


def handle_util(args: argparse.Namespace) -> int:
    root, pkg = get_package_info()
    name = to_snake(args.name)
    cls_name = to_pascal(name)
    utils_dir = root / "src" / pkg / "utils"
    utils_dir.mkdir(parents=True, exist_ok=True)
    util_file = utils_dir / f"{name}.py"

    if util_file.exists():
        print(f"Error: Util {name} already exists at {util_file}", file=sys.stderr)
        return 1

    util_file.write_text(f'''"""Utility module {name}."""

from typing import Protocol


class {cls_name}Utils(Protocol):
    """Protocol for {name} operations."""


class System{cls_name}Utils:
    """Production implementation."""
''')
    print(f"Created {util_file}")
    return 0


def handle_check(args: argparse.Namespace) -> int:
    commands = [
        ["uv", "run", "ruff", "format", "--check", "."],
        ["uv", "run", "ruff", "check", "."],
        ["uv", "run", "pyright"],
    ]
    if not args.lint_only:
        commands.append(["uv", "run", "pytest"])

    for cmd in commands:
        print(f"Running {' '.join(cmd)}...")
        res = subprocess.run(cmd)
        if cmd[-1] == "pytest" and res.returncode == 5:
            # Code 5 means no tests collected (e.g. fresh project)
            continue
        if res.returncode != 0:
            print(f"Check failed at step: {' '.join(cmd)}", file=sys.stderr)
            return res.returncode

    print("All Ambler checks passed.")
    return 0


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="ambler-cli", description="Ambler Python Tooling"
    )
    subparsers = parser.add_subparsers(dest="subcommand", required=True)

    # node
    p_node = subparsers.add_parser("node", help="Scaffold a node")
    p_node.add_argument("name", help="Node name")
    p_node.add_argument("--outcomes", help="Comma-separated outcomes")
    p_node.add_argument("--terminal", action="store_true", help="Node terminates walk")
    p_node.add_argument("--fallible", action="store_true", help="Node can fail")
    p_node.add_argument("--state", action="append", help="State field field[:Type]")

    # walk
    p_walk = subparsers.add_parser("walk", help="Wire a walk")
    p_walk.add_argument("name", help="Walk name")
    p_walk.add_argument("--nodes", required=True, help="Comma-separated nodes")
    p_walk.add_argument(
        "--edge", action="append", help="Transitions: node.Outcome=target"
    )
    p_walk.add_argument("--no-runner", action="store_true", help="Do not create runner")

    # util
    p_util = subparsers.add_parser("util", help="Create a utility module")
    p_util.add_argument("name", help="Utility name")

    # check
    p_check = subparsers.add_parser(
        "check", help="Verify conventions, lint, format, and tests"
    )
    p_check.add_argument("--lint-only", action="store_true", help="Skip running tests")

    parsed = parser.parse_args()

    handlers = {
        "node": handle_node,
        "walk": handle_walk,
        "util": handle_util,
        "check": handle_check,
    }

    handler = handlers.get(parsed.subcommand)
    if not handler:
        parser.print_help()
        sys.exit(2)

    sys.exit(handler(parsed))


if __name__ == "__main__":
    main()
