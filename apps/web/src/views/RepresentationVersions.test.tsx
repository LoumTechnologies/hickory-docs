// @vitest-environment jsdom
import { useState } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { codeFolding, foldedRanges } from "@codemirror/language";
import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { RepresentationVersions } from "./RepresentationVersions";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses revision controls and shares typed selection, actual comparison and return replies", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let source = "PRIVATE COMPARISON SOURCE";
  const compare = vi.fn().mockResolvedValue({ status: "completed", value: { base: "resolved-commit", target: null, editable: true, revision: "actual-revision" } });
  const reset = vi.fn().mockReturnValue({ status: "completed", value: { current_view_revision: "loaded-current" } });
  function Harness() {
    const [base, setBase] = useState("HEAD"), [target, setTarget] = useState("");
    return <RepresentationVersions id="lens" base={base} target={target} comparison={{ base: "older", target: "past", source, target_source: source, editable: false, revision: "r" }} filesBacking source={source} editor={null} dirty={false} busy={false} live={() => source} setBase={setBase} setTarget={setTarget} compare={compare} reset={reset} onError={vi.fn()} />;
  }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  expect(actions().map(a => a.label)).toEqual(["Set evidence visibility: older → past"]);
  await run("Set evidence visibility: older → past", { open: true });
  expect(JSON.stringify(actions())).not.toContain(source);
  await run("Set literate comparison base", { value: "main~2" });
  expect((screen.getByLabelText("Comparison base") as HTMLInputElement).value).toBe("main~2");
  const old = actions().find(a => a.label === "Show literate revision diff")!;
  source = "new source";
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).toMatchObject({ status: "stale" });
  expect(compare).not.toHaveBeenCalled();
  expect(await run("Show literate revision diff")).toMatchObject({ status: "result", outcome: { status: "completed", value: { revision: "actual-revision" } } });
  expect(await run("Return literate view to current code")).toMatchObject({ status: "result", outcome: { status: "completed", value: { current_view_revision: "loaded-current" } } });
  await run("Set evidence visibility: older → past", { open: false });
  expect(actions().map(a => a.label)).toEqual(["Set evidence visibility: older → past"]);
});

it("folds the actual comparison editor without changing source", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const source = Array.from({ length: 30 }, (_, i) => `line ${i}`).join("\n");
  const host = document.createElement("div"); document.body.append(host);
  const editor = new EditorView({ state: EditorState.create({ doc: source, extensions: [codeFolding()] }), parent: host });
  try {
    render(<UiActionContext.Provider value={registry}><RepresentationVersions id="lens" base="HEAD" target="" comparison={{ base: "old", target: null, source, target_source: source, editable: true, revision: "r" }} filesBacking source={source} editor={editor} dirty={false} busy={false} live={() => editor.state} setBase={vi.fn()} setTarget={vi.fn()} compare={vi.fn()} reset={vi.fn()} onError={vi.fn()} /></UiActionContext.Provider>);
    const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
    const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: label }); }); return result; };
    await run("Set evidence visibility: old → current", { open: true });
    expect(await run("Fold unchanged literate comparison regions")).toMatchObject({ status: "result", outcome: { status: "completed", value: { folded_regions: 1, source_written: false } } });
    expect(foldedRanges(editor.state).size).toBe(1);
    expect(editor.state.doc.toString()).toBe(source);
  } finally { editor.destroy(); host.remove(); }
});
