"""Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md"""
import io
import json
import sys
import unittest
from pathlib import Path

from pydantic_monty import Monty
from hickory_agent_worker.__main__ import Broker, execute, parse_action, run


class WorkerTests(unittest.TestCase):
    def test_only_one_declared_python_or_done_action(self):
        self.assertEqual(parse_action('{"python":"1 + 2"}'), {"python": "1 + 2"})
        for raw in ['{"shell":"rm -rf ."}', '{"python":"x","done":"ok"}', '[]', '```python\nx\n```']:
            with self.assertRaises(ValueError):
                parse_action(raw)

    def test_broker_rejects_wrong_id_and_oversized_frames(self):
        broker = Broker(io.StringIO('{"id":2,"result":"forged"}\n'), io.StringIO())
        with self.assertRaisesRegex(RuntimeError, "Mismatched"):
            broker.call("model", {})
        with self.assertRaisesRegex(RuntimeError, "Oversized"):
            broker.send({"x": "a" * 1_048_576})

    def test_monty_has_no_host_filesystem_or_cpython_escape(self):
        with Monty(binary_path=str(Path(sys.executable).parent / "monty"), request_timeout=5) as pool:
            with pool.checkout(limits={"max_memory": 2_000_000, "max_feed_duration_secs": 0.2}) as session:
                self.assertEqual(execute(session, None, "1 + 2")["result"], 3)
                output = execute(session, None, "print(42)")
                self.assertEqual(output["printed"], "42\n")
                self.assertEqual("".join(event["text"] for event in output["print_events"]), "42\n")
                self.assertTrue(all(event["stream"] == "stdout" for event in output["print_events"]))
                # Monty's eval is itself interpreted; it must not reach CPython.
                for code in ["open('/etc/passwd').read()", "import subprocess\nsubprocess.run(['sh'])", "eval(\"open('/etc/passwd').read()\")", "'x' * 10**10", "while True:\n    pass"]:
                    self.assertFalse(execute(session, None, code)["ok"], code)

    def test_tools_are_the_only_host_callback(self):
        class FakeBroker:
            def call(self, name, args):
                self.request = name, args
                return {"from": "rust"}
        broker = FakeBroker()
        with Monty(binary_path=str(Path(sys.executable).parent / "monty")) as pool:
            with pool.checkout() as session:
                result = execute(session, broker, 'tool("read_doc", {"doc":"note.md"})')
                self.assertEqual(result["result"], {"from": "rust"})
                self.assertEqual(broker.request, ("tool", {"name": "read_doc", "arguments": {"doc": "note.md"}}))

    def test_langgraph_never_interprets_shell_actions(self):
        class FakeBroker:
            def __init__(self):
                self.models = iter(['{"shell":"echo hi"}', '{"done":"Unchanged"}'])
                self.calls = []
            def call(self, name, args):
                self.calls.append(name)
                return next(self.models) if name == "model" else {"accepted": True}
            def send(self, value):
                self.exit = value
        broker = FakeBroker()
        run(broker, {"max_turns": 2}, None)
        self.assertNotIn("begin_python", broker.calls)
        self.assertTrue(broker.exit["params"]["completed"])


if __name__ == "__main__":
    unittest.main()
