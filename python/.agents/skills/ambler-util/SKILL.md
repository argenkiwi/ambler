---
name: ambler-util
description: Creates or extracts utility modules into the utils/ directory of a Python Ambler project. Use this whenever a node's DefaultUtils needs third-party (PyPI) dependencies, contains logic reusable across multiple nodes, or has implementations too complex to inline — even if the user just says "add a util", "extract this helper", "share this function", "create a utility", or "this code is getting too long for the node". When a user creates a node that imports a third-party package or has complex async logic, proactively suggest using this skill.
metadata:
  author: leandro
  version: "2.0-py"
---

# Ambler Util (Python)

Follow these steps to create or extract a utility module in the `utils/` directory.

## When to create a utility

Create a `utils/<name>.py` file when a node's `DefaultUtils` contains:

1. **External dependencies** — any third-party package installed from PyPI (e.g., `ollama`, `requests`, `markdown`)
2. **Reusable logic** — code that is or could be used by more than one node
3. **Complex implementations** — functions with significant logic (error handling, retries, client caching, multi-step operations) that clutter the node file

Simple one-liners like `print(message)` or `random.random()` belong in `DefaultUtils` directly — no need to extract them.

---

## 1. Gather requirements

Before writing, determine:

- **Utility name**: What capability does this module provide? (e.g., `ollama_chat`, `file_writer`, `markdown_parser`). File will be named `utils/<name>.py` (snake_case).
- **Functions to export**: What are the signatures? What arguments and return types? Sync or `async`?
- **Dependencies**: Does it require PyPI packages? Which ones and what versions?
- **Nodes that will use it**: Which node(s) will import this utility?

If any of the above is unclear, ask the user before writing code.

---

## 2. Create `utils/<name>.py`

Export functions that implement the side-effectful work. Keep the module focused on a single capability.

```python
# Example: utils/ollama_chat.py
from typing import Dict, List
from ollama import AsyncClient

# Stateful resources (clients, connection pools) live here — private.
_clients: Dict[str, AsyncClient] = {}


async def ollama_chat(host: str, model: str, messages: List[Dict[str, str]]) -> str:
    """
    Sends a chat request to an Ollama server and returns the reply.

    Args:
        host: Ollama server URL (e.g. "http://localhost:11434")
        model: Model name (e.g. "llama3.2")
        messages: Chat history as a list of {"role", "content"} dicts

    Returns:
        The assistant's reply text.
    """
    client = _clients.get(host)
    if client is None:
        client = AsyncClient(host=host)
        _clients[host] = client
    response = await client.chat(model=model, messages=messages)
    return response["message"]["content"]
```

### Rules

- **Export only functions** — never expose raw client instances, connection objects, or module-level state. Callers interact through function calls with clear signatures.
- **Keep state private** — if the utility needs to cache a client or pool connections, use a module-level `dict` or variable with a leading underscore (e.g., `_clients`), and never import it from elsewhere.
- **Match the node's `Utils` type** — function signatures must line up with what the consuming node's `Utils` `Protocol` declares, so `DefaultUtils` can delegate directly. If the `Protocol` method returns `Awaitable[...]`, the utility should be `async def`.
- **No Ambler imports** — utility files must not import from `ambler` or `nodes`. They are plain Python helpers.
- **Extract URL and config constants** — don't inline magic strings like API base URLs. Pull them into named UPPER_CASE module constants at the top of the file so they're easy to find and change.

### Docstrings

Every public function gets a docstring. This helps both humans reading the code and agents using the utility in future nodes understand what it does without having to trace the implementation.

```python
API_BASE_URL = "https://api.openweathermap.org/data/2.5"


def fetch_weather(city: str, api_key: str) -> float:
    """
    Fetches the current temperature for a city from the OpenWeather API.

    Args:
        city: City name (e.g. "London")
        api_key: OpenWeather API key

    Returns:
        Current temperature in Celsius.
    """
```

Keep it brief — one-sentence summary, then `Args:` and `Returns:` sections (the same Google style used in `ambler.py`). Skip `Raises:` unless the function has a documented, meaningful error contract.

---

## 3. Add dependencies (if using PyPI packages)

If the utility imports a third-party package, add it to `requirements.txt` at the project root (create the file if it doesn't exist), then install it into the project's virtual environment:

```text
ollama>=0.6.0
```

```bash
python3 -m pip install -r requirements.txt
```

Use one package per line with a minimum version. Check `requirements.txt` first — the package may already be listed.

---

## 4. Connect to the node

Import the utility in the node file and use it in `DefaultUtils`. The `Utils` `Protocol` stays unchanged — only the implementation in `DefaultUtils` changes.

```python
from typing import Dict, List
from utils.ollama_chat import ollama_chat


class DefaultUtils:
    async def chat(self, messages: List[Dict[str, str]], host: str, model: str) -> str:
        return await ollama_chat(host, model, messages)
```

Tests still inject a `MockUtils` and never touch the utility — that's the point of the `Utils` interface.

---

## 5. Checklist before finishing

- [ ] `utils/<name>.py` exists and exports the required functions.
- [ ] All PyPI packages used are listed in `requirements.txt`.
- [ ] Function signatures match the node's `Utils` `Protocol`.
- [ ] Stateful resources (clients, caches) are private (`_`-prefixed) in the util module.
- [ ] URL/config constants are extracted to named UPPER_CASE constants.
- [ ] Every public function has a docstring (`Args:`, `Returns:`).
- [ ] The node's `DefaultUtils` delegates to the utility function.
- [ ] No `ambler` or `nodes` imports appear in the utility file.
- [ ] Node tests still use mocked utils — they need no changes.
