<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Exercise scaffold editing boundaries

For maintainers preserving immutable scaffold captures while editing authored changes.

## editor tests

Check typing, programmatic replacements, hash edits, deletion and multi-edit atomic refusal with a rebound namespace and Unicode. Keep prose and patches editable, accept incoming room state, and prove generated-output locks follow earlier authored edits.

<storycode:copy id="immutable-scaffold-editor-tests">
import { representationOffset, representationRange } from "../lsp/representationMapping";
import { materializeProject } from "./hickLang";
import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import * as Y from "yjs";
import { YSyncConfig, ySyncFacet } from "y-codemirror.next";
import { capturedOutputRanges, immutableScaffolds, immutableScaffoldOutput, setScaffoldOutputRanges } from "./immutableScaffolds";

const source = 'Prose λ\n<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:exec>\n<h:ingested sha256="capture">\n<h:file path="main.rs">\nold\n</h:file>\n</h:ingested>\n</h:exec>\n<h:patch id="change">\n+new\n</h:patch>\n</h:doc>\n';

describe("immutable scaffold source", () => {
  it("refuses edits, rehashes, deletion and multi-edit batches while prose and patches remain editable", () => {
    const state = EditorState.create({ doc: source, extensions: [immutableScaffolds()] });
    const at = source.indexOf("old");
    for (const changes of [
      { from: at, to: at + 3, insert: "bad" },
      { from: at, insert: "bad" },
      { from: source.indexOf("capture"), to: source.indexOf("capture") + 7, insert: "rehash" },
      { from: source.indexOf("<h:ingested"), to: source.indexOf("</h:ingested>") + 13 },
      [{ from: 0, insert: "prose" }, { from: at, insert: "bad" }],
    ]) expect(state.update({ changes, userEvent: "input" }).newDoc.toString()).toBe(source);
    expect(state.update({ changes: { from: at, insert: "programmatic" } }).newDoc.toString()).toBe(source);
    expect(state.update({ changes: { from: 0, insert: "More prose.\n" } }).docChanged).toBe(true);
    expect(state.update({ changes: { from: 0, to: source.length, insert: source + "More prose.\n" } }).docChanged).toBe(true);
    // Namespace edits or wrapping the base in a raw fence must not unlock it.
    expect(state.update({ changes: { from: source.indexOf("xmlns:h"), to: source.indexOf("xmlns:h") + 7, insert: "xmlns:z" } }).docChanged).toBe(false);
    expect(state.update({ changes: [{ from: 0, insert: "```text\n" }, { from: source.length, insert: "```\n" }] }).docChanged).toBe(false);
    const patch = source.indexOf("+new");
    expect(state.update({ changes: { from: patch, to: patch + 4, insert: "+authored" } }).newDoc.toString()).toContain("+authored");
  });

  it("receives authoritative room refreshes without treating them as local edits", () => {
    const doc = new Y.Doc(); const text = doc.getText("source"); text.insert(0, source);
    const config = new YSyncConfig(text, null);
    const state = EditorState.create({ doc: source, extensions: [ySyncFacet.of(config), immutableScaffolds()] });
    const at = source.indexOf("old"); text.delete(at, 3); text.insert(at, "refreshed");
    const update = state.update({ changes: { from: at, to: at + 3, insert: "refreshed" } });
    expect(update.newDoc.toString()).toBe(text.toString()); doc.destroy();
  });
});

describe("immutable generated scaffold spans", () => {
  it("maps Unicode wire bytes, protects the base after authored edits, and permits trusted refresh", () => {
    const file = { path: "main.rs", language: "rust", content: "λ\nbase\n", provenance: [
      { start: 3, end: 8, origin: { kind: "ingested" as const, doc_path: "p.md", span: [0, 5] as [number, number], run: "hash" } },
    ] };
    let state = EditorState.create({ doc: file.content, extensions: [immutableScaffoldOutput(capturedOutputRanges(file))] });
    expect(state.update({ changes: { from: 2, to: 6, insert: "bad" } }).docChanged).toBe(false);
    state = state.update({ changes: { from: 0, to: 1, insert: "authored" } }).state;
    const at = state.doc.toString().indexOf("base");
    expect(state.update({ changes: { from: at + 1, insert: "bad" } }).docChanged).toBe(false);
    expect(state.update({ changes: { from: 0, insert: "patch" } }).docChanged).toBe(true);
    state = state.update({ effects: setScaffoldOutputRanges.of([]) }).state;
    expect(state.update({ changes: { from: at, to: at + 4, insert: "refresh" } }).docChanged).toBe(true);
  });
});

it("browser reconstruction verifies the captured hash before applying an authored patch", () => {
  const base = `<hick:project/>
<hick:exec id="base" container="sdk">
<hick:ingested sha256="4706a8d5a4ce67560846b3812815e532aab269f52331850ce24e0bc600afd6d9">
<hick:file path="x">
old
</hick:file>
</hick:ingested>
</hick:exec>
`;
  const patch = '<hick:patch after="base">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new\n</hick:patch>';
  const result = materializeProject("project.md", { "project.md": base + patch });
  expect(new TextDecoder().decode(new Uint8Array(result.new[0].bytes))).toBe("new\n");
  expect(() => materializeProject("project.md", { "project.md": base.replace("old\n", "tampered\n") + patch })).toThrow(/hash mismatch/);
});

it("LSP can navigate to a capture but cannot use it as an edit destination", () => {
  const file = { path: "x", hash: "", content: "base", provenance: [
    { start: 0, end: 4, origin: { kind: "ingested" as const, doc_path: "p.md", span: [0, 4] as [number, number], run: "hash" } },
  ] };
  expect(representationOffset("base", file, { line: 0, character: 2 })).toBe(2);
  expect(representationRange("base", file, { start: { line: 0, character: 0 }, end: { line: 0, character: 2 } })).toBeNull();
  expect(representationRange("base", file, { start: { line: 0, character: 2 }, end: { line: 0, character: 2 } })).toBeNull();
});
</storycode:copy>

<storycode:file path="apps/web/src/editor/immutableScaffolds.test.ts"><storycode:paste select="#immutable-scaffold-editor-tests"/></storycode:file>
</storycode:doc>
