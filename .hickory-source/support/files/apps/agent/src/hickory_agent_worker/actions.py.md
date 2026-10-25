<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/src/hickory_agent_worker/actions.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3bb4133bb950135f17025c785f601665bfcc046a40c04367d1409c5778afb7df`.

<book:entry path="apps/agent/src/hickory_agent_worker/actions.py">
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

</book:entry>

</book:doc>
