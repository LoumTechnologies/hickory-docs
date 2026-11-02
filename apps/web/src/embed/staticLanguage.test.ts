import { describe, it, expect } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { sha256 } from "../lib/reviews";
import { decodeStaticLanguage, staticLanguageClient, staticLanguageExtensions, type StaticLanguage } from "./staticLanguage";

async function fixture() {
  const source = "// 😀\nconst count = 1;\ncount";
  const data: StaticLanguage = { version: 1, source_hash: await sha256(source), state: "captured",
    spans: [{ from: 6, to: 11, class: "tok-keyword" }],
    language: { capabilities: {}, diagnostics: [], queries: {}, occurrences: [
      { from: 12, to: 17, position: { line: 1, character: 6 }, hover: { state: "captured", result: { contents: "**count**: number" } }, definition: { state: "captured", result: [{ uri: "hick:///project.md", range: { start: { line: 1, character: 6 }, end: { line: 1, character: 11 } } }] } },
    ] },
  };
  return { source, data };
}

describe("compiled read-only language information", () => {
  it("binds semantic data to both hashes and refuses invalid token classes and ranges", async () => {
    const { source, data } = await fixture();
    const hash = await sha256(JSON.stringify(data));
    await expect(decodeStaticLanguage(data, source, hash)).resolves.toEqual(data);
    await expect(decodeStaticLanguage(JSON.stringify(data), source, hash)).resolves.toEqual(data);
    await expect(decodeStaticLanguage(data, source + "changed", hash)).rejects.toThrow("Invalid");
    await expect(decodeStaticLanguage({ ...data, state: "unavailable" }, source, hash)).rejects.toThrow("hash");
    for (const spans of [[{ from: 0, to: source.length + 1, class: "tok-keyword" }], [{ from: 0, to: 2, class: "arbitrary-css" }]]) {
      const invalid = { ...data, spans };
      await expect(decodeStaticLanguage(invalid, source, await sha256(JSON.stringify(invalid)))).rejects.toThrow("Invalid");
    }
    await expect(decodeStaticLanguage(undefined, source)).resolves.toBeNull();
  });
  it("answers inside a captured UTF-16 identifier and refuses uncaptured and mutation requests", async () => {
    const { source, data } = await fixture();
    const client = staticLanguageClient(data, source);
    try {
      expect(await client.hover("hick:///project.md", { line: 1, character: 8 })).toEqual({ contents: "**count**: number" });
      await expect(client.hover("hick:///other.md", { line: 1, character: 8 })).rejects.toThrow("not captured");
      expect(await client.definition("hick:///project.md", { line: 1, character: 8 })).toHaveLength(1);
      await expect(client.request("textDocument/rename", { position: { line: 1, character: 8 } })).rejects.toThrow("not captured");
      await expect(client.hover("hick:///project.md", { line: 0, character: 0 })).rejects.toThrow("not captured");
    } finally { client.dispose(); }
  });
  it("paints compiled spans in a read-only view without changing the document", async () => {
    const { source, data } = await fixture();
    const host = document.createElement("div"); document.body.append(host);
    const view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [EditorState.readOnly.of(true), EditorView.editable.of(false), staticLanguageExtensions(data, source, "project.md", () => {}, () => {})] }) });
    try {
      await Promise.resolve();
      expect(host.querySelector(".tok-keyword")?.textContent).toBe("const");
      expect(view.state.readOnly).toBe(true);
      expect(view.state.doc.toString()).toBe(source);
    } finally { view.destroy(); host.remove(); }
  });
});
