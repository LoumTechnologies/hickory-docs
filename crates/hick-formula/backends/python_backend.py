#!/usr/bin/env python3
"""A formula backend for Python.

The whole of it. It reads LSP-framed JSON-RPC on stdin, evaluates one
expression per formula with its bindings in scope, and writes the results
back. It knows nothing about cells, sheets, dependencies, or order — the host
resolved all of that before asking (see crates/hick-formula/src/graph.rs), and
that split is what keeps this file short enough to read in one sitting.

Evaluation is `eval` with a restricted globals dict. That is not a security
boundary and is not pretended to be one: this runs under the same executor
policy as every other cell in the document (crates/hickory-executor-sandbox),
which is where the actual boundary lives. What the restriction buys is a
smaller accident surface — a formula reaching for `open` is far more often a
typo than an intention.
"""

import json
import math
import statistics
import sys


def read_message(stream):
    """One LSP-framed message, or None at end of input."""
    length = None
    while True:
        line = stream.readline()
        if not line:
            return None
        line = line.strip()
        if not line:
            break
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":", 1)[1])
    if length is None:
        return None
    return json.loads(stream.read(length).decode("utf-8"))


def write_message(stream, payload):
    body = json.dumps(payload).encode("utf-8")
    stream.write(b"Content-Length: %d\r\n\r\n" % len(body))
    stream.write(body)
    stream.flush()


def to_python(value):
    kind = value.get("kind")
    if kind == "number":
        return value["value"]
    if kind == "text":
        return value["value"]
    if kind == "bool":
        return value["value"]
    # Empty is None: `sum(x for x in row if x is not None)` is how a person
    # skips blanks, and None is what makes that read naturally.
    return None


def to_wire(result):
    if result is None:
        return {"kind": "empty"}
    if isinstance(result, bool):
        return {"kind": "bool", "value": result}
    if isinstance(result, (int, float)):
        if isinstance(result, float) and (math.isnan(result) or math.isinf(result)):
            # NaN and infinity have no JSON spelling; text is honest and
            # visible, where `null` would look like an empty cell.
            return {"kind": "text", "value": str(result)}
        return {"kind": "number", "value": float(result)}
    return {"kind": "text", "value": str(result)}


# What a formula may reach for without importing anything. Chosen to be the
# things a table actually wants: arithmetic, aggregation, a little text.
SAFE = {
    "abs": abs,
    "all": all,
    "any": any,
    "bool": bool,
    "float": float,
    "int": int,
    "len": len,
    "max": max,
    "min": min,
    "round": round,
    "sorted": sorted,
    "str": str,
    "sum": sum,
    "math": math,
    "mean": statistics.mean,
    "median": statistics.median,
    "stdev": lambda xs: statistics.stdev(xs) if len(list(xs)) > 1 else 0.0,
}


def evaluate(formula):
    scope = {name: to_python(value) for name, value in formula.get("bindings", [])}
    try:
        value = eval(formula["expression"], {"__builtins__": {}, **SAFE}, scope)
        return {"id": formula["id"], "value": to_wire(value)}
    except Exception as error:  # noqa: BLE001 - the message IS the answer
        # The language's own message, not a spreadsheet's `#VALUE!`: a
        # NameError naming the thing that is missing is the only part the
        # author can act on.
        return {
            "id": formula["id"],
            "error": {"message": f"{type(error).__name__}: {error}"},
        }


def main():
    stdin = sys.stdin.buffer
    stdout = sys.stdout.buffer
    while True:
        message = read_message(stdin)
        if message is None:
            return
        method = message.get("method")
        if method == "initialize":
            write_message(
                stdout,
                {
                    "id": message.get("id"),
                    "result": {"language": "python", "version": sys.version.split()[0]},
                },
            )
        elif method == "evaluate":
            formulas = message.get("params", {}).get("formulas", [])
            write_message(
                stdout,
                {
                    "id": message.get("id"),
                    "result": {"results": [evaluate(f) for f in formulas]},
                },
            )
        elif method == "shutdown":
            write_message(stdout, {"id": message.get("id"), "result": {}})
            return
        elif message.get("id") is not None:
            write_message(
                stdout,
                {
                    "id": message["id"],
                    "error": {"code": -32601, "message": f"no method {method}"},
                },
            )


if __name__ == "__main__":
    main()
