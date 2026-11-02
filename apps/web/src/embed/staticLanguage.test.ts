import { describe, it, expect } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { sha256 } from "../lib/reviews";
import { decodeStaticLanguage, staticLanguageClient, staticLanguageExtensions, type StaticLanguage } from "./staticLanguage";

async function fixture(newline = "\n") {
  const source = "// 😀\nconst count = 1;\ncount".replaceAll("\n", newline);
  const keyword = source.indexOf("const"), count = source.indexOf("count");
  const data: StaticLanguage = { version: 1, source_hash: await sha256(source), state: "captured",
    spans: [{ from: keyword, to: keyword+5, class: "tok-keyword" }],
    language: { capabilities: {}, diagnostics: [], queries: {}, occurrences: [
      { from: count, to: count+5, position: { line: 1, character: 6 }, hover: { state: "captured", result: { contents: "**count**: number" } }, definition: { state: "captured", result: [{ uri: "hick:///project.md", range: { start: { line: 1, character: 6 }, end: { line: 1, character: 11 } } }] } },
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
  it("expands packed tokens without losing ranges and refuses malformed encodings", async () => {
    const { source, data } = await fixture();
    const packed = { ...data, spans: [], packed_spans: { classes: ["tok-keyword"], data: [6, 5, 0] } };
    const decoded = await decodeStaticLanguage(JSON.stringify(packed), source, await sha256(JSON.stringify(packed)));
    expect(decoded?.spans).toEqual(data.spans);
    for (const encoding of [
      { classes: ["arbitrary-css"], data: [6, 5, 0] },
      { classes: ["tok-keyword"], data: [6, 5, 1] },
      { classes: ["tok-keyword"], data: [6, 5] },
      { classes: ["tok-keyword"], data: [6, 0, 0] },
    ]) {
      const invalid = { ...packed, packed_spans: encoding };
      await expect(decodeStaticLanguage(JSON.stringify(invalid), source, await sha256(JSON.stringify(invalid)))).rejects.toThrow("Invalid");
    }
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
  it("resolves compact model symbol tables locally and filters document highlights", async () => {
    const { source, data } = await fixture();
    const target = { uri: "hick:///project.md", range: { start: { line: 1, character: 6 }, end: { line: 1, character: 11 } } };
    const other = { ...target, uri: "hick:///chapter.md" };
    data.language!.occurrences[0].symbolIds = ["first", "second"];
    data.language!.shared = { first: { definition: { state: "captured", result: [target] }, references: { state: "captured", result: [target, other] } }, second: { definition: { state: "captured", result: [target] }, references: { state: "captured", result: [other] } } };
    const client = staticLanguageClient(data, source);
    try {
      expect(await client.definition("hick:///project.md", { line: 1, character: 8 })).toEqual([target]);
      expect(await client.references("hick:///project.md", { line: 1, character: 8 })).toEqual([target, other]);
      const highlights = await client.request("textDocument/documentHighlight", { textDocument: { uri: "hick:///project.md" }, position: { line: 1, character: 8 } });
      expect(highlights).toEqual([{ range: target.range, kind: 2 }]);
    } finally { client.dispose(); }
  });
  it("keeps model hover blocks flat when adding captured call signatures", async () => {
    const { source, data } = await fixture();
    const occurrence = data.language!.occurrences[0];
    occurrence.hover = { state: "captured", result: { contents: [{ language: "typescript", value: "count: number" }, "Documentation"] } };
    occurrence.signatureHelp = { state: "captured", result: { signatures: [{ label: "read(count: number): number" }] } };
    const client = staticLanguageClient(data, source);
    try {
      expect((await client.hover("hick:///project.md", { line: 1, character: 8 }))?.contents).toEqual([{ language: "typescript", value: "count: number" }, "Documentation", { value: "read(count: number): number" }]);
    } finally { client.dispose(); }
  });
  it("maps CRLF and CR storage offsets onto the normalized read-only editor", async () => {
    for (const newline of ["\r\n", "\r"]) {
      const { source, data } = await fixture(newline);
      const host = document.body.appendChild(document.createElement("div"));
      const editor = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [EditorState.readOnly.of(true), staticLanguageExtensions(data, source, "project.md", () => {}, () => {})] }) });
      const client = staticLanguageClient(data, source);
      try {
        expect(host.querySelector(".tok-keyword")?.textContent).toBe("const");
        expect(editor.state.doc.toString()).toBe(source.replace(/\r\n?/g, "\n"));
        expect(await client.hover("hick:///project.md", { line: 1, character: 8 })).toEqual({ contents: "**count**: number" });
        expect(editor.state.readOnly).toBe(true);
      } finally { client.dispose(); editor.destroy(); host.remove(); }
    }
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
