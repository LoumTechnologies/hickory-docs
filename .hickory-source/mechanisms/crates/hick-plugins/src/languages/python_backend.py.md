<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/python_backend.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1cf5c9b5ba43"></a>
## `crates/hick-plugins/src/languages/python_backend.py`

<a id="f-1cf5c9b5ba43-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-1cf5c9b5ba43-expression_statement">
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

</storycode:copy>
<a id="f-1cf5c9b5ba43-json"></a>
### `json`

<storycode:copy id="f-1cf5c9b5ba43-json">
import json
</storycode:copy>
<a id="f-1cf5c9b5ba43-math"></a>
### `math`

<storycode:copy id="f-1cf5c9b5ba43-math">
import math
</storycode:copy>
<a id="f-1cf5c9b5ba43-statistics"></a>
### `statistics`

<storycode:copy id="f-1cf5c9b5ba43-statistics">
import statistics
</storycode:copy>
<a id="f-1cf5c9b5ba43-sys"></a>
### `sys`

<storycode:copy id="f-1cf5c9b5ba43-sys">
import sys


</storycode:copy>
<a id="f-1cf5c9b5ba43-read_message"></a>
### `read_message`

<storycode:copy id="f-1cf5c9b5ba43-read_message">
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


</storycode:copy>
<a id="f-1cf5c9b5ba43-write_message"></a>
### `write_message`

<storycode:copy id="f-1cf5c9b5ba43-write_message">
def write_message(stream, payload):
    body = json.dumps(payload).encode("utf-8")
    stream.write(b"Content-Length: %d\r\n\r\n" % len(body))
    stream.write(body)
    stream.flush()


</storycode:copy>
<a id="f-1cf5c9b5ba43-to_python"></a>
### `to_python`

<storycode:copy id="f-1cf5c9b5ba43-to_python">
def to_python(value):
    kind = value.get("kind")
    if kind == "number":
        return value["value"]
    if kind == "text":
        return value["value"]
    if kind == "bool":
        return value["value"]
    if kind == "list":
        return [to_python(item) for item in value["value"]]
    # Empty is None: `sum(x for x in row if x is not None)` is how a person
    # skips blanks, and None is what makes that read naturally.
    return None


</storycode:copy>
<a id="f-1cf5c9b5ba43-to_wire"></a>
### `to_wire`

<storycode:copy id="f-1cf5c9b5ba43-to_wire">
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


</storycode:copy>
<a id="f-1cf5c9b5ba43-expression_statement-2"></a>
### `expression_statement`

What a formula may reach for without importing anything. Chosen to be the
things a table actually wants: arithmetic, aggregation, a little text.

<storycode:copy id="f-1cf5c9b5ba43-expression_statement-2">
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


</storycode:copy>
<a id="f-1cf5c9b5ba43-evaluate"></a>
### `evaluate`

<storycode:copy id="f-1cf5c9b5ba43-evaluate">
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


</storycode:copy>
<a id="f-1cf5c9b5ba43-main"></a>
### `main`

<storycode:copy id="f-1cf5c9b5ba43-main">
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


</storycode:copy>
<a id="f-1cf5c9b5ba43-if_statement"></a>
### `if_statement`

<storycode:copy id="f-1cf5c9b5ba43-if_statement">
if __name__ == "__main__":
    main()
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/python_backend.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/python_backend.py"><storycode:paste select="#f-1cf5c9b5ba43-expression_statement"/><storycode:paste select="#f-1cf5c9b5ba43-json"/><storycode:paste select="#f-1cf5c9b5ba43-math"/><storycode:paste select="#f-1cf5c9b5ba43-statistics"/><storycode:paste select="#f-1cf5c9b5ba43-sys"/><storycode:paste select="#f-1cf5c9b5ba43-read_message"/><storycode:paste select="#f-1cf5c9b5ba43-write_message"/><storycode:paste select="#f-1cf5c9b5ba43-to_python"/><storycode:paste select="#f-1cf5c9b5ba43-to_wire"/><storycode:paste select="#f-1cf5c9b5ba43-expression_statement-2"/><storycode:paste select="#f-1cf5c9b5ba43-evaluate"/><storycode:paste select="#f-1cf5c9b5ba43-main"/><storycode:paste select="#f-1cf5c9b5ba43-if_statement"/></storycode:file>

</storycode:doc>
