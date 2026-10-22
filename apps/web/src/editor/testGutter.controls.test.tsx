// Guarantee: docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
import { testGutter, type TestGutterOptions, type TestMark } from "./testGutter";
import { sha256 } from "../lib/reviews";
vi.mock("../lib/reviews", () => ({ sha256: vi.fn(async () => "a".repeat(64)) }));
const views: EditorView[] = [];
afterEach(() => { cleanup(); for (const view of views.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } vi.mocked(sha256).mockReset().mockResolvedValue("a".repeat(64)); });
const source = "#[test]\nfn adds() {}\n";
async function setup(onRun: TestGutterOptions["onRun"], text = source, language = "rust", serializedSource?: TestGutterOptions["serializedSource"]) {
  const host = document.body.appendChild(document.createElement("div"));
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: text, extensions: testGutter({ language, onRun, serializedSource }) }) }); views.push(view);
  const registry = new UiActions({ workspace: "tests", window: "one" }, () => true, async () => {});
  function Bridge() { useEditorActions(view, "tests"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  await waitFor(() => expect(actions().find(action => action.label === "Run test at editor line")?.enabled).toBe(true));
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "acp:gutter", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  const status = async () => { const reply = await run("Read test gutter launch status bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } }; return JSON.parse(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes))); };
  return { view, registry, actions, run, status };
}
it("excludes duplicate launches, keeps status readable while pending and reports admission rather than a passing test", async () => {
  let resolve!: (value: unknown) => void;
  const launch = vi.fn((_mark: TestMark, _hash: string, _current: () => boolean) => new Promise<unknown>(done => { resolve = done; }));
  const { view, registry, actions, run, status } = await setup(launch);
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:gutter", kind: "agent" }, { handle: actions().find(action => action.label === "Run test at editor line")!.handle, arguments: { line: 2 }, request_id: "launch" }); });
  await waitFor(() => expect(launch).toHaveBeenCalledTimes(1));
  expect(launch.mock.calls[0][0]).toEqual({ line: 2, name: "adds" });
  expect(launch.mock.calls[0][1]).toBe("a".repeat(64));
  expect(await status()).toMatchObject({ pending: true, test_passed: null });
  expect(await run("Run test at editor line", { line: 2 })).toMatchObject({ status: "disabled" });
  const marker = view.dom.querySelector('.cm-test-run[role="button"]')!;
  expect(marker.getAttribute("aria-disabled")).toBe("true");
  await act(async () => marker.dispatchEvent(new MouseEvent("mousedown", { bubbles: true })));
  expect(launch).toHaveBeenCalledTimes(1);
  await act(async () => resolve({ id: "terminal", title: "test: adds", source_hash: "a".repeat(64), source_checked_at_admission: true }));
  expect(await pending).toMatchObject({ outcome: { status: "completed", value: { terminal: { id: "terminal" }, source_checked_at_admission: true, source_frozen: false, test_passed: null } } });
  expect(await status()).toMatchObject({ pending: false, terminal: { id: "terminal" } });
  expect(view.state.doc.toString()).toBe(source);
});
it("refuses a changed source before dispatch and records cancellation after an admitted launch", async () => {
  let hashDone!: (hash: string) => void;
  vi.mocked(sha256).mockImplementationOnce(() => new Promise(done => { hashDone = done; }));
  const launch = vi.fn();
  const { view, registry, actions } = await setup(launch);
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:gutter", kind: "agent" }, { handle: actions().find(action => action.label === "Run test at editor line")!.handle, arguments: { line: 2 }, request_id: "changed" }); });
  await waitFor(() => expect(hashDone).toBeTypeOf("function"));
  await act(async () => view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "#[test]\nfn changed() {}\n" } }));
  await act(async () => hashDone("a".repeat(64)));
  expect(await pending).toMatchObject({ outcome: { status: "unavailable" } });
  expect(launch).not.toHaveBeenCalled();

  let resolve!: (value: unknown) => void;
  const admitted = vi.fn((_mark: TestMark, _hash: string, _current: () => boolean) => new Promise<unknown>(done => { resolve = done; }));
  const second = await setup(admitted);
  await act(async () => { pending = second.registry.act(second.registry.scope, { id: "acp:gutter", kind: "agent" }, { handle: second.actions().find(action => action.label === "Run test at editor line")!.handle, arguments: { line: 2 }, request_id: "cancelled" }); });
  await waitFor(() => expect(admitted).toHaveBeenCalledTimes(1));
  expect(second.registry.cancel(second.registry.scope, { id: "acp:gutter", kind: "agent" }, "cancelled")).toBe(true);
  await act(async () => resolve({ id: "possibly-running", title: "test: adds", source_hash: "a".repeat(64), source_checked_at_admission: true }));
  expect(await pending).not.toMatchObject({ outcome: { status: "completed" } });
  expect(await second.status()).toMatchObject({ cancellation_requested: true, terminal: { id: "possibly-running" } });
});
it("does not turn an empty named marker into a whole-file run or trust an older unguarded host", async () => {
  const launch = vi.fn();
  const empty = await setup(launch, 'it("", () => {});\n', "typescript");
  expect(await empty.run("Run test at editor line", { line: 1 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(launch).not.toHaveBeenCalled();
  const oldHost = await setup(async () => ({ id: "legacy", title: "test: adds" }));
  expect(await oldHost.run("Run test at editor line", { line: 2 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(await oldHost.status()).toMatchObject({ terminal: { id: "legacy" }, source_checked_at_admission: false, test_passed: null });
});

it("fingerprints matching owned CRLF bytes and refuses malformed Unicode", async () => {
  const raw = source.replaceAll("\n", "\r\n");
  const launch = vi.fn(async () => ({ id: "crlf", title: "test: adds", source_hash: "a".repeat(64), source_checked_at_admission: true }));
  const actual = await setup(launch, raw, "rust", () => raw);
  expect(actual.view.state.doc.toString()).toBe(source);
  await actual.run("Run test at editor line", { line: 2 });
  expect(sha256).toHaveBeenLastCalledWith(raw);
  const malformed = await setup(launch, source + "\ud800");
  expect(await malformed.run("Run test at editor line", { line: 2 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(launch).toHaveBeenCalledTimes(1);
});
