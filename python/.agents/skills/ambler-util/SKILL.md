name: ambler-util
description: Creates or extracts utility modules into the utils/ directory of an Ambler project. Use this whenever a node's defaultUtils needs external dependencies, contains logic reusable across multiple nodes, or has implementations too complex to inline.
metadata:
  author: leandro
  version: "1.0-py"

# Ambler Util (Python)

Follow these steps to create or extract a utility module in the `utils/` directory.

## When to create a utility

Create a `utils/<name>.py` file when a node's `DefaultUtils` contains:

1. **External dependencies** (e.g., `requests`, `numpy`)
2. **Reusable logic**
3. **Complex implementations**

---

## 1. Gather requirements

- **Utility name**: File will be named `utils/<name>.py`.
- **Functions to export**: What are the signatures?

---

## 2. Create `utils/<name>.py`

```python
# Example: utils/weather_util.py
import requests

def fetch_weather(city: str, api_key: str) -> float:
    """
    Fetches the current temperature for a city.
    
    :param city: City name
    :param api_key: API key
    :return: Current temperature in Celsius
    """
    response = requests.get(f"https://api.openweathermap.org/data/2.5/weather?q={city}&appid={api_key}")
    return response.json()["main"]["temp"]
```

---

## 3. Connect to the node

Import the utility in the node file and use it in `DefaultUtils`.

```python
from utils.weather_util import fetch_weather

class DefaultUtils:
    def get_weather(self, city, key):
        return fetch_weather(city, key)
```
