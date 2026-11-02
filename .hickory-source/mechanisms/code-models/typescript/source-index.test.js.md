<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Check binding identity before publishing navigation

A fast index is useful only if navigation preserves compiler binding. Construct a complete project with generic properties, aliases, a shorthand that denotes two symbols, unrelated same-spelling properties and non-BMP text. Assert relationships by source offsets, not traversal order or generated identity strings.

<static:copy id="static-code-models-typescript-source-index-test-js-0">
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { graphql } from "graphql";
import { CodeModel } from "./model.js";
import { buildSchema } from "./schema.js";

const types = `/** A generic box. */
export interface Box<T> { value: T }
/** Preserve the input value. */
export function identity<T>(value: T): T { return value; }
export const box: Box<number> = { value: identity(42) };
export function read(box: Box<number>) { const value = box.value; return identity(value); }
export function shadow(value: string) { return identity(value); }
`;
const consumer = `import { identity as id, box, Box } from "./types";
const emoji = "😀";
const value = 42;
const shorthand: Box<number> = { value };
const other = { value: "other" };
id(box.value); other.value; shorthand["value"];
`;
function fixture(run) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "hick-source-index-"));
  try {
    fs.writeFileSync(path.join(root, "types.ts"), types);
    fs.writeFileSync(path.join(root, "consumer.ts"), consumer);
    fs.writeFileSync(path.join(root, "tsconfig.json"), JSON.stringify({ compilerOptions: { strict: true, target: "ES2022", module: "ESNext", moduleResolution: "node", noEmit: true } }));
    return run(new CodeModel(root), root);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
}
const occurrence = (index, file, text, needle, shift = 0) => {
  const at = text.indexOf(needle) + shift;
  assert.ok(at >= 0);
  const found = index.files.find(item => item.path === file).occurrences.find(item => item.from === at);
  assert.ok(found, `Missing occurrence ${file}:${needle}+${shift}`);
  return found;
};

test("generic, contextual and indexed properties share their declaration; unrelated properties stay separate", () => fixture(model => {
  const index = model.sourceIndex();
  const declared = occurrence(index, "types.ts", types, "value: T").symbols[0];
  for (const [file, text, needle, shift] of [["types.ts", types, "value: identity", 0], ["types.ts", types, "box.value", 4], ["consumer.ts", consumer, "box.value", 4], ["consumer.ts", consumer, '"value"]', 0]]) {
    assert.ok(occurrence(index, file, text, needle, shift).symbols.includes(declared));
  }
  assert.ok(!occurrence(index, "consumer.ts", consumer, "other.value", 6).symbols.includes(declared));
  const symbol = index.symbols.find(item => item.id === declared);
  assert.deepEqual(symbol.declarations, [{ file: "types.ts", from: types.indexOf("value: T"), to: types.indexOf("value: T") + 5 }]);
}));

test("import aliases resolve to the original symbol and shorthand retains both value and property", () => fixture(model => {
  const index = model.sourceIndex();
  const identity = occurrence(index, "types.ts", types, "identity<T>").symbols[0];
  assert.ok(occurrence(index, "consumer.ts", consumer, "id(box").symbols.includes(identity));
  const shorthand = occurrence(index, "consumer.ts", consumer, "{ value };", 2);
  assert.equal(shorthand.symbols.length, 2);
  const local = occurrence(index, "consumer.ts", consumer, "value = 42").symbols[0];
  assert.ok(shorthand.symbols.includes(local));
  const parameters = [occurrence(index, "types.ts", types, "value: T):").symbols[0], occurrence(index, "types.ts", types, "value: string").symbols[0]];
  assert.notEqual(parameters[0], parameters[1]);
  assert.ok(!parameters.includes(local));
}));

test("the cached index retains UTF-16, documentation, contextual types, diagnostics and document features", () => fixture((model, root) => {
  fs.writeFileSync(path.join(root, "missing.ts"), 'import { absent } from "./missing-module"; absent();');
  model.project.addSourceFileAtPath(path.join(root, "missing.ts"));
  model.files = model.project.getSourceFiles().filter(file => !file.getFilePath().includes("/node_modules/"));
  const index = model.sourceIndex();
  assert.strictEqual(model.sourceIndex(), index);
  assert.equal(index.coordinateEncoding, "utf-16");
  assert.equal(occurrence(index, "consumer.ts", consumer, "box.value", 4).type, "number");
  assert.equal(occurrence(index, "consumer.ts", consumer, "other.value", 6).type, "string");
  assert.ok(index.symbols.some(symbol => symbol.documentation === "Preserve the input value."));
  assert.ok(index.diagnostics.some(item => item.code === 2307 && item.file === "missing.ts"));
  const file = index.files.find(item => item.path === "consumer.ts");
  assert.equal(file.length, consumer.length);
  assert.ok(file.classifications.length > 0);
  assert.ok(file.calls.some(call => call.signature.includes("number")));
  assert.ok(index.files.some(file => file.folds.length > 0));
  assert.ok(!JSON.stringify(index).includes(root));
}));

test("private members retain authored names and stable identities", () => fixture((model, root) => {
  const text = 'export class Secret { #value = 1; read() { return this.#value; } }';
  fs.writeFileSync(path.join(root, "private.ts"), text);
  model.project.addSourceFileAtPath(path.join(root, "private.ts"));
  model.files = model.project.getSourceFiles().filter(file => !file.getFilePath().includes("/node_modules/"));
  const index = model.sourceIndex();
  const field = index.symbols.find(symbol => symbol.name === "#value");
  assert.ok(field);
  assert.equal(index.files.find(file => file.path === "private.ts").occurrences.filter(item => item.symbols.includes(field.id)).length, 2);
  assert.ok(!index.symbols.some(symbol => symbol.name.includes("__#")));
}));

test("BOM, CRLF and non-BMP text keep the indexed file identity and UTF-16 offsets", () => fixture((model, root) => {
  const text = '\ufeffexport const café = "😀";\r\ncafé;\r\n';
  fs.writeFileSync(path.join(root, "unicode.ts"), text);
  model.project.addSourceFileAtPath(path.join(root, "unicode.ts"));
  model.files = model.project.getSourceFiles().filter(file => !file.getFilePath().includes("/node_modules/"));
  const file = model.sourceIndex().files.find(file => file.path === "unicode.ts");
  assert.equal(file.sha256, createHash("sha256").update(text).digest("hex"));
  assert.equal(file.length, text.length);
  assert.ok(file.occurrences.some(item => item.from === text.lastIndexOf("café") && text.slice(item.from, item.to) === "café"));
}));

test("GraphQL exposes a finite source index independently of recursive type declarations", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "hick-index-schema-"));
  try {
    fs.writeFileSync(path.join(root, "types.ts"), types);
    const result = await graphql({ schema: buildSchema(new CodeModel(root)), source: "{ sourceIndex { version coordinateEncoding files { path occurrences { from to symbols type } } symbols { id declarations { file from to } } } }" });
    assert.equal(result.errors, undefined);
    assert.equal(result.data.sourceIndex.version, 1);
    assert.ok(result.data.sourceIndex.files[0].occurrences.length > 20);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});
</static:copy>

<static:file path="code-models/typescript/source-index.test.js"><static:paste select="#static-code-models-typescript-source-index-test-js-0"/></static:file>
</static:doc>
