<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Close or recover a document without losing the wrong draft

For: Authors editing executable documents and the files those documents produce.

Close a changed note, choose Discard, then reopen the workspace. The discarded draft must stay discarded. A crash instead recovers the last retained draft as unsaved work.

## What must happen

A buffer remains dirty until its actual file write is acknowledged. Tab close offers Save, Discard and Cancel; window close offers Save all, Close without saving and Cancel. Explicit discard removes the corresponding recovery draft and suppresses late writes from that discarded buffer. Save failure, cancellation or changed source identity keeps the affected view open. Each unnamed note has independent recovery outside the project. Named-file recovery follows its setting, which defaults off. Recovery does not create a project file or acknowledge a save. A recovered draft is compared with its recorded base and current file before restoration or merge. An absent editor view must not be read as an empty buffer.

## Limits of the evidence

Periodic recovery can lose input after its last successful flush. Frontend tests mock persistence and do not certify native window destruction or crash durability. Keep the original merge, path-renaming and startup-note clauses as detailed requirements.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the explicit-discard, in-flight-write and dead-view corrections into one current lifecycle rule. Keep crash recovery distinct from an intentional close without saving.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-0b8059fcf25a"></a>
## `apps/web/src/components/PromptPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Close or recover a document without losing the wrong draft](contract.md).

<a id="f-0b8059fcf25a-PromptPolicy"></a>
### `PromptPolicy`

In-app prompts keep sync and in-flight jobs responsive while an answer is pending.

<storycode:copy id="f-0b8059fcf25a-PromptPolicy">
// In-app prompts keep sync and in-flight jobs responsive while an answer is pending.
import { useCallback, useEffect, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";

type PromptPolicy = { signal?: AbortSignal; agent?: boolean; currentRevision?: () => unknown };
</storycode:copy>
<a id="f-0b8059fcf25a-PromptBase"></a>
### `PromptBase`

<storycode:copy id="f-0b8059fcf25a-PromptBase">
type PromptBase = { id: string; title: string; agent: boolean; currentRevision?: () => unknown; baseline: unknown; cleanup?: () => void };
</storycode:copy>
<a id="f-0b8059fcf25a-TextPrompt"></a>
### `TextPrompt`

<storycode:copy id="f-0b8059fcf25a-TextPrompt">
interface TextPrompt extends PromptBase { kind: "text"; initial: string; resolve: (value: string | null) => void }
</storycode:copy>
<a id="f-0b8059fcf25a-ChoicePrompt"></a>
### `ChoicePrompt`

<storycode:copy id="f-0b8059fcf25a-ChoicePrompt">
interface ChoicePrompt<T> extends PromptBase { kind: "choice"; options: { label: string; value: T }[]; resolve: (value: T | null) => void }
</storycode:copy>
<a id="f-0b8059fcf25a-Prompt"></a>
### `Prompt`

<storycode:copy id="f-0b8059fcf25a-Prompt">
type Prompt = TextPrompt | ChoicePrompt<unknown>;
</storycode:copy>
<a id="f-0b8059fcf25a-base"></a>
### `base`

<storycode:copy id="f-0b8059fcf25a-base">
const base = (title: string, policy: PromptPolicy): PromptBase => ({ id: crypto.randomUUID(), title, agent: policy.agent === true, currentRevision: policy.currentRevision, baseline: policy.currentRevision?.() });

</storycode:copy>
<a id="f-0b8059fcf25a-usePrompt"></a>
### `usePrompt`

<storycode:copy id="f-0b8059fcf25a-usePrompt">
export function usePrompt() {
  const [prompt, setPrompt] = useState<Prompt | null>(null);
  const pending = useRef<Prompt | null>(null);
  const replace = useCallback((next: Prompt, signal?: AbortSignal) => {
    if (signal?.aborted) { next.resolve(null); return; }
    pending.current?.cleanup?.(); pending.current?.resolve(null);
    pending.current = next;
    const cancel = () => { if (pending.current !== next) return; next.cleanup?.(); pending.current = null; setPrompt(null); next.resolve(null); };
    if (signal) { signal.addEventListener("abort", cancel, { once: true }); next.cleanup = () => signal.removeEventListener("abort", cancel); }
    setPrompt(next);
  }, []);
  const askText = useCallback((title: string, initial: string, policy: PromptPolicy = {}): Promise<string | null> => new Promise(resolve => {
    replace({ ...base(title, policy), kind: "text", initial, resolve }, policy.signal);
  }), [replace]);
  const askChoice = useCallback(<T,>(title: string, options: { label: string; value: T }[], policy: PromptPolicy = {}): Promise<T | null> => new Promise(resolve => {
    replace({ ...base(title, policy), kind: "choice", options, resolve: resolve as (value: unknown) => void }, policy.signal);
  }), [replace]);
  const settle = useCallback((value: unknown, expectedId?: string) => {
    const current = pending.current;
    if (expectedId && current?.id !== expectedId) return;
    pending.current = null; setPrompt(null); current?.cleanup?.(); current?.resolve(value as never);
  }, []);
  useEffect(() => () => { const current = pending.current; pending.current = null; current?.cleanup?.(); current?.resolve(null); }, []);
  return { prompt, askText, askChoice, settle };
}

</storycode:copy>
<a id="f-0b8059fcf25a-PromptPanel"></a>
### `PromptPanel`

<storycode:copy id="f-0b8059fcf25a-PromptPanel">
export function PromptPanel({ prompt, onSettle }: { prompt: ReturnType<typeof usePrompt>["prompt"]; onSettle: (value: unknown, expectedId?: string) => void }) {
  return prompt ? <ActivePrompt key={prompt.id} prompt={prompt} onSettle={onSettle} /> : null;
}
</storycode:copy>
<a id="f-0b8059fcf25a-ActivePrompt"></a>
### `ActivePrompt`

<storycode:copy id="f-0b8059fcf25a-ActivePrompt">
function ActivePrompt({ prompt, onSettle }: { prompt: Prompt; onSettle: (value: unknown, expectedId?: string) => void }) {
  const [draft, setDraft] = useState(prompt.kind === "text" ? prompt.initial : "");
  const [error, setError] = useState("");
  const parent = `prompt:${prompt.id}`;
  const current = () => !prompt.currentRevision || Object.is(prompt.currentRevision(), prompt.baseline);
  const revision = useUiRevision(() => [prompt.id, draft, error, current()]);
  const finish = (value: unknown): UiOutcome => {
    if (value !== null && !current()) { const reason = "Source changed while this prompt was open. Cancel and request a current action."; setError(reason); return { status: "unavailable", reason }; }
    onSettle(value, prompt.id);
    return { status: "completed", value: { prompt: prompt.id, settled: true, cancelled: value === null, effect: "Caller resumes; inspect actual operation result, not just prompt settlement" } };
  };
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], execute: UiCommand["execute"], enabled = true) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: enabled && (key === "read" || key === "cancel" || current()), reason: !current() ? "Source changed; obtain a new prompt" : undefined }), execute } });
  const human = (): UiOutcome => ({ status: "requires_input", request: parent, reason: "This prompt requires a human decision under its caller's policy; inspect its actual result afterward" });
  if (prompt.kind === "text") {
    add("draft", "Set prompt text draft", { text: { type: "string", maxLength: 1048576 } }, async (args, actor) => {
      if (actor.kind === "agent" && !prompt.agent) return human();
      setDraft(args.text as string); return { status: "completed", value: { draft_changed: true, submitted: false } };
    });
    add("submit", "Apply prompt text", {}, async (_args, actor) => actor.kind === "agent" && !prompt.agent ? human() : finish(draft.trim() ? draft : null));
  } else add("choice", "Choose prompt option", { index: { type: "number", minimum: 0, maximum: 1000000, integer: true } }, async (args, actor) => {
    if (actor.kind === "agent" && !prompt.agent) return human();
    const option = prompt.options[args.index as number];
    return option ? finish(option.value) : { status: "unavailable", reason: "Observe a current option index" };
  });
  add("cancel", "Cancel active prompt", {}, async () => finish(null));
  add("read", "Read active prompt bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ id: prompt.id, title: prompt.title, kind: prompt.kind, draft: prompt.kind === "text" ? draft : null, options: prompt.kind === "choice" ? prompt.options.map((o, index) => ({ index, label: o.label })) : null, source_current: current(), agent_input_allowed: prompt.agent, error }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside active prompt" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  const invoke = useUiCommands(commands);
  const click = (key: string, args: Parameters<typeof invoke>[1] = {}) => { void invoke(`${parent}:${key}`, args).catch(reason => setError(String(reason))); };
  return <div className="prompt-panel" role="dialog" aria-label={prompt.title}>
    <p className="prompt-panel__title">{prompt.title}</p>
    {error && <p role="alert">{error}</p>}
    {!current() && <p role="status">Source changed; cancel and request a current action.</p>}
    {prompt.kind === "text" ? <form onSubmit={event => { event.preventDefault(); click("submit"); }}>
      <input autoFocus value={draft} aria-label={prompt.title} onChange={event => setDraft(event.target.value)} onKeyDown={event => { if (event.key === "Escape") click("cancel"); }} />
      <button type="submit" disabled={!current()}>Apply</button><button type="button" onClick={() => click("cancel")}>Cancel</button>
    </form> : <ul className="prompt-panel__options">{prompt.options.map((option, index) => <li key={`${option.label}-${index}`}><button type="button" disabled={!current()} onClick={() => click("choice", { index })}>{option.label}</button></li>)}<li><button type="button" onClick={() => click("cancel")}>Cancel</button></li></ul>}
  </div>;
}
</storycode:copy>

### Assemble `apps/web/src/components/PromptPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/PromptPanel.tsx"><storycode:paste select="#f-0b8059fcf25a-PromptPolicy"/><storycode:paste select="#f-0b8059fcf25a-PromptBase"/><storycode:paste select="#f-0b8059fcf25a-TextPrompt"/><storycode:paste select="#f-0b8059fcf25a-ChoicePrompt"/><storycode:paste select="#f-0b8059fcf25a-Prompt"/><storycode:paste select="#f-0b8059fcf25a-base"/><storycode:paste select="#f-0b8059fcf25a-usePrompt"/><storycode:paste select="#f-0b8059fcf25a-PromptPanel"/><storycode:paste select="#f-0b8059fcf25a-ActivePrompt"/></storycode:file>

<a id="output-06cfd4483cf0"></a>
## `apps/web/src/lib/drafts.lifecycle.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Close or recover a document without losing the wrong draft](contract.md).

<a id="f-06cfd4483cf0-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-06cfd4483cf0-expression_statement">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { discardRecoveryDraft, useDraftKeeper } from "./drafts";

afterEach(() => { cleanup(); vi.restoreAllMocks(); });

</storycode:copy>
<a id="f-06cfd4483cf0-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-06cfd4483cf0-expression_statement-2">
it("waits for an unfinished recovery write and never rewrites explicitly discarded work", async () => {
  let finishWrite!: () => void;
  const writing = new Promise<void>(resolve => { finishWrite = resolve; });
  const save = vi.spyOn(api, "saveDraft").mockImplementation(async () => { await writing; return { ok: true }; });
  const discard = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const { unmount } = renderHook(() => useDraftKeeper({
    path: "note.md", read: () => ({ contents: "unsaved", base: "original" }),
  }));
  act(() => window.dispatchEvent(new Event("pagehide")));
  await waitFor(() => expect(save).toHaveBeenCalledOnce());
  const discarded = discardRecoveryDraft("note.md");
  expect(discard).not.toHaveBeenCalled();
  await act(async () => { finishWrite(); await discarded; });
  expect(discard).toHaveBeenCalledWith("note.md");
  act(() => window.dispatchEvent(new Event("pagehide")));
  unmount();
  expect(save).toHaveBeenCalledOnce();
});

</storycode:copy>
<a id="f-06cfd4483cf0-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-06cfd4483cf0-expression_statement-3">
it("queues new named recovery after an explicit discard cleanup instead of suppressing it forever", async () => {
  let contents = "old draft", finish!: () => void;
  const order: string[] = [];
  const save = vi.spyOn(api, "saveDraft").mockImplementation(async draft => { order.push(`write:${draft.contents}`); return { ok: true }; });
  const discard = vi.spyOn(api, "discardDraft").mockImplementation(async () => { await new Promise<void>(resolve => { finish = resolve; }); order.push("delete"); return { ok: true }; });
  const { unmount } = renderHook(() => useDraftKeeper({ path: "note.md", read: () => ({ contents, base: "base" }) }));
  act(() => window.dispatchEvent(new Event("pagehide"))); await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  const deleting = discardRecoveryDraft("note.md"); await waitFor(() => expect(discard).toHaveBeenCalledTimes(1));
  contents = "new retained bytes"; act(() => window.dispatchEvent(new Event("pagehide")));
  expect(save).toHaveBeenCalledTimes(1);
  await act(async () => { finish(); await deleting; });
  await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(order).toEqual(["write:old draft", "delete", "write:new retained bytes"]); unmount();
});
</storycode:copy>
<a id="f-06cfd4483cf0-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-06cfd4483cf0-expression_statement-4">
it("orders undo-to-base cleanup between an earlier recovery write and later input", async () => {
  let contents = "old draft", finish!: () => void;
  const order: string[] = []; let writes = 0;
  const save = vi.spyOn(api, "saveDraft").mockImplementation(async draft => { if (++writes === 1) await new Promise<void>(resolve => { finish = resolve; }); order.push(`write:${draft.contents}`); return { ok: true }; });
  const discard = vi.spyOn(api, "discardDraft").mockImplementation(async () => { order.push("delete"); return { ok: true }; });
  const { unmount } = renderHook(() => useDraftKeeper({ path: "note.md", read: () => ({ contents, base: "base" }) }));
  act(() => window.dispatchEvent(new Event("pagehide"))); await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  contents = "base"; act(() => window.dispatchEvent(new Event("pagehide")));
  contents = "new input"; act(() => window.dispatchEvent(new Event("pagehide")));
  expect(discard).not.toHaveBeenCalled();
  await act(async () => finish()); await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(order).toEqual(["write:old draft", "delete", "write:new input"]); unmount();
});
</storycode:copy>
<a id="f-06cfd4483cf0-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-06cfd4483cf0-expression_statement-5">
it("does not let a retired path's save cleanup suppress the replacement path's recovery", async () => {
  const save = vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
  const discard = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const sources: Record<string, string> = { "first.md": "old source", "second.md": "new source" };
  const { result, rerender, unmount } = renderHook(({ path }) => useDraftKeeper({ path, read: () => ({ contents: sources[path], base: "base" }) }), { initialProps: { path: "first.md" } });
  const oldCleanup = result.current;
  rerender({ path: "second.md" });
  act(() => oldCleanup());
  await waitFor(() => expect(discard).toHaveBeenCalledWith("first.md"));
  act(() => window.dispatchEvent(new Event("pagehide")));
  await waitFor(() => expect(save).toHaveBeenCalledWith(expect.objectContaining({ path: "second.md", contents: "new source" })));
  unmount();
});
</storycode:copy>

### Assemble `apps/web/src/lib/drafts.lifecycle.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/drafts.lifecycle.test.tsx"><storycode:paste select="#f-06cfd4483cf0-expression_statement"/><storycode:paste select="#f-06cfd4483cf0-expression_statement-2"/><storycode:paste select="#f-06cfd4483cf0-expression_statement-3"/><storycode:paste select="#f-06cfd4483cf0-expression_statement-4"/><storycode:paste select="#f-06cfd4483cf0-expression_statement-5"/></storycode:file>

<a id="output-74298466496e"></a>
## `apps/web/src/lib/drafts.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Close or recover a document without losing the wrong draft](contract.md).

<a id="f-74298466496e-DiscardRequest"></a>
### `DiscardRequest`

Unsaved work recovered after an unexpected interruption.
Explicit Discard removes recovery; Cancel leaves the buffer open.
A draft is written to the user's own data directory (never the project —
see api.md, "Workspace state and drafts") along with the BASE: the file's
contents when that editing session began. The base is what makes coming
back safe rather than merely possible, because the file may have moved on
while the app was closed — a branch checked out, a `git pull`, an edit in
another editor. With a base, that is a three-way merge that resolves every
region only one side touched. Without one it would be a standoff.
This module is the DECISION and the plumbing around it. The merge itself is
lib/merge.ts, and the UI that asks about conflicts is
components/MergeView.tsx.

<storycode:copy id="f-74298466496e-DiscardRequest">
// Unsaved work recovered after an unexpected interruption.
// Explicit Discard removes recovery; Cancel leaves the buffer open.
//
// A draft is written to the user's own data directory (never the project —
// see api.md, "Workspace state and drafts") along with the BASE: the file's
// contents when that editing session began. The base is what makes coming
// back safe rather than merely possible, because the file may have moved on
// while the app was closed — a branch checked out, a `git pull`, an edit in
// another editor. With a base, that is a three-way merge that resolves every
// region only one side touched. Without one it would be a standoff.
//
// This module is the DECISION and the plumbing around it. The merge itself is
// lib/merge.ts, and the UI that asks about conflicts is
// components/MergeView.tsx.

import { useCallback, useEffect, useRef } from "react";

import { api } from "../api/client";
import type { WorkspaceDraft } from "../api/types";

type DiscardRequest = { path: string; waitUntil: (operation: Promise<unknown>) => void };
</storycode:copy>
<a id="f-74298466496e-discardRecoveryDraft"></a>
### `discardRecoveryDraft`

<storycode:copy id="f-74298466496e-discardRecoveryDraft">
export async function discardRecoveryDraft(path: string) {
  const pending: Promise<unknown>[] = [];
  window.dispatchEvent(new CustomEvent<DiscardRequest>("hickory-draft-discarded", {
    detail: { path, waitUntil: operation => pending.push(operation) },
  }));
  if (pending.length) await Promise.all(pending);
  else await api.discardDraft(path);
}

</storycode:copy>
<a id="f-74298466496e-Disposition"></a>
### `Disposition`

What to do with a draft, given what the file says now.

<storycode:copy id="f-74298466496e-Disposition">
/** What to do with a draft, given what the file says now. */
export type Disposition =
  /** The buffer and the file agree — there was nothing unsaved after all.
   * The draft is stale and should be discarded. */
  | { kind: "clean" }
  /** The file is exactly what the draft was taken from: nobody else touched
   * it. Put the draft in the buffer, still unsaved, and say nothing. */
  | { kind: "restore"; contents: string }
  /** The file moved on while we were away. Both sides have changes, and the
   * reader has to see them. */
  | { kind: "merge"; base: string; ours: string; theirs: string };

</storycode:copy>
<a id="f-74298466496e-draftDisposition"></a>
### `draftDisposition`

What should happen to `draft` now that the file says `onDisk`.
The order of these tests matters. "Clean" is checked first because a draft
whose contents match the file is not a conflict however far the file has
moved — somebody saved the same text from somewhere else, and there is
nothing to merge or to warn about.

<storycode:copy id="f-74298466496e-draftDisposition">
/**
 * What should happen to `draft` now that the file says `onDisk`.
 *
 * The order of these tests matters. "Clean" is checked first because a draft
 * whose contents match the file is not a conflict however far the file has
 * moved — somebody saved the same text from somewhere else, and there is
 * nothing to merge or to warn about.
 */
export function draftDisposition(
  draft: Pick<WorkspaceDraft, "contents" | "base">,
  onDisk: string,
): Disposition {
  if (draft.contents === onDisk) return { kind: "clean" };
  if (draft.base === onDisk) return { kind: "restore", contents: draft.contents };
  return { kind: "merge", base: draft.base, ours: draft.contents, theirs: onDisk };
}

</storycode:copy>
<a id="f-74298466496e-DRAFT_POLL_MS"></a>
### `DRAFT_POLL_MS`

How often a changed buffer is written down.

<storycode:copy id="f-74298466496e-DRAFT_POLL_MS">
/** How often a changed buffer is written down. */
export const DRAFT_POLL_MS = 2000;

</storycode:copy>
<a id="f-74298466496e-useDraftKeeper"></a>
### `useDraftKeeper`

Save changed buffers periodically and at `pagehide`, not only at graceful shutdown. An explicit discard waits behind pending writes and suppresses the same buffer, so an unfinished save cannot resurrect discarded recovery data.

<storycode:copy id="f-74298466496e-useDraftKeeper">
/**
 * Keep a draft of one buffer.
 *
 * The buffer is READ through a callback rather than passed in as a prop, and
 * that is the whole design of this hook. An editor pane deliberately does not
 * re-render on every keystroke — the CodeMirror view owns the text — so there
 * is no prop that changes when the buffer does, and forcing one would put a
 * React render on the typing path to power a background save.
 *
 * So it polls, on a slow timer, and writes only when the text has actually
 * moved since the last write. Two things fall out of that which a
 * debounced-on-change version would not give:
 *
 *  - It covers a CRASH, not just a graceful close. A draft written only at
 *    shutdown is a draft that is not there after the one event most likely to
 *    lose work.
 *  - It costs nothing while nobody is typing: an unchanged buffer is a string
 *    comparison, not a request.
 *
 * `pagehide` flushes as well, because it is the last moment the page reliably
 * gets, and a poll due in a second's time would never fire.
 *
 * `contents === base` means nothing is unsaved, and the draft is DISCARDED
 * rather than written — otherwise every file ever opened would accumulate a
 * draft identical to itself, and every start would have to filter them out.
 */
export function useDraftKeeper({
  path,
  read,
  enabled = true,
}: {
  /** Project-relative path. Nothing is written without one. */
  path: string | null;
  /** The buffer as it stands, and what the file held when this session began. */
  read: () => { contents: string; base: string };
  enabled?: boolean;
}): () => void {
  const readRef = useRef(read);
  readRef.current = read;
  const activePath = useRef(path); activePath.current = path;
  // A buffer can become a real file immediately before its component
  // unmounts. Remember that transition outside React state, so the cleanup
  // below cannot race it by writing the just-saved buffer back as a draft.
  const discardedRef = useRef(false);
  const discardedContents = useRef<string | null>(null);

  useEffect(() => {
    if (!enabled || !path) return;
    discardedRef.current = false;
    // What we last put in the store, so an untouched buffer costs a string
    // comparison rather than a request.
    let written: string | null = null;
    let pending: Promise<unknown> = Promise.resolve();
    const onDiscarded = (event: Event) => {
      const request = (event as CustomEvent<DiscardRequest>).detail;
      if (request.path !== path) return;
      discardedRef.current = true; discardedContents.current = readRef.current().contents;
      pending = pending.catch(() => {}).then(() => api.discardDraft(path));
      request.waitUntil(pending);
    };

    const flush = () => {
      if (activePath.current !== path) return;
      const { contents, base } = readRef.current();
      if (discardedRef.current) {
        if (contents === discardedContents.current) return;
        discardedRef.current = false; written = null;
      }
      if (contents === base) {
        if (written !== null) {
          written = null;
          pending = pending.catch(() => {}).then(() => api.discardDraft(path)).catch(() => { written = contents; });
        }
        return;
      }
      if (contents === written) return;
      written = contents;
      pending = pending.catch(() => {}).then(() => api.saveDraft({ path, contents, base, saved_at: Date.now() })).catch(() => {
        // A draft that cannot be written must not interrupt typing. The route
        // already degrades loudly; here the honest response is to let the
        // editor keep working, and to try again on the next tick.
        written = null;
      });
    };

    const timer = window.setInterval(flush, DRAFT_POLL_MS);
    // `pagehide` rather than `beforeunload`: it fires for a tab being frozen
    // or discarded as well as closed, and it is the event WebKit delivers.
    window.addEventListener("pagehide", flush);
    window.addEventListener("hickory-draft-discarded", onDiscarded);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("pagehide", flush);
      window.removeEventListener("hickory-draft-discarded", onDiscarded);
      // Keep recovery on unmount unless an explicit Discard suppressed it.
      flush();
    };
  }, [path, enabled]);

  return useCallback(() => {
    if (!path) return;
    if (activePath.current === path) { discardedRef.current = true; discardedContents.current = readRef.current().contents; }
    void discardRecoveryDraft(path).catch(() => {});
  }, [path]);
}
</storycode:copy>

### Assemble `apps/web/src/lib/drafts.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/drafts.ts"><storycode:paste select="#f-74298466496e-DiscardRequest"/><storycode:paste select="#f-74298466496e-discardRecoveryDraft"/><storycode:paste select="#f-74298466496e-Disposition"/><storycode:paste select="#f-74298466496e-draftDisposition"/><storycode:paste select="#f-74298466496e-DRAFT_POLL_MS"/><storycode:paste select="#f-74298466496e-useDraftKeeper"/></storycode:file>

<a id="output-972530db3158"></a>
## `apps/web/src/lib/windowClose.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Close or recover a document without losing the wrong draft](contract.md).

<a id="f-972530db3158-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-972530db3158-expression_statement">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
import { expect, it, vi } from "vitest";
import { requestApplicationClose, requestWorkspaceClose, WORKSPACE_CLOSE, type WorkspaceCloseRequest } from "./windowClose";

it("requires a mounted workspace owner and forwards its real result and cancellation scope", async () => {
  expect(await requestWorkspaceClose()).toMatchObject({ status: "unavailable", reason: expect.stringContaining("No fallback") });
  const controller = new AbortController(); let actual: AbortSignal | undefined;
  const listener = (event: Event) => {
    const request = (event as CustomEvent<WorkspaceCloseRequest>).detail;
    request.accept(); actual = request.signal;
    request.respond({ status: "unavailable", reason: "actual source changed" });
  };
  window.addEventListener(WORKSPACE_CLOSE, listener);
  try { expect(await requestWorkspaceClose(controller.signal)).toMatchObject({ status: "unavailable", reason: "actual source changed" }); expect(actual).toBe(controller.signal); }
  finally { window.removeEventListener(WORKSPACE_CLOSE, listener); }
});
</storycode:copy>
<a id="f-972530db3158-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-972530db3158-expression_statement-2">
it("reveals the retained workspace and never falls back past its refusal", async () => {
  const reveal = vi.fn(), hostClose = vi.fn(async () => ({ ok: true as const }));
  const workspaceClose = vi.fn(async () => ({ status: "unavailable" as const, reason: "save refused" }));
  expect(await requestApplicationClose({ workspace: "/docs/kept", inWorkspace: false, reveal, workspaceClose, hostClose })).toMatchObject({ status: "unavailable", reason: "save refused" });
  expect(reveal).toHaveBeenCalledExactlyOnceWith("/docs/kept"); expect(workspaceClose).toHaveBeenCalledTimes(1); expect(hostClose).not.toHaveBeenCalled();
  workspaceClose.mockRejectedValue(Error("owner retired"));
  expect(await requestApplicationClose({ workspace: "/docs/kept", inWorkspace: true, reveal, workspaceClose, hostClose })).toMatchObject({ status: "unavailable", reason: expect.stringContaining("without fallback") });
  expect(hostClose).not.toHaveBeenCalled(); expect(reveal).toHaveBeenCalledTimes(1);
});
</storycode:copy>
<a id="f-972530db3158-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-972530db3158-expression_statement-3">
it("reports only a valid host acknowledgement when no workspace exists", async () => {
  const options = { workspace: null, inWorkspace: false, reveal: vi.fn(), workspaceClose: vi.fn(), hostClose: vi.fn(async () => ({ ok: true as const })) };
  expect(await requestApplicationClose(options)).toMatchObject({ status: "completed", value: { host_close_request_acknowledged: true, window_destruction_observed: false } });
  options.hostClose.mockResolvedValue({ ok: false } as unknown as { ok: true });
  expect(await requestApplicationClose(options)).toMatchObject({ status: "unavailable", reason: expect.stringContaining("Invalid host close acknowledgement") });
  const controller = new AbortController(); controller.abort();
  expect(await requestApplicationClose({ ...options, signal: controller.signal })).toMatchObject({ status: "completed", value: { cancelled: true, host_close_submitted: false } });
  expect(options.hostClose).toHaveBeenCalledTimes(2); expect(options.workspaceClose).not.toHaveBeenCalled();
});
</storycode:copy>

### Assemble `apps/web/src/lib/windowClose.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/windowClose.test.ts"><storycode:paste select="#f-972530db3158-expression_statement"/><storycode:paste select="#f-972530db3158-expression_statement-2"/><storycode:paste select="#f-972530db3158-expression_statement-3"/></storycode:file>

<a id="output-e5e25a4f8e9b"></a>
## `apps/web/src/lib/windowClose.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Close or recover a document without losing the wrong draft](contract.md).

<a id="f-e5e25a4f8e9b-WORKSPACE_CLOSE"></a>
### `WORKSPACE_CLOSE`

<storycode:copy id="f-e5e25a4f8e9b-WORKSPACE_CLOSE">
import type { UiOutcome } from "./uiActions";

export const WORKSPACE_CLOSE = "hickory-workspace-close-request";
</storycode:copy>
<a id="f-e5e25a4f8e9b-WorkspaceCloseRequest"></a>
### `WorkspaceCloseRequest`

<storycode:copy id="f-e5e25a4f8e9b-WorkspaceCloseRequest">
export interface WorkspaceCloseRequest {
  signal?: AbortSignal;
  accept(): void;
  respond(outcome: UiOutcome): void;
}

</storycode:copy>
<a id="f-e5e25a4f8e9b-requestWorkspaceClose"></a>
### `requestWorkspaceClose`

An actual mounted workspace accepts responsibility synchronously, then replies.

<storycode:copy id="f-e5e25a4f8e9b-requestWorkspaceClose">
/** An actual mounted workspace accepts responsibility synchronously, then replies. */
export function requestWorkspaceClose(signal?: AbortSignal, target: Window = window): Promise<UiOutcome> {
  return new Promise(resolve => {
    let accepted = false;
    target.dispatchEvent(new CustomEvent<WorkspaceCloseRequest>(WORKSPACE_CLOSE, {
      detail: { signal, accept: () => { accepted = true; }, respond: resolve },
    }));
    if (!accepted) resolve({ status: "unavailable", reason: "The workspace close owner is not mounted. No fallback host close was submitted; reopen the workspace and retry." });
  });
}

</storycode:copy>
<a id="f-e5e25a4f8e9b-requestApplicationClose"></a>
### `requestApplicationClose`

Navigation reveals existing guarded prompts; it never bypasses their owner.

<storycode:copy id="f-e5e25a4f8e9b-requestApplicationClose">
/** Navigation reveals existing guarded prompts; it never bypasses their owner. */
export async function requestApplicationClose(options: {
  workspace: string | null; inWorkspace: boolean; signal?: AbortSignal;
  reveal(path: string): void; workspaceClose(signal?: AbortSignal): Promise<UiOutcome>;
  hostClose(): Promise<{ ok: true }>;
}): Promise<UiOutcome> {
  if (options.signal?.aborted) return { status: "completed", value: { cancelled: true, host_close_submitted: false } };
  if (options.workspace !== null) {
    try {
      if (!options.inWorkspace) options.reveal(options.workspace);
      return await options.workspaceClose(options.signal);
    } catch (error) {
      return { status: "unavailable", reason: `Workspace close failed without fallback: ${error instanceof Error ? error.message : String(error)}` };
    }
  }
  try {
    const reply = await options.hostClose();
    if (reply?.ok !== true) throw Error("Invalid host close acknowledgement");
    return { status: "completed", value: { host_close_request_acknowledged: true, window_destruction_observed: false,
      owner_cancellation_requested: !!options.signal?.aborted, scope: "Acknowledged callback only; window shutdown can prevent response delivery. A lost response does not establish closure or rollback." } };
  } catch (error) {
    return { status: "unavailable", reason: `Host close success is unacknowledged: ${error instanceof Error ? error.message : String(error)}. The window may already be closing.` };
  }
}
</storycode:copy>

### Assemble `apps/web/src/lib/windowClose.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/windowClose.ts"><storycode:paste select="#f-e5e25a4f8e9b-WORKSPACE_CLOSE"/><storycode:paste select="#f-e5e25a4f8e9b-WorkspaceCloseRequest"/><storycode:paste select="#f-e5e25a4f8e9b-requestWorkspaceClose"/><storycode:paste select="#f-e5e25a4f8e9b-requestApplicationClose"/></storycode:file>

<a id="output-c3dca2adcf89"></a>
## `apps/web/src/shell/ShellView.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Close or recover a document without losing the wrong draft](contract.md).

<a id="f-c3dca2adcf89-expression_statement"></a>
### `expression_statement`

@vitest-environment jsdom
The tab strip's pointer gestures, at the level only a real DOM answers:
middle-click closes the tab it lands on, whatever kind of tab it is, and
the button that closes must not also start a drag.

<storycode:copy id="f-c3dca2adcf89-expression_statement">
// @vitest-environment jsdom
//
// The tab strip's pointer gestures, at the level only a real DOM answers:
// middle-click closes the tab it lands on, whatever kind of tab it is, and
// the button that closes must not also start a drag.

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { ShellView } from "./ShellView";
import { freeform, open, tab, type Layout } from "./layout";

afterEach(cleanup);

</storycode:copy>
<a id="f-c3dca2adcf89-shell"></a>
### `shell`

Renders a shell over `layout` and reports the layout it hands back.

<storycode:copy id="f-c3dca2adcf89-shell">
/** Renders a shell over `layout` and reports the layout it hands back. */
function shell(layout: Layout) {
  const seen: Layout[] = [];
  const { container } = render(
    <ShellView
      layout={layout}
      onLayout={(next) => seen.push(next)}
      render={(t) => <div>{t.target}</div>}
    />,
  );
  /** The tab in the strip whose file is `target`. */
  const strip = (target: string) =>
    container.querySelector<HTMLElement>(`.shell-tab[data-shell-tab-target="${target}"]`)!;
  return { seen, strip };
}

</storycode:copy>
<a id="f-c3dca2adcf89-pointer"></a>
### `pointer`

jsdom has no PointerEvent; a MouseEvent carries every field read here.

<storycode:copy id="f-c3dca2adcf89-pointer">
/** jsdom has no PointerEvent; a MouseEvent carries every field read here. */
function pointer(el: Element, type: "pointerdown" | "pointerup", button: number) {
  fireEvent(el, new MouseEvent(type, { bubbles: true, cancelable: true, button }));
}

</storycode:copy>
<a id="f-c3dca2adcf89-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c3dca2adcf89-expression_statement-2">
describe("middle-click closes a tab", () => {
  it.each([
    ["a document", tab("document", "notes.md")],
    ["a terminal", tab("terminal", "term:1")],
    ["a plain file", tab("file", "README.md")],
  ])("closes %s", (_name, closing) => {
    const layout = open(open(freeform(), tab("document", "keep.md")), closing);
    const { seen, strip } = shell(layout);

    const el = strip(closing.target);
    pointer(el, "pointerdown", 1);
    pointer(el, "pointerup", 1);

    const next = seen.at(-1)!;
    expect(next.root.type === "pane" && next.root.tabs.map((t) => t.target)).toEqual([
      "keep.md",
    ]);
  });

  it("takes no notice of the right button", () => {
    const layout = open(open(freeform(), tab("document", "a.md")), tab("document", "b.md"));
    const { seen, strip } = shell(layout);

    pointer(strip("a.md"), "pointerdown", 2);
    pointer(strip("a.md"), "pointerup", 2);
    expect(seen).toHaveLength(0);
  });

  it("does not close the tab a press merely drifted onto", () => {
    const layout = open(open(freeform(), tab("document", "a.md")), tab("document", "b.md"));
    const { seen, strip } = shell(layout);

    pointer(strip("a.md"), "pointerdown", 1);
    pointer(strip("b.md"), "pointerup", 1);
    expect(seen).toHaveLength(0);
  });
});

</storycode:copy>
<a id="f-c3dca2adcf89-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c3dca2adcf89-expression_statement-3">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
describe("unsaved tabs", () => {
  it("draws the data-driven asterisk and asks its owner before closing", () => {
    const dirty = tab("untitled", "untitled", "Untitled");
    const layout = open(freeform(), dirty);
    const request = vi.fn();
    const { getByRole } = render(
      <ShellView
        layout={layout}
        onLayout={() => {}}
        render={() => null}
        dirtyTabIds={new Set([dirty.id])}
        onRequestCloseTab={request}
      />,
    );
    expect(getByRole("tab").textContent).toBe("Untitled *");
    fireEvent.click(getByRole("button", { name: "Close Untitled" }));
    expect(request).toHaveBeenCalledWith(layout.focus, dirty.id);
  });
});
</storycode:copy>

### Assemble `apps/web/src/shell/ShellView.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/ShellView.test.tsx"><storycode:paste select="#f-c3dca2adcf89-expression_statement"/><storycode:paste select="#f-c3dca2adcf89-shell"/><storycode:paste select="#f-c3dca2adcf89-pointer"/><storycode:paste select="#f-c3dca2adcf89-expression_statement-2"/><storycode:paste select="#f-c3dca2adcf89-expression_statement-3"/></storycode:file>

<a id="output-1eaf7867608d"></a>
## `apps/web/src/views/useUntitledRecovery.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Close or recover a document without losing the wrong draft](contract.md).

<a id="f-1eaf7867608d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-1eaf7867608d-expression_statement">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { freeform, panes } from "../shell/layout";
import { openUntitledTab } from "./workspaceState";
import { useUntitledRecovery } from "./useUntitledRecovery";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-1eaf7867608d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-1eaf7867608d-expression_statement-2">
it.each(["changed bytes", "replaced tab"])("resumes unnamed recovery after discard for %s, ordered after cleanup", async change => {
  const layout = openUntitledTab(freeform()), pane = panes(layout.root)[0], entry = pane.tabs[0], layoutRef = { current: layout };
  const sourcesRef = { current: { [entry.id]: "captured draft" } };
  vi.spyOn(api, "drafts").mockResolvedValue({ drafts: [] });
  const order: string[] = [];
  const save = vi.spyOn(api, "saveDraft").mockImplementation(async draft => { order.push(`write:${draft.contents}`); return { ok: true }; });
  let finish!: () => void;
  const discard = vi.spyOn(api, "discardDraft").mockImplementation(async () => { await new Promise<void>(resolve => { finish = resolve; }); order.push("delete"); return { ok: true }; });
  const setLayout = vi.fn(), setSources = vi.fn(), initialBaselines = {};
  const { result, unmount } = renderHook(() => useUntitledRecovery({ hydrated: true, layoutRef, sourcesRef, setLayout, setSources, initialBaselines }));
  await waitFor(() => expect(result.current.ready).toBe(true));
  act(() => window.dispatchEvent(new Event("pagehide"))); await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  const work: Promise<unknown>[] = [];
  act(() => window.dispatchEvent(new CustomEvent("hickory-untitled-discarded", { detail: { id: entry.id, waitUntil: (operation: Promise<unknown>) => work.push(operation) } })));
  await waitFor(() => expect(discard).toHaveBeenCalledTimes(1));
  if (change === "changed bytes") sourcesRef.current[entry.id] = "new draft";
  else layoutRef.current = { ...layout, root: { ...pane, tabs: pane.tabs.map(tab => ({ ...tab })) } };
  act(() => window.dispatchEvent(new Event("pagehide"))); expect(save).toHaveBeenCalledTimes(1);
  await act(async () => { finish(); await Promise.all(work); }); await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(order).toEqual(["write:captured draft", "delete", `write:${change === "changed bytes" ? "new draft" : "captured draft"}`]); unmount();
});
</storycode:copy>

### Assemble `apps/web/src/views/useUntitledRecovery.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useUntitledRecovery.test.tsx"><storycode:paste select="#f-1eaf7867608d-expression_statement"/><storycode:paste select="#f-1eaf7867608d-expression_statement-2"/></storycode:file>

<a id="output-ec052e0ed871"></a>
## `apps/web/src/views/useUntitledRecovery.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [New Document Creates Another Unnamed Buffer](../new-document-is-an-act/contract.md), [Close or recover a document without losing the wrong draft](contract.md).

<a id="f-ec052e0ed871-reidentify"></a>
### `reidentify`

Each unnamed buffer keeps its own recovery draft outside the project.

<storycode:copy id="f-ec052e0ed871-reidentify">
// Each unnamed buffer keeps its own recovery draft outside the project.
import { useEffect, useRef, useState } from "react";
import type { Dispatch, MutableRefObject, SetStateAction } from "react";
import { api } from "../api/client";
import { DRAFT_POLL_MS } from "../lib/drafts";
import { UNTITLED_RECOVERY_KEY, untitledDraftKey } from "../lib/newDoc";
import { open, panes, type Layout, type Node } from "../shell/layout";
import { openUntitledTab } from "./workspaceState";

function reidentify(layout: Layout, oldId: string, id: string): Layout {
  const visit = (node: Node): Node => node.type === "pane"
    ? { ...node, tabs: node.tabs.map(tab => tab.id === oldId ? { ...tab, id } : tab) }
    : { ...node, children: node.children.map(visit) };
  return { ...layout, root: visit(layout.root) };
}

</storycode:copy>
<a id="f-ec052e0ed871-useUntitledRecovery"></a>
### `useUntitledRecovery`

<storycode:copy id="f-ec052e0ed871-useUntitledRecovery">
export function useUntitledRecovery({ hydrated, layoutRef, setLayout, sourcesRef, setSources, initialBaselines }: {
  hydrated: boolean;
  layoutRef: MutableRefObject<Layout>;
  setLayout: Dispatch<SetStateAction<Layout>>;
  sourcesRef: MutableRefObject<Record<string, string>>;
  setSources: Dispatch<SetStateAction<Record<string, string>>>;
  initialBaselines: Record<string, string>;
}) {
  const [ready, setReady] = useState(false);
  const clearClean = useRef<(() => void) | null>(null);
  const [baselines, setBaselines] = useState(initialBaselines);
  const baselinesRef = useRef(baselines);
  baselinesRef.current = baselines;
  useEffect(() => {
    if (!hydrated) return;
    let cancelled = false;
    void api.drafts().then(({ drafts }) => {
      if (cancelled) return;
      let next = layoutRef.current;
      const identities = new Map<string, string>();
      const sources: Record<string, string> = {};
      const bases: Record<string, string> = {};
      for (const draft of drafts.filter(draft => draft.path.startsWith("untitled:") && draft.contents !== draft.base)) {
        const id = draft.path.slice("untitled:".length);
        let tab = panes(next.root).flatMap(pane => pane.tabs).find(tab => tab.kind === "untitled" && tab.id === id);
        // Older versions kept only one unnamed draft. Adopt that into the
        // startup buffer without overwriting anything typed during hydration.
        if (draft.path === UNTITLED_RECOVERY_KEY) tab = panes(next.root).flatMap(pane => pane.tabs).find(tab => tab.kind === "untitled");
        // The startup introduction is a fallback, not another recovered
        // document. Reuse it for the first draft even when that draft began
        // blank or the bundled introduction changed since the previous run.
        if (!tab && Object.keys(sources).length === 0) {
          const startup = panes(next.root).flatMap(pane => pane.tabs).find(tab => tab.kind === "untitled" && tab.id in baselinesRef.current && (sourcesRef.current[tab.id] ?? baselinesRef.current[tab.id]) === baselinesRef.current[tab.id]);
          if (startup) {
            identities.set(startup.id, id);
            next = reidentify(next, startup.id, id);
            tab = { ...startup, id };
          }
        }
        if (!tab) {
          next = openUntitledTab(next, true);
          const pane = panes(next.root).find(pane => pane.id === next.focus)!;
          tab = pane.tabs[pane.active];
          if (draft.path !== UNTITLED_RECOVERY_KEY) {
            identities.set(tab.id, id);
            next = reidentify(next, tab.id, id);
            tab = { ...tab, id };
          }
        }
        const current = sourcesRef.current[tab.id];
        if (current !== undefined && current !== (baselinesRef.current[tab.id] ?? "")) continue;
        sources[tab.id] = draft.contents;
        bases[tab.id] = draft.base;
        if (draft.path === UNTITLED_RECOVERY_KEY) {
          void api.saveDraft({ ...draft, path: untitledDraftKey(tab.id) }).then(() => api.discardDraft(UNTITLED_RECOVERY_KEY)).catch(() => {});
        }
      }
      if (Object.keys(sources).length === 0) return;
      setBaselines(previous => ({ ...previous, ...bases }));
      setSources(previous => ({ ...previous, ...sources }));
      const recovered = panes(next.root).flatMap(pane => pane.tabs).filter(tab => tab.id in sources);
      setLayout(current => {
        // Folder hydration can have added Files while drafts were loading.
        // Apply recovered buffers to that live arrangement, never replace it
        // with the earlier snapshot and lose newly opened panes.
        let merged = current;
        for (const [oldId, id] of identities) merged = reidentify(merged, oldId, id);
        for (const tab of recovered) {
          if (panes(merged.root).some(pane => pane.tabs.some(entry => entry.id === tab.id))) continue;
          const editor = panes(merged.root).find(pane => !pane.tabs.some(entry => entry.kind === "tree"));
          merged = open(merged, tab, editor?.id);
        }
        layoutRef.current = merged;
        return merged;
      });
    }).catch(() => {}).finally(() => { if (!cancelled) setReady(true); });
    return () => { cancelled = true; };
  }, [hydrated, layoutRef, setLayout, sourcesRef, setSources]);

  useEffect(() => {
    if (!ready) return;
    const written = new Map<string, string>();
    const saved = new Map<string, { contents: string; tab: object | undefined }>();
    const suppress = (id: string) => saved.set(id, { contents: sourcesRef.current[id] ?? baselinesRef.current[id] ?? "", tab: panes(layoutRef.current.root).flatMap(pane => pane.tabs).find(tab => tab.id === id) });
    const pending = new Map<string, Promise<unknown>>();
    const cleaned = new Map<string, { base: string; tab: object }>();
    const clear = (id: string, base: string, tab: object) => {
      if (cleaned.get(id)?.base === base && cleaned.get(id)?.tab === tab) return;
      cleaned.set(id, { base, tab }); written.delete(id);
      const cleanup = (pending.get(id) ?? Promise.resolve()).catch(() => {}).then(() => api.discardDraft(untitledDraftKey(id))).catch(() => { cleaned.delete(id); });
      pending.set(id, cleanup);
    };
    const clearCurrent = () => {
      for (const tab of panes(layoutRef.current.root).flatMap(pane => pane.tabs).filter(tab => tab.kind === "untitled")) {
        const base = baselinesRef.current[tab.id] ?? "";
        if ((sourcesRef.current[tab.id] ?? base) === base) clear(tab.id, base, tab); else cleaned.delete(tab.id);
      }
    };
    clearClean.current = clearCurrent;
    const onSaved = (event: Event) => {
      const id = (event as CustomEvent<string>).detail;
      suppress(id);
      const cleanup = (pending.get(id) ?? Promise.resolve()).catch(() => {}).then(() => api.discardDraft(untitledDraftKey(id)));
      pending.set(id, cleanup); void cleanup.catch(() => {});
    };
    const onDiscarded = (event: Event) => {
      const { id, waitUntil } = (event as CustomEvent<{ id: string; waitUntil: (operation: Promise<unknown>) => void }>).detail;
      suppress(id);
      const cleanup = (pending.get(id) ?? Promise.resolve()).catch(() => {}).then(() => api.discardDraft(untitledDraftKey(id)));
      pending.set(id, cleanup); waitUntil(cleanup);
    };
    const flush = () => {
      for (const tab of panes(layoutRef.current.root).flatMap(pane => pane.tabs).filter(tab => tab.kind === "untitled")) {
        const suppressed = saved.get(tab.id);
        if (suppressed?.tab === tab && suppressed.contents === (sourcesRef.current[tab.id] ?? baselinesRef.current[tab.id] ?? "")) continue;
        if (suppressed) { saved.delete(tab.id); written.delete(tab.id); }
        const base = baselinesRef.current[tab.id] ?? "";
        const contents = sourcesRef.current[tab.id] ?? base;
        if (contents === base) clear(tab.id, base, tab);
        else if (written.get(tab.id) !== contents) {
          cleaned.delete(tab.id);
          written.set(tab.id, contents);
          const write = (pending.get(tab.id) ?? Promise.resolve()).then(() =>
            api.saveDraft({ path: untitledDraftKey(tab.id), contents, base, saved_at: Date.now() })
          ).catch(() => written.delete(tab.id));
          pending.set(tab.id, write);
        }
      }
    };
    const timer = window.setInterval(flush, DRAFT_POLL_MS);
    window.addEventListener("pagehide", flush);
    window.addEventListener("hickory-untitled-saved", onSaved);
    window.addEventListener("hickory-untitled-discarded", onDiscarded);
    return () => {
      if (clearClean.current === clearCurrent) clearClean.current = null;
      window.clearInterval(timer);
      window.removeEventListener("pagehide", flush);
      window.removeEventListener("hickory-untitled-saved", onSaved);
      window.removeEventListener("hickory-untitled-discarded", onDiscarded);
      flush();
    };
  }, [ready, layoutRef, sourcesRef]);
  // Serialize undo-to-base cleanup with earlier writes and later new drafts.
  useEffect(() => { if (ready) clearClean.current?.(); });
  return { ready, baselines };
}
</storycode:copy>

### Assemble `apps/web/src/views/useUntitledRecovery.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useUntitledRecovery.ts"><storycode:paste select="#f-ec052e0ed871-reidentify"/><storycode:paste select="#f-ec052e0ed871-useUntitledRecovery"/></storycode:file>

## Test the behavior: `retaining_saved_file_drafts_is_off_by_default_and_persists`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_ui_settings.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-c816a698604d-retaining_saved_file_drafts_is_off_by_default_and_persists"></a>
### `retaining_saved_file_drafts_is_off_by_default_and_persists`

<storycode:copy id="f-c816a698604d-retaining_saved_file_drafts_is_off_by_default_and_persists">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
#[tokio::test(flavor = "multi_thread")]
async fn retaining_saved_file_drafts_is_off_by_default_and_persists() {
    let session = start().await;
    let (_, initial) = get(&session, "/api/settings/ui").await;
    assert_eq!(initial["retain_unsaved_saved_files"], false);

    let (status, changed) = put(
        &session,
        "/api/settings/ui",
        json!({ "retain_unsaved_saved_files": true }),
    )
    .await;
    assert_eq!(status, 200, "{changed}");
    assert_eq!(changed["retain_unsaved_saved_files"], true);
    assert!(
        UiStore::load(&session.ui_path)
            .unwrap()
            .retain_unsaved_saved_files
    );
}

</storycode:copy>

<a id="output-4fb4173715bd"></a>
## `crates/hickory-cli/tests/serve_workspace.rs`

Related contracts: [Close or recover a document without losing the wrong draft](contract.md).

<a id="f-4fb4173715bd-DOC"></a>
### `DOC`

What the window remembers: which tabs were open, and which buffers had
unsaved changes when it closed.
Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
Driven the way the page drives it — real HTTP against the real router, no
in-process shortcut (`.instructions/framework-agnostic-system-tests.md`).
The state directory is named through `HICKORY_STATE_DIR` so this test can
prove a draft round-trips without writing into the developer's home.

<storycode:copy id="f-4fb4173715bd-DOC">
//! What the window remembers: which tabs were open, and which buffers had
//! unsaved changes when it closed.
//!
//! Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
//!
//! Driven the way the page drives it — real HTTP against the real router, no
//! in-process shortcut (`.instructions/framework-agnostic-system-tests.md`).
//! The state directory is named through `HICKORY_STATE_DIR` so this test can
//! prove a draft round-trips without writing into the developer's home.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="demo.md">
# Demo
</hick:doc>
"##;

</storycode:copy>
<a id="f-4fb4173715bd-Session"></a>
### `Session`

<storycode:copy id="f-4fb4173715bd-Session">
struct Session {
    base: String,
    project: std::path::PathBuf,
    _dir: tempfile::TempDir,
}

</storycode:copy>
<a id="f-4fb4173715bd-state_dir"></a>
### `state_dir`

One state directory for the whole binary, set once.
`HICKORY_STATE_DIR` is process-global, so a per-test directory would be a
race: these tests run concurrently, and whichever set the variable last
would own everybody's writes. Isolation comes from the store's own key
instead — each test gets its own project folder, and the store is keyed by
the project's canonical path, so two tests can never see each other's
layout even though they share a root.

<storycode:copy id="f-4fb4173715bd-state_dir">
/// One state directory for the whole binary, set once.
///
/// `HICKORY_STATE_DIR` is process-global, so a per-test directory would be a
/// race: these tests run concurrently, and whichever set the variable last
/// would own everybody's writes. Isolation comes from the store's own key
/// instead — each test gets its own project folder, and the store is keyed by
/// the project's canonical path, so two tests can never see each other's
/// layout even though they share a root.
fn state_dir() -> &'static std::path::Path {
    static STATE: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    let dir = STATE.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        // SAFETY: written exactly once, inside OnceLock's initialisation, and
        // nothing else in this binary touches this variable.
        unsafe { std::env::set_var(hickory_workspace::STATE_DIR_VAR, dir.path()) };
        dir
    });
    dir.path()
}

</storycode:copy>
<a id="f-4fb4173715bd-start"></a>
### `start`

<storycode:copy id="f-4fb4173715bd-start">
async fn start() -> Session {
    let _ = state_dir();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.md"), DOC).unwrap();
    let root = dir.path().canonicalize().unwrap();

    let prepared = prepare(ServeOptions {
        target: root.join("demo.md"),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        project: root,
        _dir: dir,
    }
}

</storycode:copy>
<a id="f-4fb4173715bd-send"></a>
### `send`

<storycode:copy id="f-4fb4173715bd-send">
async fn send(
    session: &Session,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
) -> (u16, Value) {
    let mut req = reqwest::Client::new().request(method, format!("{}{path}", session.base));
    if let Some(body) = body {
        req = req.json(&body);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

</storycode:copy>
<a id="f-4fb4173715bd-get"></a>
### `get`

<storycode:copy id="f-4fb4173715bd-get">
async fn get(session: &Session, path: &str) -> (u16, Value) {
    send(session, reqwest::Method::GET, path, None).await
}

</storycode:copy>
<a id="f-4fb4173715bd-put"></a>
### `put`

<storycode:copy id="f-4fb4173715bd-put">
async fn put(session: &Session, path: &str, body: Value) -> (u16, Value) {
    send(session, reqwest::Method::PUT, path, Some(body)).await
}

</storycode:copy>
<a id="f-4fb4173715bd-a_window_layout_round_trips"></a>
### `a_window_layout_round_trips`

<storycode:copy id="f-4fb4173715bd-a_window_layout_round_trips">
#[tokio::test(flavor = "multi_thread")]
async fn a_window_layout_round_trips() {
    let session = start().await;

    let (status, body) = get(&session, "/api/workspace/ui").await;
    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "state": null }),
        "a fresh project remembers nothing"
    );

    let layout = json!({
        "panes": [{ "tabs": ["demo.md"], "active": 0 }],
        "wrap": { "demo.md": 72 },
    });
    let (status, _) = put(&session, "/api/workspace/ui", json!({ "state": layout })).await;
    assert_eq!(status, 200);

    let (_, body) = get(&session, "/api/workspace/ui").await;
    assert_eq!(body, json!({ "state": layout }));
}

</storycode:copy>
<a id="f-4fb4173715bd-a_draft_comes_back_with_the_bytes_it_was_taken_from"></a>
### `a_draft_comes_back_with_the_bytes_it_was_taken_from`

<storycode:copy id="f-4fb4173715bd-a_draft_comes_back_with_the_bytes_it_was_taken_from">
#[tokio::test(flavor = "multi_thread")]
async fn a_draft_comes_back_with_the_bytes_it_was_taken_from() {
    // The base is the whole reason a restored draft can be MERGED rather than
    // fought over: it is the common ancestor of the buffer and whatever the
    // file says now.
    let session = start().await;

    let (status, _) = put(
        &session,
        "/api/workspace/drafts",
        json!({
            "path": "demo.md",
            "contents": "half a sentence",
            "base": "# Demo\n",
            "saved_at": 1_700_000_000_000u64,
        }),
    )
    .await;
    assert_eq!(status, 200);

    let (status, body) = get(&session, "/api/workspace/drafts").await;
    assert_eq!(status, 200);
    let drafts = body["drafts"].as_array().expect("a list of drafts");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0]["path"], "demo.md");
    assert_eq!(drafts[0]["contents"], "half a sentence");
    assert_eq!(drafts[0]["base"], "# Demo\n");
}

</storycode:copy>
<a id="f-4fb4173715bd-saving_the_file_throws_its_draft_away"></a>
### `saving_the_file_throws_its_draft_away`

<storycode:copy id="f-4fb4173715bd-saving_the_file_throws_its_draft_away">
#[tokio::test(flavor = "multi_thread")]
async fn saving_the_file_throws_its_draft_away() {
    let session = start().await;
    put(
        &session,
        "/api/workspace/drafts",
        json!({ "path": "demo.md", "contents": "x", "base": "", "saved_at": 1 }),
    )
    .await;

    let (status, _) = send(
        &session,
        reqwest::Method::DELETE,
        "/api/workspace/drafts?path=demo.md",
        None,
    )
    .await;
    assert_eq!(status, 200);

    let (_, body) = get(&session, "/api/workspace/drafts").await;
    assert_eq!(body["drafts"].as_array().unwrap().len(), 0);
}

</storycode:copy>
<a id="f-4fb4173715bd-discarding_a_draft_that_is_not_there_is_not_an_error"></a>
### `discarding_a_draft_that_is_not_there_is_not_an_error`

<storycode:copy id="f-4fb4173715bd-discarding_a_draft_that_is_not_there_is_not_an_error">
#[tokio::test(flavor = "multi_thread")]
async fn discarding_a_draft_that_is_not_there_is_not_an_error() {
    // The page discards on every save, and most saves have nothing to discard.
    let session = start().await;
    let (status, _) = send(
        &session,
        reqwest::Method::DELETE,
        "/api/workspace/drafts?path=never-existed.md",
        None,
    )
    .await;
    assert_eq!(status, 200);
}

</storycode:copy>
<a id="f-4fb4173715bd-nothing_is_written_inside_the_project_folder"></a>
### `nothing_is_written_inside_the_project_folder`

<storycode:copy id="f-4fb4173715bd-nothing_is_written_inside_the_project_folder">
#[tokio::test(flavor = "multi_thread")]
async fn nothing_is_written_inside_the_project_folder() {
    // The guarantee this whole store exists for. A draft is unfinished work
    // its author has not decided to keep; a `.gitignore` entry is a promise
    // this tool cannot keep on somebody else's machine.
    let session = start().await;
    put(
        &session,
        "/api/workspace/ui",
        json!({ "state": { "panes": [] } }),
    )
    .await;
    put(
        &session,
        "/api/workspace/drafts",
        json!({ "path": "demo.md", "contents": "unsaved", "base": "", "saved_at": 1 }),
    )
    .await;

    let names: Vec<String> = std::fs::read_dir(&session.project)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        names,
        vec!["demo.md".to_string()],
        "the project folder grew something: {names:?}"
    );
}

</storycode:copy>
<a id="f-4fb4173715bd-a_layout_carrying_a_document_is_refused_with_a_reason"></a>
### `a_layout_carrying_a_document_is_refused_with_a_reason`

<storycode:copy id="f-4fb4173715bd-a_layout_carrying_a_document_is_refused_with_a_reason">
#[tokio::test(flavor = "multi_thread")]
async fn a_layout_carrying_a_document_is_refused_with_a_reason() {
    // The failure mode this guards against is somebody stuffing buffer
    // contents into the layout blob, where it would be reloaded on every
    // start and never merged against anything.
    let session = start().await;
    let huge = "x".repeat(1024 * 1024 + 1);
    let (status, body) = put(
        &session,
        "/api/workspace/ui",
        json!({ "state": { "oops": huge } }),
    )
    .await;
    assert_eq!(status, 422);
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("draft store"),
        "the message must say where this belongs instead: {body}"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/serve_workspace.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_workspace.rs"><storycode:paste select="#f-4fb4173715bd-DOC"/><storycode:paste select="#f-4fb4173715bd-Session"/><storycode:paste select="#f-4fb4173715bd-state_dir"/><storycode:paste select="#f-4fb4173715bd-start"/><storycode:paste select="#f-4fb4173715bd-send"/><storycode:paste select="#f-4fb4173715bd-get"/><storycode:paste select="#f-4fb4173715bd-put"/><storycode:paste select="#f-4fb4173715bd-a_window_layout_round_trips"/><storycode:paste select="#f-4fb4173715bd-a_draft_comes_back_with_the_bytes_it_was_taken_from"/><storycode:paste select="#f-4fb4173715bd-saving_the_file_throws_its_draft_away"/><storycode:paste select="#f-4fb4173715bd-discarding_a_draft_that_is_not_there_is_not_an_error"/><storycode:paste select="#f-4fb4173715bd-nothing_is_written_inside_the_project_folder"/><storycode:paste select="#f-4fb4173715bd-a_layout_carrying_a_document_is_refused_with_a_reason"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/App.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-25cc6f821b46)
- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/components/PromptPanel.tsx](contract.md#output-0b8059fcf25a)
- [apps/web/src/lib/drafts.ts](contract.md#output-74298466496e)
- [apps/web/src/lib/outputSave.ts](../../../mechanisms/apps/web/src/lib/outputSave.ts.md#output-ff2a9a5b4231)
- [apps/web/src/lib/viewBuffers.ts](../../../mechanisms/apps/web/src/lib/viewBuffers.ts.md#output-7b9e9d92a497)
- [apps/web/src/lib/windowClose.ts](contract.md#output-e5e25a4f8e9b)
- [apps/web/src/shell/SpatialShellView.tsx](../../../mechanisms/apps/web/src/shell/SpatialShellView.tsx.md#output-64456f67b44e)
- [apps/web/src/shell/views.tsx](../../../mechanisms/apps/web/src/shell/views.tsx.md#output-68a8fba10658)
- [apps/web/src/views/RepresentationPane.tsx](../../../mechanisms/apps/web/src/views/RepresentationPane.tsx.md#output-cd66c1c2250e)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [apps/web/src/views/useUnsavedLifecycle.ts](../../../mechanisms/apps/web/src/views/useUnsavedLifecycle.ts.md#output-0a84f897aa4c)
- [apps/web/src/views/useUntitledRecovery.ts](contract.md#output-ec052e0ed871)
- [crates/hickory-cli/src/serve/shell.rs](../../../mechanisms/crates/hickory-cli/src/serve/shell.rs.md#output-16984801199b)

### Tests and fixtures

- [apps/web/src/App.test.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-65ac24c24624)
- [apps/web/src/components/PlainFilePane.test.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-d923b2fdae48)
- [apps/web/src/components/PromptPanel.test.tsx](../../../mechanisms/apps/web/src/components/PromptPanel.test.tsx.md#output-6e5d4425a8d2)
- [apps/web/src/lib/drafts.lifecycle.test.tsx](contract.md#output-06cfd4483cf0)
- [apps/web/src/lib/outputSave.test.ts](../../../mechanisms/apps/web/src/lib/outputSave.ts.md#output-5dd719c6fb43)
- [apps/web/src/lib/windowClose.test.ts](contract.md#output-972530db3158)
- [apps/web/src/shell/FolderTreePane.test.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-508e32e299fd)
- [apps/web/src/shell/ShellView.test.tsx](contract.md#output-c3dca2adcf89)
- [apps/web/src/shell/SpatialShellView.test.tsx](../../../mechanisms/apps/web/src/shell/SpatialShellView.tsx.md#output-57fb3ab47480)
- [apps/web/src/views/SettingsView.test.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-5bfac230ae5d)
- [apps/web/src/views/useUnsavedLifecycle.test.tsx](../../../mechanisms/apps/web/src/views/useUnsavedLifecycle.ts.md#output-18ff98d49ede)
- [apps/web/src/views/useUntitledRecovery.test.tsx](contract.md#output-1eaf7867608d)
- [crates/hickory-cli/tests/serve_ui_settings.rs](../../../mechanisms/crates/hickory-cli/tests/serve_ui_settings.rs.md#output-c816a698604d)
- [crates/hickory-cli/tests/serve_workspace.rs](contract.md#output-4fb4173715bd)

### Focused witnesses

- `waits for an unfinished recovery write and never rewrites explicitly discarded work` in `apps/web/src/lib/drafts.lifecycle.test.tsx`.
- `keeps an inactive view open when its attempted Save fails` in `apps/web/src/views/useUnsavedLifecycle.test.tsx`.

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
