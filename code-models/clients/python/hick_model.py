"""Talking to a hick code model server, so a generator does not have to.

Every generator written against a model server was re-implementing the same
forty lines: spawn the process, frame the JSON, notice that GraphQL reports
failure inside a successful-looking body, and then navigate untyped
dictionaries. Measured on the warehouse's generator, that plumbing was 36% of
the file and the rules it existed to serve were 9%.

None of it is language-specific, and none of it is the interesting part.

    with Model("hick-model-csharp", "src/Domain") as model:
        for service in model.q(TYPES).types:
            for method in service.methods:
                print(method.name, method.returnType.fullName)

Deliberately NOT a typed client generated from the schema. That would need a
code generator per language, which is the thing this library exists to make
unnecessary — and attribute access over the response gives most of the
benefit, because the schema is already the documentation and a typo raises
rather than returning None.
"""

from __future__ import annotations

import json
import subprocess
from typing import Any

__all__ = ["Model", "ModelError", "Node", "Emit"]


class ModelError(RuntimeError):
    """The server refused, or could not be reached."""


class Node:
    """A response, reachable with dots.

    `t.properties[0].type.fullName` rather than
    `t["properties"][0]["type"]["fullName"]` — which matters less for length
    than for the failure mode: a mistyped field raises here and returns `None`
    from a dictionary, and a generator that silently reads `None` emits
    silently wrong code.
    """

    __slots__ = ("_data", "_cache")

    def __init__(self, data: dict[str, Any]):
        object.__setattr__(self, "_data", data)
        object.__setattr__(self, "_cache", {})

    # Two nodes over the same response are the SAME node.
    #
    # Without this, `wrap` handed back a fresh object on every access, so
    # `param in route_params` compared identities that could never match and
    # every parameter looked like a body parameter. It generated a handler
    # taking `string sku, string sku`. Caught by diffing against the previous
    # generator's output, which is the only reason it is not still there.
    def __eq__(self, other: object) -> bool:
        return isinstance(other, Node) and other._data is self._data

    def __hash__(self) -> int:
        return id(self._data)

    def __getattr__(self, name: str) -> Any:
        cache = object.__getattribute__(self, "_cache")
        if name in cache:
            return cache[name]
        try:
            value = wrap(self._data[name])
        except KeyError:
            raise AttributeError(
                f"no field `{name}` in this response — the schema has "
                f"{', '.join(sorted(self._data)) or '(nothing)'}. "
                f"Run the server with no query to print what can be asked."
            ) from None
        cache[name] = value
        return value

    def get(self, name: str, default: Any = None) -> Any:
        return wrap(self._data.get(name, default))

    def __contains__(self, name: str) -> bool:
        return name in self._data

    def __repr__(self) -> str:
        return f"Node({', '.join(sorted(self._data))})"


def wrap(value: Any) -> Any:
    if isinstance(value, dict):
        return Node(value)
    if isinstance(value, list):
        return [wrap(v) for v in value]
    return value


class Model:
    """One server, held open across a generation pass.

    Held open because binding a compilation is the entire cost — 2.3s for
    Roslyn on a small project against 0.3ms per query afterwards — so a
    generator that spawns per question pays it per question.
    """

    def __init__(self, binary: str, root: str, timeout: float = 300.0):
        self.binary, self.root, self.timeout = binary, root, timeout
        self._proc: subprocess.Popen[str] | None = None

    def __enter__(self) -> "Model":
        try:
            self._proc = subprocess.Popen(
                [self.binary, self.root],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                text=True, bufsize=1,
            )
        except FileNotFoundError:
            raise ModelError(
                f"no model server at `{self.binary}`.\n"
                f"  `hick lang` shows which languages have one, and a document "
                f"that uses one should declare it with a needs element so this "
                f"is caught before the cell runs rather than inside the script."
            ) from None
        return self

    def __exit__(self, *_exc: object) -> None:
        if self._proc and self._proc.stdin:
            self._proc.stdin.close()
        if self._proc:
            self._proc.wait(timeout=10)

    def q(self, query: str, **variables: Any) -> Node:
        """Ask one question. Raises on refusal rather than returning empty."""
        if not self._proc or not self._proc.stdin or not self._proc.stdout:
            raise ModelError("the model server is not running — use `with Model(…)`")
        # Annotated because the values are not all strings: `variables` is a
        # nested object, and an inferred `dict[str, str]` makes assigning it an
        # error in the generated client's own type check.
        request: dict[str, Any] = {"query": query}
        if variables:
            request["variables"] = variables
        self._proc.stdin.write(json.dumps(request) + "\n")
        self._proc.stdin.flush()
        line = self._proc.stdout.readline()
        if not line:
            raise ModelError("the model server closed without answering")
        answer = json.loads(line)
        # GraphQL reports failure inside a 200-shaped body, so an unchecked
        # caller treats "you asked for a field that does not exist" as an
        # empty result and generates nothing, silently.
        if answer.get("errors"):
            raise ModelError(
                "the model refused the query:\n  "
                + "\n  ".join(e.get("message", str(e)) for e in answer["errors"])
            )
        return Node(answer["data"])


class Emit:
    """A buffer for building source text.

    The other third of a generator, and the third nobody enjoys: appending to
    a list and remembering where the commas go. `join` exists because a
    parameter list is the single most common thing a generator emits and
    getting its separators right by hand is where the fiddly bugs live.
    """

    def __init__(self) -> None:
        self._lines: list[str] = []

    def __call__(self, *lines: str) -> "Emit":
        self._lines.extend(lines)
        return self

    def join(self, parts: list[str], sep: str = ",\n") -> "Emit":
        """One entry per line with `sep` between — a parameter list."""
        if parts:
            self._lines.append(sep.join(parts))
        return self

    def blank(self) -> "Emit":
        self._lines.append("")
        return self

    def text(self) -> str:
        return "\n".join(self._lines) + "\n"

    def write(self, path: str) -> None:
        import os
        os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
        with open(path, "w") as handle:
            handle.write(self.text())
