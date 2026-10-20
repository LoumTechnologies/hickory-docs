// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { ProfileEvidence } from "./ProfileEvidence";
import type { RuntimeRun } from "./runtimeObservations";
import { selectedObservations } from "./observations";
import { revealRuntimeEvent } from "./runtimeNavigation";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
vi.mock("./runtimeNavigation", () => ({ revealRuntimeEvent: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
const source = "one\nhot\n";
function captured(id = "profile"): RuntimeRun {
  return { schema_version: 1, id, started_unix_ms: 1, status: "completed", pinned: false,
    sources: [{ id: "s", uri: "hot.py", revision: "captured", bytes: [...new TextEncoder().encode(source)] }],
    events: [{ id: "cost", provider: "cprofile", block: null, invocation: null, parent_invocation: null, thread: null, task: null, time: null,
      emission: { source: "s", block: null, bytes: { start: 4, end: 7 }, precision: "exact" }, callers: [],
      observation: { kind: "profile", metric: "self_seconds", value: 0.25, unit: "s", aggregation: "function", samples: null,
        function: { name: "hot", file: "hot.py", line: 2, primitive_calls: 2, total_calls: 3 },
        interval: { start: { domain: "run", unit: "s", value: 0 }, end: { domain: "run", unit: "s", value: 1 }, scope: "main thread" } } }],
    truncated: false, missing_artifacts: [], detail: null };
}
it("shares profile disclosure and pinning and resets selection on run replacement", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const draw = (id: string) => <UiActionContext.Provider value={registry}><ProfileEvidence run={captured(id)} stale={false} /></UiActionContext.Provider>;
  const view = render(draw("first"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const handle = (label: string) => actions().find(action => action.label === label)!.handle;
  const run = async (target: string, args: Record<string, boolean | number>) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: target, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(actions().some(action => action.label === "Read disclosed profile aggregates bytes")).toBe(false);
  expect(selectedObservations("hot.py", source)).toHaveLength(0);
  await run(handle("Set profile evidence visibility"), { open: true });
  expect(selectedObservations("hot.py", source)[0].evidence).toEqual(["cost"]);
  const reading = await run(handle("Read disclosed profile aggregates bytes"), { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array(reading.outcome.value.bytes)))[0]).toMatchObject({ self_seconds: 0.25, total_calls: 3 });
  fireEvent.click(screen.getByRole("checkbox"));
  await run(handle("Set profile evidence visibility"), { open: false });
  expect(selectedObservations("hot.py", source)).toHaveLength(1);
  const old = handle("Set profile evidence context pin");
  view.rerender(draw("second"));
  expect(selectedObservations("hot.py", source)).toHaveLength(0);
  expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(false);
  expect(await run(old, { pinned: true })).not.toMatchObject({ status: "result" });
});
it("reveals the actual recorded event without jumping into unrelated editor bytes", () => {
  const editor = new EditorView({ state: EditorState.create({ doc: "different buffer" }), parent: document.body });
  const rendered = render(<ProfileEvidence run={captured()} view={editor} stale={false} />);
  const fold = document.querySelector("details")!; fold.open = true; fireEvent(fold, new Event("toggle"));
  fireEvent.click(screen.getByText("Source · cost"));
  expect(editor.state.selection.main.head).toBe(0);
  expect(revealRuntimeEvent).toHaveBeenCalledWith("profile", "cost");
  rendered.rerender(<ProfileEvidence run={captured()} view={editor} stale />);
  expect((screen.getByText("Source · cost") as HTMLButtonElement).disabled).toBe(true);
  rendered.unmount(); editor.destroy();
});
