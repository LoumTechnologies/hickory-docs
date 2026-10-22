// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "../editor/uiEditorActions";
import { runtimeEditor, setRuntimeReading } from "./cmRuntime";
import { selectedObservations, type ObservationReading } from "./observations";
import { RUNTIME_REVEAL } from "./runtimeNavigation";
let view: EditorView | undefined;
afterEach(() => { cleanup(); const host = view?.dom.parentElement; view?.destroy(); host?.remove(); });
it("shares native inline disclosure/log navigation, reads only explicit expansion and retires source owners", async () => {
  const source = "code\n";
  const scope = { run: "inline-run", stop: "runtime", frame: null, invocation: null };
  const reading: ObservationReading = { schema_version: 1, source: { id: "inline.md", uri: "inline.md", revision: "captured", bytes: [...new TextEncoder().encode(source)] }, scope,
    locations: [{ id: "L", revision: "captured", line: 1, location: { source: "inline.md", bytes: { start: 0, end: 4 }, precision: "exact" } }],
    annotations: [{ evidence: "event-one", provider: "runtime", payload_schema: "runtime/v1", location: "L", scope, payload: { kind: "runtime", event: { observation: { kind: "log", level: "info", message: "CAPTURED BODY", fields: {} } } } }], unsupported: ["not verification"] };
  const host = document.body.appendChild(document.createElement("div"));
  const current = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: runtimeEditor }) }); view = current;
  const registry = new UiActions({ workspace: "p", window: "inline" }, () => true, async () => {});
  function Bridge() { useEditorActions(current, "inline"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue>) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "acp:inline", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  const disclose = "Set inline runtime annotation disclosure", read = "Read expanded inline runtime annotation bytes", reveal = "Show inline runtime event in execution log";
  await act(async () => current.dispatch({ effects: setRuntimeReading.of(reading) }));
  expect(JSON.stringify(actions())).not.toContain("CAPTURED BODY");
  expect(actions().some(action => action.label === read)).toBe(false);
  expect(selectedObservations("inline.md", source)).toEqual([]);
  await run(disclose, { index: 0, open: true });
  const native = current.dom.querySelector<HTMLDetailsElement>("details.runtime-inline")!;
  native.scrollIntoView = vi.fn();
  expect(native.open).toBe(true);
  expect(selectedObservations("inline.md", source)[0]).toMatchObject({ evidence: ["event-one"] });
  const reply = await run(read, { index: 0, offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes))).toContain("CAPTURED BODY");
  const navigation = vi.fn(); window.addEventListener(RUNTIME_REVEAL, navigation);
  try {
    expect(await run(reveal, { index: 0, event: 0 })).toMatchObject({ outcome: { value: { navigation_requested: true } } });
    await act(async () => native.querySelector<HTMLButtonElement>("button")!.click());
    expect(navigation).toHaveBeenCalledTimes(2);
    expect(native.scrollIntoView).toHaveBeenCalledTimes(2);
    await act(async () => { native.open = false; native.dispatchEvent(new Event("toggle")); });
    expect(actions().some(action => action.label === read)).toBe(false);
    expect(selectedObservations("inline.md", source)).toEqual([]);
    await run(disclose, { index: 0, open: true });
    const old = actions().find(action => action.label === read)!;
    const oldButton = native.querySelector<HTMLButtonElement>("button")!;
    await act(async () => current.dispatch({ changes: { from: 0, insert: "new " } }));
    expect(selectedObservations("inline.md", source)).toEqual([]);
    expect(await registry.act(registry.scope, { id: "acp:inline", kind: "agent" }, { handle: old.handle, arguments: { index: 0, offset: 0, limit: 8192 }, request_id: "stale" })).not.toMatchObject({ outcome: { status: "completed" } });
    await act(async () => oldButton.click());
    expect(navigation).toHaveBeenCalledTimes(2);
  } finally { window.removeEventListener(RUNTIME_REVEAL, navigation); }
});
