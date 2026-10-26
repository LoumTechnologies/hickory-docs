<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/testGutterActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9f6f73085576"></a>
## `apps/web/src/editor/testGutterActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-9f6f73085576-TestScan"></a>
### `TestScan`

<storycode:copy id="f-9f6f73085576-TestScan">
import { StateEffect, StateField, EditorState, type Extension } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import { editorLanguageActions } from "./languageActions";
import type { TestGutterOptions, TestMark } from "./testGutter";
import { sha256 } from "../lib/reviews";
import { prepareEditorChanges } from "./textInput";
import type { UiOutcome } from "../lib/uiActions";
export type TestScan = { marks: TestMark[]; source: EditorState["doc"] | null };
</storycode:copy>
<a id="f-9f6f73085576-LaunchState"></a>
### `LaunchState`

<storycode:copy id="f-9f6f73085576-LaunchState">
type LaunchState = { source: EditorState["doc"] | null; hash: string | null; mark: TestMark | null; error: string | null; terminal: { id: string; title: string } | null; cancelled: boolean; admission: boolean };
</storycode:copy>
<a id="f-9f6f73085576-testGutterActions"></a>
### `testGutterActions`

The gutter's native and model requests share source admission and one pending slot.

<storycode:copy id="f-9f6f73085576-testGutterActions">
/** The gutter's native and model requests share source admission and one pending slot. */
export function testGutterActions(options: TestGutterOptions, scan: StateField<TestScan>) {
  let pending = false;
  const update = StateEffect.define<Partial<LaunchState>>();
  const status = StateField.define<LaunchState>({ create: () => ({ source: null, hash: null, mark: null, error: null, terminal: null, cancelled: false, admission: false }), update: (value, tr) => { for (const effect of tr.effects) if (effect.is(update)) value = { ...value, ...effect.value }; return value; } });
  const publish = (view: EditorView, patch: Partial<LaunchState>) => { if (view.dom.isConnected && view.state.field(status, false)) view.dispatch({ effects: update.of(patch) }); };
  const run = async (view: EditorView, mark: TestMark, signal: AbortSignal): Promise<UiOutcome> => {
    const source = view.state.doc;
    const owned = () => view.dom.isConnected && view.state.doc === source && !!view.state.field(status, false);
    const current = () => owned() && !signal.aborted;
    const refuse = (reason: string, input = false): UiOutcome => {
      publish(view, { error: reason, cancelled: signal.aborted });
      if (current()) options.onError?.(reason);
      return { status: input ? "requires_input" : "unavailable", reason, ...(input ? { request: "test-gutter-source-admission" } : {}) };
    };
    if (!owned() || pending) return { status: "unavailable", reason: "Test gutter retired or another launch is pending" };
    const observed = view.state.field(scan);
    if (observed.source !== source || !observed.marks.some(candidate => candidate.line === mark.line && candidate.name === mark.name)) return refuse("Test markers changed; wait for the current source scan before running");
    if (!mark.name.trim()) return refuse("This test has no nonempty name; the runner cannot target it safely from this gutter", true);
    pending = true; publish(view, { source, mark, hash: null, error: null, terminal: null, cancelled: false, admission: false });
    try {
      let hash: string;
      try {
        const candidate = options.serializedSource?.();
        const serialized = candidate !== undefined && EditorState.create({ doc: candidate }).doc.eq(source) ? candidate : source.toString();
        const validation = prepareEditorChanges(serialized, []);
        if ("reason" in validation) return refuse(validation.reason, true);
        hash = await sha256(serialized);
      }
      catch { return refuse("This host cannot fingerprint the current test source. Use a supported secure browser/desktop context before running", true); }
      if (!current()) return refuse("Test source changed, closed or was cancelled before dispatch; no test request was submitted");
      publish(view, { hash });
      const result = await options.onRun(mark, hash, current);
      const value = result && typeof result === "object" ? result as Record<string, unknown> : null;
      const terminal = value && typeof value.id === "string" && typeof value.title === "string" ? { id: value.id, title: value.title } : null;
      const admission = value?.source_checked_at_admission === true && value.source_hash === hash;
      publish(view, { terminal, admission, cancelled: signal.aborted });
      if (!terminal || !admission) return refuse(`The host did not confirm a terminal with saved-source admission.${terminal ? ` A terminal may have opened (${terminal.id});` : ""} Inspect launch status and update the host before retrying`, true);
      if (signal.aborted) return refuse("Cancellation followed dispatch; inspect the test launch status for a possible terminal before retrying");
      return { status: "completed", value: { callback_completed: true, terminal: terminal ? { id: terminal.id, title: terminal.title.slice(0, 1024), title_truncated: terminal.title.length > 1024 } : null, source_hash: hash, source_checked_at_admission: admission, source_frozen: false, originating_source_retired: !owned(), test_passed: null, scope: "Launch admission/callback only; inspect the actual terminal. No immutable project, completed test or current document verification is claimed" } };
    } catch (error) { return refuse(error instanceof Error ? error.message : String(error)); }
    finally { pending = false; publish(view, {}); }
  };
  const prefix = crypto.randomUUID(), number = { type: "number" as const, integer: true, minimum: 0, maximum: 100_000_000 };
  const visible = (view: EditorView) => view.state.field(scan).marks.length > 0;
  const extensions: Extension = [status,
    editorLanguageActions.of({ id: `${prefix}:read-tests`, subject: `${prefix}:marker-reading`, label: "Read test gutter marker page", visible, enabled: () => true,
      parameters: { first: number, count: { ...number, minimum: 1, maximum: 64 } },
      execute: async (view, args) => {
        const observed = view.state.field(scan), first = Number(args.first), count = Number(args.count), page = observed.marks.slice(first, first + count);
        if (first > observed.marks.length || new TextEncoder().encode(JSON.stringify(page)).length > 8192) return { status: "unavailable", reason: "Test marker page is outside the scan or exceeds 8192 UTF-8 bytes; request fewer marks" };
        return { status: "completed", value: { tests: page.map(mark => ({ ...mark })), total: observed.marks.length, next: first + count < observed.marks.length ? first + count : null, rescan_pending: observed.source !== view.state.doc, textual_detection_only: true } };
      } }),
    editorLanguageActions.of({ id: `${prefix}:run-test`, label: "Run test at editor line", visible, parameters: { line: { ...number, minimum: 1 } },
      enabled: view => !pending && view.state.field(scan).source === view.state.doc,
      execute: async (view, args, signal) => { const mark = view.state.field(scan).marks.find(mark => mark.line === args.line); return mark ? run(view, mark, signal) : { status: "unavailable", reason: "No current test marker at this line" }; } }),
    editorLanguageActions.of({ id: `${prefix}:test-status`, subject: `${prefix}:status-reading`, label: "Read test gutter launch status bytes", visible: view => visible(view) || pending || !!view.state.field(status).error || !!view.state.field(status).terminal,
      parameters: { offset: number, limit: { ...number, minimum: 1, maximum: 8192 } }, enabled: () => true,
      execute: async (view, args) => {
        const actual = view.state.field(status), bytes = new TextEncoder().encode(JSON.stringify({ pending, requested_test: actual.mark, source_hash: actual.hash, originating_source_retired: !!actual.source && actual.source !== view.state.doc, terminal: actual.terminal, error: actual.error, cancellation_requested: actual.cancelled, source_checked_at_admission: actual.admission, source_frozen: false, test_passed: null }));
        const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
        if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds current launch status" };
        return { status: "completed", value: { bytes: [...bytes.slice(offset, end)], encoding: "JSON UTF-8", span: [offset, end], next: end < bytes.length ? end : null, total_bytes: bytes.length } };
      } }),
  ];
  return { extensions, pending: () => pending, run };
}
</storycode:copy>

### Assemble `apps/web/src/editor/testGutterActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/testGutterActions.ts"><storycode:paste select="#f-9f6f73085576-TestScan"/><storycode:paste select="#f-9f6f73085576-LaunchState"/><storycode:paste select="#f-9f6f73085576-testGutterActions"/></storycode:file>

</storycode:doc>
