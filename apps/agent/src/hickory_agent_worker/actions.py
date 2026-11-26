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
    if set(value) == {"tool", "arguments"} and isinstance(value["tool"], str) and value["tool"].strip() and isinstance(value["arguments"], dict):
        if len(json.dumps(value, allow_nan=False).encode()) > 65_536:
            raise ValueError("Direct tool action exceeds 64 KiB; use a bounded snippet")
        return value
    raise ValueError('Use exactly {"tool":"name","arguments":{}}, {"python":"..."}, or {"done":"..."}')

