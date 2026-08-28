#!/usr/bin/env node
// `hick-model-typescript <root>` — the TypeScript code model, served as GraphQL.
//
// One JSON object per line on stdin, one per line on stdout. Diagnostics go to
// stderr; stdout carries the protocol and nothing else.

import { graphql } from "graphql";
import readline from "node:readline";
import fs from "node:fs";
import { CodeModel } from "./model.js";
import { buildSchema } from "./schema.js";

const root = process.argv[2] ?? process.cwd();
if (!fs.existsSync(root) || !fs.statSync(root).isDirectory()) {
  console.error(`hick-model-typescript: no such directory: ${root}`);
  console.error("  Pass the source root to model, e.g. `hick-model-typescript src`.");
  process.exit(2);
}

let model;
try {
  model = new CodeModel(root);
} catch (e) {
  console.error(`hick-model-typescript: could not read ${root}: ${e.message}`);
  process.exit(2);
}
const schema = buildSchema(model);
console.error(`hick-model-typescript: ${model.info().fileCount} file(s) under ${model.info().root}; ready`);
for (const gap of model.unresolved().slice(0, 5)) console.error(`  unresolved: ${gap.message}`);

const lines = readline.createInterface({ input: process.stdin, terminal: false });
for await (const line of lines) {
  if (!line.trim()) continue;
  let body;
  try {
    const request = JSON.parse(line);
    const result = await graphql({
      schema,
      source: request.query ?? "",
      variableValues: request.variables ?? undefined,
    });
    body = JSON.stringify(result);
  } catch (e) {
    // A malformed request must not end the session: the caller is a generator
    // being written, and being written means getting it wrong.
    body = JSON.stringify({ errors: [{ message: e.message }] });
  }
  process.stdout.write(body.replace(/\n/g, " ") + "\n");
}
