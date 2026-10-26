<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/javascript_backend.mjs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bed493ca03c6"></a>
## `crates/hick-plugins/src/languages/javascript_backend.mjs`

<a id="f-bed493ca03c6-hash_bang_line"></a>
### `hash_bang_line`

<storycode:copy id="f-bed493ca03c6-hash_bang_line">
#!/usr/bin/env node
</storycode:copy>
<a id="f-bed493ca03c6-import_statement"></a>
### `import_statement`

A formula backend for JavaScript.
The twin of python_backend.py, and deliberately the same shape: read
LSP-framed JSON-RPC on stdin, evaluate one expression per formula with its
bindings in scope, write the results back. It knows nothing about cells,
sheets, dependencies or order — the host resolved all of that before asking
(crates/hick-formula/src/graph.rs).
Evaluation is `new Function`, with the bindings as parameters. That is not
a security boundary and is not pretended to be one: this runs under the
same executor policy as every other cell in the document. What it buys is
that a formula is an ordinary expression, so `A1 * 2` and
`rows.filter(r => r > 3).length` both mean what they look like.

<storycode:copy id="f-bed493ca03c6-import_statement">
// A formula backend for JavaScript.
//
// The twin of python_backend.py, and deliberately the same shape: read
// LSP-framed JSON-RPC on stdin, evaluate one expression per formula with its
// bindings in scope, write the results back. It knows nothing about cells,
// sheets, dependencies or order — the host resolved all of that before asking
// (crates/hick-formula/src/graph.rs).
//
// Evaluation is `new Function`, with the bindings as parameters. That is not
// a security boundary and is not pretended to be one: this runs under the
// same executor policy as every other cell in the document. What it buys is
// that a formula is an ordinary expression, so `A1 * 2` and
// `rows.filter(r => r > 3).length` both mean what they look like.

import process from "node:process";

</storycode:copy>
<a id="f-bed493ca03c6-decoder"></a>
### `decoder`

<storycode:copy id="f-bed493ca03c6-decoder">
const decoder = new TextDecoder();

</storycode:copy>
<a id="f-bed493ca03c6-toJs"></a>
### `toJs`

<storycode:copy id="f-bed493ca03c6-toJs">
function toJs(value) {
  switch (value.kind) {
    case "number":
    case "text":
    case "bool":
      return value.value;
    case "list":
      return value.value.map(toJs);
    default:
      // Empty is null: `xs.filter(x => x !== null)` is how a person skips
      // blanks, and null is what makes that read naturally.
      return null;
  }
}

</storycode:copy>
<a id="f-bed493ca03c6-toWire"></a>
### `toWire`

<storycode:copy id="f-bed493ca03c6-toWire">
function toWire(result) {
  if (result === null || result === undefined) return { kind: "empty" };
  if (typeof result === "boolean") return { kind: "bool", value: result };
  if (typeof result === "number") {
    // NaN and Infinity have no JSON spelling; text is honest and visible,
    // where null would look like an empty cell.
    if (!Number.isFinite(result)) return { kind: "text", value: String(result) };
    return { kind: "number", value: result };
  }
  return { kind: "text", value: String(result) };
}

</storycode:copy>
<a id="f-bed493ca03c6-HELPERS"></a>
### `HELPERS`

The helpers a table actually wants, without an import.

<storycode:copy id="f-bed493ca03c6-HELPERS">
/** The helpers a table actually wants, without an import. */
const HELPERS = {
  sum: (xs) => xs.reduce((a, b) => a + (Number(b) || 0), 0),
  mean: (xs) => (xs.length ? HELPERS.sum(xs) / xs.length : 0),
  min: (...xs) => Math.min(...xs.flat()),
  max: (...xs) => Math.max(...xs.flat()),
  round: (x, places = 0) => Number(Math.round(Number(`${x}e${places}`)) + `e-${places}`),
  Math,
};

</storycode:copy>
<a id="f-bed493ca03c6-evaluate"></a>
### `evaluate`

<storycode:copy id="f-bed493ca03c6-evaluate">
function evaluate(formula) {
  const names = formula.bindings.map(([name]) => name);
  const values = formula.bindings.map(([, value]) => toJs(value));
  const helperNames = Object.keys(HELPERS);
  try {
    // eslint-disable-next-line no-new-func
    const run = new Function(
      ...helperNames,
      ...names,
      `"use strict"; return (${formula.expression});`,
    );
    return { id: formula.id, value: toWire(run(...helperNames.map((n) => HELPERS[n]), ...values)) };
  } catch (error) {
    // The language's own message. A TypeError naming the thing that is
    // missing is the only part the author can act on.
    return { id: formula.id, error: { message: String(error && error.message ? `${error.name}: ${error.message}` : error) } };
  }
}

</storycode:copy>
<a id="f-bed493ca03c6-write"></a>
### `write`

<storycode:copy id="f-bed493ca03c6-write">
function write(payload) {
  const body = Buffer.from(JSON.stringify(payload), "utf8");
  process.stdout.write(`Content-Length: ${body.length}\r\n\r\n`);
  process.stdout.write(body);
}

</storycode:copy>
<a id="f-bed493ca03c6-buffer"></a>
### `buffer`

<storycode:copy id="f-bed493ca03c6-buffer">
let buffer = Buffer.alloc(0);

</storycode:copy>
<a id="f-bed493ca03c6-drain"></a>
### `drain`

<storycode:copy id="f-bed493ca03c6-drain">
function drain() {
  for (;;) {
    const header = buffer.indexOf("\r\n\r\n");
    if (header < 0) return;
    const head = decoder.decode(buffer.subarray(0, header));
    const match = /content-length:\s*(\d+)/i.exec(head);
    if (!match) {
      buffer = buffer.subarray(header + 4);
      continue;
    }
    const length = Number(match[1]);
    const start = header + 4;
    if (buffer.length < start + length) return;
    const message = JSON.parse(decoder.decode(buffer.subarray(start, start + length)));
    buffer = buffer.subarray(start + length);
    handle(message);
  }
}

</storycode:copy>
<a id="f-bed493ca03c6-handle"></a>
### `handle`

<storycode:copy id="f-bed493ca03c6-handle">
function handle(message) {
  if (message.method === "initialize") {
    write({ id: message.id, result: { language: "javascript", version: process.version } });
  } else if (message.method === "evaluate") {
    const formulas = message.params?.formulas ?? [];
    write({ id: message.id, result: { results: formulas.map(evaluate) } });
  } else if (message.method === "shutdown") {
    write({ id: message.id, result: {} });
    process.exit(0);
  } else if (message.id !== undefined && message.id !== null) {
    write({ id: message.id, error: { code: -32601, message: `no method ${message.method}` } });
  }
}

</storycode:copy>
<a id="f-bed493ca03c6-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-bed493ca03c6-expression_statement">
process.stdin.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  drain();
});
</storycode:copy>
<a id="f-bed493ca03c6-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-bed493ca03c6-expression_statement-2">
process.stdin.on("end", () => process.exit(0));
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/javascript_backend.mjs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/javascript_backend.mjs"><storycode:paste select="#f-bed493ca03c6-hash_bang_line"/><storycode:paste select="#f-bed493ca03c6-import_statement"/><storycode:paste select="#f-bed493ca03c6-decoder"/><storycode:paste select="#f-bed493ca03c6-toJs"/><storycode:paste select="#f-bed493ca03c6-toWire"/><storycode:paste select="#f-bed493ca03c6-HELPERS"/><storycode:paste select="#f-bed493ca03c6-evaluate"/><storycode:paste select="#f-bed493ca03c6-write"/><storycode:paste select="#f-bed493ca03c6-buffer"/><storycode:paste select="#f-bed493ca03c6-drain"/><storycode:paste select="#f-bed493ca03c6-handle"/><storycode:paste select="#f-bed493ca03c6-expression_statement"/><storycode:paste select="#f-bed493ca03c6-expression_statement-2"/></storycode:file>

</storycode:doc>
