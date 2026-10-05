import { act, render, waitFor, cleanup } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { EditorView } from "@codemirror/view";
import { DocumentEmbed } from "./DocumentEmbed";
import { createMemoryStorage, exportWorkspace, decodeWorkspaceExport, importWorkspace, searchWorkspace, StorageConflict } from "./storage";
import { materializeLiteralFiles } from "../editor/hickLang";
// Guarantees: docs/guarantees/embedding/an-embed-has-no-implicit-host.md
// docs/guarantees/embedding/literal-files-use-native-bytes.md

afterEach(cleanup);
const fixtures = [
  "# Bare markdown\n\n🦀 & < >\n",
  "---\ntitle: A note\n---\n# Frontmatter\n",
  '<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="a.js">var n = 1;</h:file></h:doc>',
  '<hick:copy id="x"><hick:example /> & raw</hick:copy>\n<hick:file path="a"><hick:paste select="#x" /></hick:file>',
  '<hick:include path="missing.md"/>\n<hick:when test="local">conditional</hick:when>',
  '<hick:transform select="#x" from="old" instruct="summarize">recorded summary</hick:transform>',
  '<hick:session><hick:turn role="user">A request</hick:turn></hick:session>',
  '<hick:container name="c"/><hick:exec container="c">echo never-run</hick:exec>',
  '<hick:future unknown="true">unknown bytes 🦀</hick:future>',
];
describe("portable embedding", () => {
  it.each(fixtures)("opens source losslessly without host requests: %s", async (source) => {
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("No host"));
    const onChange = vi.fn(); let view: EditorView | null = null;
    render(<DocumentEmbed path="note.md" source={source} revision="1" onChange={onChange} onViewReady={(next) => { view = next; }} />);
    await waitFor(() => expect(view).not.toBeNull());
    expect(view!.state.doc.toString()).toBe(source); expect(onChange).not.toHaveBeenCalled(); expect(fetchSpy).not.toHaveBeenCalled();
    fetchSpy.mockRestore();
  });
  it("isolates instances, suppresses host-update echoes, reports selection and disposes", async () => {
    const onChange = vi.fn(), selection = vi.fn(); let left: EditorView | null = null, right: EditorView | null = null;
    const props = { path: "a.md", source: "a", revision: "1", onChange, onSelection: selection, onViewReady: (v: EditorView | null) => { left = v; } };
    const host = render(<><DocumentEmbed {...props} /><DocumentEmbed path="b.md" source="b" revision="1" onViewReady={(v) => { right = v; }} /></>);
    await waitFor(() => expect(left).not.toBeNull());
    act(() => left!.dispatch({ changes: { from: 1, insert: "x" }, selection: { anchor: 2 } }));
    expect(onChange).toHaveBeenCalledWith({ path: "a.md", baseRevision: "1", source: "ax" }); expect(right!.state.doc.toString()).toBe("b"); expect(selection).toHaveBeenCalled();
    onChange.mockClear();
    host.rerender(<DocumentEmbed {...props} source="host" revision="2" />);
    expect(left!.state.doc.toString()).toBe("host"); expect(onChange).not.toHaveBeenCalled();
    const oldView = left!; host.unmount(); expect(left).toBeNull(); expect(oldView.dom.isConnected).toBe(false);
  });
  it("WASM materialization retains exact output/source bytes", () => {
    const source = '\ufeff# 🦀\n<hick:file path="a.js">\nvar text = "é";\n</hick:file>';
    const file = materializeLiteralFiles(source)[0];
    const bytes = new TextEncoder().encode(source), output = new TextEncoder().encode(file.content);
    for (const segment of file.segments) expect(output.slice(...segment.output)).toEqual(bytes.slice(...segment.source));
  });
  it("conflicts retain prior bytes and exports preserve assets and evidence", async () => {
    const storage = createMemoryStorage(); const bytes = new TextEncoder().encode("# 🦀 source\n");
    const first = await storage.write("note.md", bytes, null);
    await expect(storage.write("note.md", new Uint8Array([0]), null)).rejects.toBeInstanceOf(StorageConflict);
    expect((await storage.read("note.md"))?.bytes).toEqual(bytes);
    await storage.write("assets/file.bin", new Uint8Array([0, 255, 11]), null);
    await storage.write(".hick-cache/evidence.json", new TextEncoder().encode('{"run":"old"}'), null);
    const imported = decodeWorkspaceExport(await exportWorkspace(storage));
    expect(imported.find((file) => file.path === "assets/file.bin")?.bytes).toEqual(new Uint8Array([0, 255, 11]));
    await expect(storage.delete("note.md", "wrong")).rejects.toBeInstanceOf(StorageConflict);
    await storage.delete("note.md", first.revision); expect(await storage.read("note.md")).toBeNull();
  });
  it("imports source, assets and evidence atomically, refusing any existing path", async () => {
    const source = createMemoryStorage(), target = createMemoryStorage();
    for (const [path, bytes] of [["notes/meeting.md", new TextEncoder().encode("# Meeting\nA finding 🦀\n")], ["notes/image.bin", new Uint8Array([0, 255])], [".hick-cache/result", new Uint8Array([123, 125])]] as const) await source.write(path, bytes, null);
    const bundle = await exportWorkspace(source);
    await importWorkspace(target, bundle);
    expect(await exportWorkspace(target)).toEqual(bundle);
    const empty = createMemoryStorage();
    await empty.write(".hick-cache/result", new Uint8Array([42]), null);
    await expect(importWorkspace(empty, bundle)).rejects.toBeInstanceOf(StorageConflict);
    expect(await empty.list()).toEqual([".hick-cache/result"]);
    expect(await searchWorkspace(target, "FINDING")).toEqual([{ path: "notes/meeting.md", line: 2, text: "A finding 🦀" }]);
  });
  it("a conflicting batch cannot partly delete or rename files", async () => {
    const storage = createMemoryStorage();
    const original = await storage.write("original.md", new Uint8Array([1]), null);
    await storage.write("occupied.md", new Uint8Array([2]), null);
    await expect(storage.batch([{ path: original.path, delete: true, expected: original.revision }, { path: "occupied.md", bytes: original.bytes, expected: null }])).rejects.toBeInstanceOf(StorageConflict);
    expect((await storage.read(original.path))?.bytes).toEqual(original.bytes);
  });
});
