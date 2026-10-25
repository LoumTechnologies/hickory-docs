<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/src/hickory_agent_worker/runtime.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A running agent can be stopped](../../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md)
- [Model snippets cannot hang or flood an agent session indefinitely](../../../../../../guarantees/agent/a-script-cannot-hang-or-flood-a-session/contract.md)
- [An exhausted Python iteration budget retains incomplete work](../../../../../../guarantees/agent/an-exhausted-turn-budget-hands-off/contract.md)
- [Completion does not certify runtime verification](../../../../../../guarantees/agent/completion-does-not-certify-verification/contract.md)
- [Python agent code crosses two explicit boundaries](../../../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md)
- [Python model requests carry validated context manifests](../../../../../../guarantees/agent/python-requests-have-validated-context/contract.md)

Captured SHA-256: `d409e5e43d77cdf00df4466991d7f51853c45413e090f42aeda95645939cf935`.

<book:entry path="apps/agent/src/hickory_agent_worker/runtime.py">
"""Bounded hosted transport and restricted model-code execution."""
import json
import sys
from pathlib import Path

from pydantic_monty import Monty

MAX_FRAME = 1_048_576


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


</book:entry>

</book:doc>
