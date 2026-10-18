"""Model response protocol; interpreted snippets remain separate from policy."""
import json

def parse_action(raw):
    value = json.loads(raw)
    if not isinstance(value, dict):
        raise ValueError("Return a JSON object")
    if set(value) == {"python"} and isinstance(value["python"], str):
        if len(value["python"].encode()) > 65_536:
            raise ValueError("Python snippet exceeds 64 KiB")
        return value
    if set(value) == {"done"} and isinstance(value["done"], str):
        return value
    raise ValueError('Use exactly {"python": "..."} or {"done": "..."}')

