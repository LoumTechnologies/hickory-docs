<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/agent/src/hickory_agent_worker/actions.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c95d123c0e92"></a>
## `apps/agent/src/hickory_agent_worker/actions.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

<a id="f-c95d123c0e92-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c95d123c0e92-expression_statement">
"""Model response protocol; interpreted snippets remain separate from policy."""
</storycode:copy>
<a id="f-c95d123c0e92-json"></a>
### `json`

<storycode:copy id="f-c95d123c0e92-json">
import json

</storycode:copy>
<a id="f-c95d123c0e92-parse_action"></a>
### `parse_action`

<storycode:copy id="f-c95d123c0e92-parse_action">
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

</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/actions.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/actions.py"><storycode:paste select="#f-c95d123c0e92-expression_statement"/><storycode:paste select="#f-c95d123c0e92-json"/><storycode:paste select="#f-c95d123c0e92-parse_action"/></storycode:file>

</storycode:doc>
