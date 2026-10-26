<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/RuntimeLogView.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ef24fe77224f"></a>
## `apps/web/src/debug/RuntimeLogView.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-ef24fe77224f-RuntimeLog"></a>
### `RuntimeLog`

<storycode:copy id="f-ef24fe77224f-RuntimeLog">
import { useEffect, useMemo, useRef, useState } from "react";
import { EditorView } from "@codemirror/view";
import { EditingSurface } from "../embed/EditingSurface";
import { RibbonOverlay } from "../shell/Ribbons";
import { openReview, sha256 } from "../lib/reviews";
import type { RuntimeRun, RuntimeEvent, RuntimeLocation } from "./runtimeObservations";
import { logLinks, outputLines, printable } from "./runtimeLog";
import { useRuntimeLogActions } from "./useRuntimeLogActions";
import { RUNTIME_REVEAL, revealRuntimeEvent } from "./runtimeNavigation";
import "./runtimeLog.css";

export function RuntimeLog({ run }: { run: RuntimeRun }) {
  const [open, setOpen] = useState(false);
  const visibility = useRef({ open: false, epoch: 0 });
  const changeOpen = (value: boolean) => {
    if (visibility.current.open !== value) visibility.current.epoch++;
    visibility.current.open = value; setOpen(value);
  };
  const [reviewPending, setReviewPending] = useState(false);
  const [reviewErrorState, setReviewErrorState] = useState<{ record: string; message: string } | null>(null);
  const reviewLock = useRef(false), mounted = useRef(true);
  const recordKey = JSON.stringify(run), currentRecord = useRef(recordKey); currentRecord.current = recordKey;
  const reviewError = reviewErrorState?.record === recordKey ? reviewErrorState.message : null;
  const setReviewError = (message: string | null) => setReviewErrorState(message === null ? null : { record: recordKey, message });
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const [pair, setPair] = useState("");
  const [sourceId, setSourceId] = useState("");
  const [pendingLine, setPendingLine] = useState<{ record: string; source: string; line: number } | null>(null);
  const [style, setStyle] = useState<"bands" | "braces">("braces");
  const [selected, setSelected] = useState("");
  const [container, setContainer] = useState<HTMLDivElement | null>(null);
  const [logView, setLogView] = useState<EditorView | null>(null);
  const [sourceView, setSourceView] = useState<EditorView | null>(null);
  const pairs = [...new Set(run.events.flatMap(event => event.observation.kind === "output" ? [JSON.stringify([event.observation.stream, event.observation.category])] : []))];
  const chosen = pair && pairs.includes(pair) ? pair : pairs[0];
  const [stream, category] = chosen ? JSON.parse(chosen) as string[] : ["stdout", "log"];
  const lines = useMemo(() => outputLines(run, stream, category), [run, stream, category]);
  const logPath = `runtime-log-${run.id}.txt`;
  const source = run.sources.find(source => source.id === sourceId) ?? run.sources[0];
  const sourcePath = `runtime-source-${run.id}-${encodeURIComponent(source?.id ?? "missing")}.txt`;
  const sourceText = new TextDecoder().decode(new Uint8Array(source?.bytes ?? []));
  const logText = lines.map(line => printable(line.bytes).replace(/\n$/, "") + (line.terminated ? "" : " ⟪partial line⟫")).join("\n");
  const links = useMemo(() => source ? logLinks(run, lines, logPath, sourcePath, source.id) : [], [run, lines, logPath, sourcePath, source?.id]);
  const sources = useMemo(() => [{ docPath: logPath, docSource: logText, view: logView ?? undefined },
    ...(source ? [{ docPath: sourcePath, docSource: sourceText, view: sourceView ?? undefined }] : [])], [logView, sourceView, logPath, sourcePath, logText, sourceText, source]);
  function jump(view: EditorView | null, line: number) {
    if (!view || !view.dom.isConnected) return; const target = view.state.doc.line(Math.max(1, Math.min(line, view.state.doc.lines)));
    view.dispatch({ selection: { anchor: target.from }, effects: EditorView.scrollIntoView(target.from, { y: "center" }) }); view.focus();
  }
  function navigate(event: RuntimeEvent, location: RuntimeLocation | null) {
    setSelected(event.id); revealRuntimeEvent(run.id, event.id);
    const captured = run.sources.find(item => item.id === location?.source);
    if (location?.bytes && captured) {
      setSourceId(captured.id);
      setPendingLine({ record: recordKey, source: captured.id, line: new TextDecoder().decode(new Uint8Array(captured.bytes.slice(0, location.bytes.start))).split("\n").length });
    }
  }
  useEffect(() => {
    if (!pendingLine) return;
    if (pendingLine.record !== recordKey || pendingLine.source !== source?.id) { setPendingLine(null); return; }
    if (sourceView) { jump(sourceView, pendingLine.line); setPendingLine(null); }
  }, [pendingLine, recordKey, source?.id, sourceText, sourceView]);
  useEffect(() => {
    const reveal = (event: Event) => {
      const detail = (event as CustomEvent<{ run: string; event: string }>).detail;
      if (detail.run !== run.id) return;
      changeOpen(true); setSelected(detail.event);
      const captured = run.events.find(item => item.id === detail.event);
      if (captured?.observation.kind === "output") setPair(JSON.stringify([captured.observation.stream, captured.observation.category]));
      const index = lines.findIndex(line => line.contributors.some(item => item.event === detail.event));
      if (index >= 0) jump(logView, index + 1);
    };
    window.addEventListener(RUNTIME_REVEAL, reveal); return () => window.removeEventListener(RUNTIME_REVEAL, reveal);
  }, [run.id, lines, logView]);
  useEffect(() => { const index = lines.findIndex(line => line.contributors.some(item => item.event === selected)); if (index >= 0) jump(logView, index + 1); }, [selected, lines, logView]);
  async function challenge(event: RuntimeEvent, signal?: AbortSignal): Promise<boolean> {
    if (signal?.aborted || reviewLock.current || !mounted.current || !visibility.current.open || currentRecord.current !== recordKey) return false;
    const disclosure = visibility.current.epoch;
    reviewLock.current = true; setReviewPending(true); setReviewError(null);
    try {
      const exact = links.find(link => link.key.includes(`:${event.id}:`) && link.review);
      if (exact?.review) { openReview(exact.review); return true; }
      const record = JSON.stringify(run), source_hash = await sha256(record);
      if (signal?.aborted || !mounted.current || !visibility.current.open || visibility.current.epoch !== disclosure || currentRecord.current !== recordKey) return false;
      const location = event.observation.kind === "exception" ? event.observation.throwing : event.emission;
      openReview({ kind: "runtime-record", path: `runtime-record://${run.id}`, quote: JSON.stringify(event),
        source_hash, lines: [1, 1], relationship: { run: run.id, event,
          source_revision: run.sources.find(source => source.id === location?.source)?.revision ?? null, precision: location?.precision ?? "unknown" } });
      return true;
    } catch (cause) { if (mounted.current && currentRecord.current === recordKey) setReviewError(String(cause)); return false; }
    finally { reviewLock.current = false; if (mounted.current) setReviewPending(false); }
  }
  useRuntimeLogActions(run, { open, pair: chosen, pairs, source: source?.id ?? null, style, selected, lines, pending: reviewPending, error: reviewError }, {
    open: changeOpen, pair: setPair, source: setSourceId, style: setStyle, navigate, challenge,
  });
  return <details className="runtime-log" open={open} onToggle={event => changeOpen(event.currentTarget.open)}>
    <summary>Execution log and source relationships · run {run.id}</summary>
    {open && <>
      {reviewError && <p role="alert">{reviewError}</p>}
      <p>Observed execution provenance · revision {source?.revision ?? "not retained"}. These links are separate from byte lineage, model context and declared support. Emission and calling context are different locations. Captured source is historical.</p>
      <label>Runtime log stream <select value={chosen ?? ""} onChange={event => setPair(event.target.value)}>{pairs.map(value => <option key={value} value={value}>{JSON.parse(value).join(" · ")}</option>)}</select></label>
      <label>Captured source <select value={source?.id ?? ""} disabled={!source} onChange={event => setSourceId(event.target.value)}>{run.sources.map(item => <option key={item.id} value={item.id}>{item.uri} · {item.revision}</option>)}</select></label>
      <label>Execution connector style <select value={style} onChange={event => setStyle(event.target.value as "bands" | "braces")}><option value="braces">Braces</option><option value="bands">Ribbons</option></select></label>
      <div className="runtime-log-panes" ref={setContainer}>
        <div><h4>Captured log · {stream}/{category}</h4><EditingSurface readOnly value={logText} onViewReady={setLogView} ariaLabel="Captured execution log" /></div>
        {source ? <div><h4>Captured source · {source.uri}</h4><EditingSurface readOnly value={sourceText} onViewReady={setSourceView} ariaLabel="Captured execution source" /></div> : <p>No captured source retained for this run.</p>}
        <RibbonOverlay container={container} sources={sources} files={[]} links={links} ribbonStyle={style} visibility="always" onNavigate={target => {
          if (target.kind !== "path") return;
          const link = links.find(item => item.key === target.linkKey);
          if (link) { const event = run.events.find(event => link.key.includes(`:${event.id}:`)); if (event) revealRuntimeEvent(run.id, event.id); }
          jump(target.path === logPath ? logView : sourceView, target.lines?.[0] ?? 1);
        }} />
      </div>
      <ol aria-label="Execution line contributors">{lines.map((line, index) => <li key={index}>
        <span>Log line {index + 1} · {line.terminated ? "LF terminated" : "partial"} · {line.contributors.length} contributions</span>
        {line.contributors.map((item, occurrence) => {
          const event = run.events.find(event => event.id === item.event)!;
          return <div key={`${item.event}:${occurrence}`} data-runtime-event={event.id}>
            <strong>{selected === event.id ? "Selected · " : ""}{event.id}</strong> · invocation {event.invocation ?? "unknown"} · chunk bytes {item.chunk_bytes.start}–{item.chunk_bytes.end} → line bytes {item.line_bytes.start}–{item.line_bytes.end}
            <button onClick={() => navigate(event, event.emission)}>Emission · {event.emission?.precision ?? "unknown"}</button>
            {event.callers.map((caller, callerIndex) => <button key={callerIndex} onClick={() => navigate(event, caller)}>Calling context {callerIndex + 1} · {caller.precision}</button>)}
            <button disabled={reviewPending} onClick={() => void challenge(event)}>Challenge runtime relationship</button>
          </div>;
        })}
      </li>)}</ol>
      {run.events.filter(event => event.observation.kind !== "output").map(event => <p key={event.id}>{selected === event.id ? "Selected · " : ""}{event.id} · typed {event.observation.kind} observation; not inferred from stderr.
        <button onClick={() => navigate(event, event.observation.kind === "exception" ? event.observation.throwing : event.emission)}>Throwing/emitting location</button>
        <button disabled={reviewPending} onClick={() => void challenge(event)}>Challenge runtime relationship</button></p>)}
    </>}
  </details>;
}
</storycode:copy>

### Assemble `apps/web/src/debug/RuntimeLogView.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/RuntimeLogView.tsx"><storycode:paste select="#f-ef24fe77224f-RuntimeLog"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-9faa6c493ee7"></a>
## `apps/web/src/debug/RuntimeLogView.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-9faa6c493ee7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9faa6c493ee7-expression_statement">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { RuntimeLog } from "./RuntimeLogView";
import type { RuntimeRun } from "./runtimeObservations";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import * as reviews from "../lib/reviews";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-9faa6c493ee7-fixture"></a>
### `fixture`

<storycode:copy id="f-9faa6c493ee7-fixture">
function fixture(): RuntimeRun {
  return { schema_version: 1, id: "one", started_unix_ms: 1, status: "completed", pinned: false, truncated: false, missing_artifacts: [], detail: null,
    sources: [{ id: "first", uri: "first.md", revision: "a".repeat(64), bytes: [...new TextEncoder().encode("PRIVATE first\nsecond\n")] },
      { id: "second", uri: "second.md", revision: "b".repeat(64), bytes: [...new TextEncoder().encode("other source\n")] }],
    events: ["stdout", "stderr"].map((stream, index) => ({ id: `event-${index}`, provider: "fixture", block: "main", invocation: "call", parent_invocation: null,
      thread: null, task: null, time: null, emission: { source: "first", block: "main", bytes: { start: 0, end: 7 }, precision: "exact" }, callers: [],
      observation: { kind: "output", stream, category: "write", bytes: [...new TextEncoder().encode(`${stream} output\n`)] } })) };
}
</storycode:copy>
<a id="f-9faa6c493ee7-tools"></a>
### `tools`

<storycode:copy id="f-9faa6c493ee7-tools">
function tools() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  return { registry, actions, run };
}
</storycode:copy>
<a id="f-9faa6c493ee7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9faa6c493ee7-expression_statement-2">
it("shares only expanded runtime relationships, selectors and exact captured review requests", async () => {
  const { registry, actions, run } = tools();
  const review = vi.spyOn(reviews, "openReview").mockImplementation(() => {});
  const captured = fixture();
  render(<UiActionContext.Provider value={registry}><RuntimeLog run={captured} /></UiActionContext.Provider>);
  expect(actions().some(action => action.label === "Read runtime log relationship metadata bytes")).toBe(false);
  expect(JSON.stringify(actions())).not.toContain("PRIVATE");
  await run("Set runtime log visibility", { open: true });
  const reading = await run("Read runtime log relationship metadata bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  const text = new TextDecoder().decode(Uint8Array.from(reading.outcome.value.bytes));
  expect(text).toContain("event-0"); expect(text).not.toContain("PRIVATE");
  const old = actions().find(action => action.label === "Go to runtime emitting/throwing location: event-0")!.handle;
  fireEvent.change(screen.getByLabelText("Runtime log stream"), { target: { value: JSON.stringify(["stderr", "write"]) } });
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "other-stream" })).not.toMatchObject({ status: "result" });
  await run("Select runtime captured source", { index: 1 });
  expect((screen.getByLabelText("Captured source") as HTMLSelectElement).value).toBe("second");
  await run("Challenge runtime relationship: event-1");
  expect(review.mock.calls[0][0]).toMatchObject({ kind: "runtime-record", relationship: { source_revision: captured.sources[0].revision } });
  expect(review.mock.calls[0][0].source_hash).toBe(await reviews.sha256(JSON.stringify(captured)));
  await run("Set runtime relationship connector style", { style: "bands" });
  expect((screen.getByLabelText("Execution connector style") as HTMLSelectElement).value).toBe("bands");
  await run("Go to runtime emitting/throwing location: event-1");
  expect((screen.getByLabelText("Captured source") as HTMLSelectElement).value).toBe("first");
  expect(await run("Challenge runtime relationship: event-1")).toMatchObject({ status: "result", outcome: { value: { review_requested: true, review_completed: false } } });
  expect(review).toHaveBeenCalledWith(expect.objectContaining({ source_hash: captured.sources[0].revision, relationship: expect.objectContaining({ run: "one", event: expect.objectContaining({ id: "event-1" }) }) }));
  await run("Set runtime log visibility", { open: false });
  expect(actions().some(action => action.label === "Select runtime captured source")).toBe(false);
  expect(actions().some(action => action.label === "Read runtime log relationship metadata bytes")).toBe(false);
});
</storycode:copy>
<a id="f-9faa6c493ee7-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-9faa6c493ee7-expression_statement-3">
it("handles missing captured source and refuses a late review hash after capture replacement", async () => {
  const { registry, actions, run } = tools();
  const review = vi.spyOn(reviews, "openReview").mockImplementation(() => {});
  let finish!: (hash: string) => void;
  vi.spyOn(reviews, "sha256").mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const captured = { ...fixture(), sources: [] };
  const panel = (value: RuntimeRun) => <UiActionContext.Provider value={registry}><RuntimeLog run={value} /></UiActionContext.Provider>;
  const view = render(panel(captured));
  await run("Set runtime log visibility", { open: true });
  expect(screen.getByText("No captured source retained for this run.")).toBeTruthy();
  expect(actions().some(action => action.label === "Select runtime captured source")).toBe(false);
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === "Challenge runtime relationship: event-0")!.handle, arguments: {}, request_id: "late",
  }); });
  view.rerender(panel({ ...captured, id: "replacement" }));
  await act(async () => { finish("c".repeat(64)); expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable" } }); });
  expect(review).not.toHaveBeenCalled();
  await act(async () => { pending = registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === "Challenge runtime relationship: event-0")!.handle, arguments: {}, request_id: "cancelled-hash",
  }); });
  expect(registry.cancel(registry.scope, { id: "python:fixture", kind: "agent" }, "cancelled-hash")).toBe(true);
  await act(async () => { finish("e".repeat(64)); expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable" } }); });
  expect(review).not.toHaveBeenCalled();
  await act(async () => { pending = registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === "Challenge runtime relationship: event-0")!.handle, arguments: {}, request_id: "closed-disclosure",
  }); });
  expect(await run("Set runtime log visibility", { open: false })).toMatchObject({ status: "result", outcome: { value: { open: false } } });
  expect(await run("Set runtime log visibility", { open: true })).toMatchObject({ status: "result", outcome: { value: { open: true } } });
  await act(async () => { finish("d".repeat(64)); expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable" } }); });
  expect(review).not.toHaveBeenCalled();
});
</storycode:copy>

### Assemble `apps/web/src/debug/RuntimeLogView.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/RuntimeLogView.test.tsx"><storycode:paste select="#f-9faa6c493ee7-expression_statement"/><storycode:paste select="#f-9faa6c493ee7-fixture"/><storycode:paste select="#f-9faa6c493ee7-tools"/><storycode:paste select="#f-9faa6c493ee7-expression_statement-2"/><storycode:paste select="#f-9faa6c493ee7-expression_statement-3"/></storycode:file>


</storycode:doc>
