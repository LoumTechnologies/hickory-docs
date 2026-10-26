<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/agent/src/hickory_agent_worker/runtime.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-56e1a215e79c"></a>
## `apps/agent/src/hickory_agent_worker/runtime.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [A running agent can be stopped](../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md), [Model snippets cannot hang or flood an agent session indefinitely](../../../../../guarantees/agent/a-script-cannot-hang-or-flood-a-session/contract.md), [An exhausted Python iteration budget retains incomplete work](../../../../../guarantees/agent/an-exhausted-turn-budget-hands-off/contract.md), [Distinguish an accepted change from a verified runtime result](../../../../../guarantees/agent/completion-does-not-certify-verification/contract.md), [Separate model snippets from trusted host operations](../../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md), [Python model requests carry validated context manifests](../../../../../guarantees/agent/python-requests-have-validated-context/contract.md).

<a id="f-56e1a215e79c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-56e1a215e79c-expression_statement">
"""Bounded hosted transport and restricted model-code execution."""
</storycode:copy>
<a id="f-56e1a215e79c-json"></a>
### `json`

<storycode:copy id="f-56e1a215e79c-json">
import json
</storycode:copy>
<a id="f-56e1a215e79c-sys"></a>
### `sys`

<storycode:copy id="f-56e1a215e79c-sys">
import sys
</storycode:copy>
<a id="f-56e1a215e79c-Path"></a>
### `Path`

<storycode:copy id="f-56e1a215e79c-Path">
from pathlib import Path

</storycode:copy>
<a id="f-56e1a215e79c-Monty"></a>
### `Monty`

<storycode:copy id="f-56e1a215e79c-Monty">
from pydantic_monty import Monty

</storycode:copy>
<a id="f-56e1a215e79c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-56e1a215e79c-expression_statement-2">
MAX_FRAME = 1_048_576


</storycode:copy>
<a id="f-56e1a215e79c-Broker"></a>
### `Broker`

<storycode:copy id="f-56e1a215e79c-Broker">
class Broker:
    def __init__(self, reader=sys.stdin, writer=sys.stdout):
        self.reader, self.writer = reader, writer
        self.sequence = 0

    def receive(self):
        line = self.reader.readline(MAX_FRAME + 1)
        if not line or len(line.encode()) > MAX_FRAME or not line.endswith("\n"):
            raise RuntimeError("Missing or oversized broker frame")
        return json.loads(line)

    def send(self, value):
        line = json.dumps(value, ensure_ascii=True, allow_nan=False) + "\n"
        if len(line.encode()) > MAX_FRAME:
            raise RuntimeError("Oversized worker frame")
        self.writer.write(line)
        self.writer.flush()

    def call(self, method, params):
        self.sequence += 1
        self.send({"id": self.sequence, "method": method, "params": params})
        response = self.receive()
        if response.get("id") != self.sequence:
            raise RuntimeError("Mismatched broker response")
        if "error" in response:
            raise RuntimeError(response["error"])
        result = response["result"]
        if method == 'tool' and hasattr(self, 'context') and isinstance(result, dict):
            self.context.import_references(result.get('context_references', []))
        return result




</storycode:copy>
<a id="f-56e1a215e79c-execute"></a>
### `execute`

<storycode:copy id="f-56e1a215e79c-execute">
def execute(session, broker, code):
    # No mounts, filesystem callbacks, CPython eval/exec, shell or provider SDK.
    # Every external effect is an RPC the Rust supervisor independently checks.
    printed = []
    print_events = []
    size = 0

    def capture(stream, chunk):
        nonlocal size
        size += len(chunk.encode())
        if size > 65_536:
            raise RuntimeError("Printed output exceeds 64 KiB")
        if len(print_events) >= 1024:
            raise RuntimeError("Print event budget exhausted")
        printed.append(chunk)
        print_events.append({"stream": stream, "text": chunk})

    try:
        result = session.feed_run(
            code,
            external_lookup={"tool": lambda name, arguments: broker.call(
                "tool", {"name": name, "arguments": arguments}
            )},
            print_callback=capture,
        )
        # Avoid exporting arbitrary interpreter proxies or non-JSON values.
        json.dumps(result, allow_nan=False)
        return {"ok": True, "result": result, "printed": "".join(printed),
                "print_events": print_events}
    except Exception as exc:
        return {"ok": False, "error": str(exc)[:8192], "printed": "".join(printed),
                "print_events": print_events}


</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/runtime.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/runtime.py"><storycode:paste select="#f-56e1a215e79c-expression_statement"/><storycode:paste select="#f-56e1a215e79c-json"/><storycode:paste select="#f-56e1a215e79c-sys"/><storycode:paste select="#f-56e1a215e79c-Path"/><storycode:paste select="#f-56e1a215e79c-Monty"/><storycode:paste select="#f-56e1a215e79c-expression_statement-2"/><storycode:paste select="#f-56e1a215e79c-Broker"/><storycode:paste select="#f-56e1a215e79c-execute"/></storycode:file>

</storycode:doc>
