"""Private worker protocol. The user-facing CLI is the Rust hickory-agent binary."""
import json
import sys
from pathlib import Path
from typing import TypedDict

from langgraph.graph import END, START, StateGraph
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
        return response["result"]


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


class State(TypedDict):
    action: dict
    turns: int
    stopped: bool


def run(broker, bootstrap, session):
    # The Rust host owns the conversation, actual tool observations and model
    # requests. The graph never supplies substitute messages or test verdicts.
    def model(state):
        raw = broker.call("model", {})
        try:
            return {"action": parse_action(raw), "turns": state["turns"] + 1}
        except (ValueError, TypeError) as exc:
            broker.call("observation", {"ok": False, "error": str(exc)})
            return {"action": {}, "turns": state["turns"] + 1}

    def act(state):
        action = state["action"]
        if "done" in action:
            result = broker.call("finish", {"summary": action["done"]})
            return {"stopped": result["accepted"]}
        if "python" in action:
            broker.call("begin_python", {"code": action["python"]})
            outcome = execute(session, broker, action["python"])
            broker.call("observation", outcome)
        return {"stopped": False}

    def route(state):
        return END if state["stopped"] or state["turns"] >= bootstrap["max_turns"] else "model"

    graph = StateGraph(State)
    graph.add_node("model", model)
    graph.add_node("act", act)
    graph.add_edge(START, "model")
    graph.add_edge("model", "act")
    graph.add_conditional_edges("act", route)
    result = graph.compile().invoke(
        {"action": {}, "turns": 0, "stopped": False},
        {"recursion_limit": bootstrap["max_turns"] * 3 + 5},
    )
    broker.send({"method": "exit", "params": {"completed": result["stopped"]}})


def main():
    broker = Broker()
    bootstrap = broker.receive()
    # Pin the installed runtime, rather than resolving a model-controlled PATH.
    binary = Path(sys.executable).parent / "monty"
    with Monty(binary_path=str(binary), min_processes=1, max_processes=1,
               request_timeout=15) as pool:
        with pool.checkout(limits={
            "max_memory": 32_000_000,
            "max_feed_duration_secs": 5.0,
            "max_turn_duration_secs": 5.0,
            "max_suspensions": 64,
            "max_recursion_depth": 100,
            "max_total_sleep_secs": 0,
        }) as session:
            run(broker, bootstrap, session)


if __name__ == "__main__":
    main()
