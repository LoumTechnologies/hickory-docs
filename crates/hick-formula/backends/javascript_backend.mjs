#!/usr/bin/env node
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

const decoder = new TextDecoder();

function toJs(value) {
  switch (value.kind) {
    case "number":
    case "text":
    case "bool":
      return value.value;
    default:
      // Empty is null: `xs.filter(x => x !== null)` is how a person skips
      // blanks, and null is what makes that read naturally.
      return null;
  }
}

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

/** The helpers a table actually wants, without an import. */
const HELPERS = {
  sum: (xs) => xs.reduce((a, b) => a + (Number(b) || 0), 0),
  mean: (xs) => (xs.length ? HELPERS.sum(xs) / xs.length : 0),
  min: (...xs) => Math.min(...xs.flat()),
  max: (...xs) => Math.max(...xs.flat()),
  round: (x, places = 0) => Number(Math.round(Number(`${x}e${places}`)) + `e-${places}`),
  Math,
};

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

function write(payload) {
  const body = Buffer.from(JSON.stringify(payload), "utf8");
  process.stdout.write(`Content-Length: ${body.length}\r\n\r\n`);
  process.stdout.write(body);
}

let buffer = Buffer.alloc(0);

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

process.stdin.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  drain();
});
process.stdin.on("end", () => process.exit(0));
