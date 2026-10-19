import { expect, it, vi } from "vitest";
// @ts-expect-error Node runtime crypto is available to Vitest without browser Node typings.
import { webcrypto } from "node:crypto";
import { EditorState } from "@codemirror/state";
import { codeFolding, foldEffect } from "@codemirror/language";
import { projectDocument } from "./contextProjection";
import type { EditorView } from "@codemirror/view";
vi.stubGlobal("crypto", webcrypto);
// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
it("preserves Unicode source coordinates, duplicate passages and omitted ranges while pins bypass folds", async () => {
  const content = "α\nsame\nhidden\nsame";
  const state = EditorState.create({ doc: content, extensions: [codeFolding()] }).update({ effects: foldEffect.of({ from: 6, to: 13 }) }).state;
  const view = { state } as EditorView;
  const result = await projectDocument("unsaved", content, view);
  expect(result.segments).toEqual([
    { start: 0, end: 7, line_start: 1, line_end: 2, text: "α\nsame", omitted: false },
    { start: 7, end: 14, line_start: 2, line_end: 3, text: "", omitted: true },
    { start: 14, end: 19, line_start: 3, line_end: 4, text: "\nsame", omitted: false },
  ]);
  const pinned = await projectDocument("unsaved", content, view, true);
  expect(pinned.segments.map(s => s.text).join("")).toBe(content);
  expect(pinned.revision).toBe(result.revision);
  expect((await projectDocument("other", content)).id).toBe("other");
  expect((await projectDocument("unsaved", content + "edit")).revision).not.toBe(result.revision);
});

// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
it("retains fold intent across editor unmounts and refuses stale source coordinates", async () => {
  const { EditorView } = await import("@codemirror/view");
  const { rememberReadingState, restoreReadingState, retainedReadingState } = await import("./contextProjection");
  const content = "head\nhidden\ntail";
  const first = new EditorView({ state: EditorState.create({ doc: content, extensions: [codeFolding()] }) });
  first.dispatch({ effects: foldEffect.of({ from: 4, to: 11 }) });
  const original = await projectDocument("a", content, first);
  rememberReadingState("unique-document", first);
  first.destroy();
  expect(await projectDocument("a", content, retainedReadingState("unique-document"))).toEqual(original);
  const second = new EditorView({ state: EditorState.create({ doc: content, extensions: [codeFolding()] }) });
  try {
    restoreReadingState("unique-document", second);
    expect(await projectDocument("a", content, second)).toEqual(original);
    const stale = await projectDocument("a", "changed unsaved source", retainedReadingState("unique-document"));
    expect(stale.selection).toBe("unavailable");
    expect(stale.segments.every(s => s.omitted)).toBe(true);
  } finally { second.destroy(); }
});
