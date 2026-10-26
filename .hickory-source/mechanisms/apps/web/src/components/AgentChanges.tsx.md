<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AgentChanges.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d82ea3dcdcf9"></a>
## `apps/web/src/components/AgentChanges.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Separate model snippets from trusted host operations](../../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-d82ea3dcdcf9-AgentChanges"></a>
### `AgentChanges`

<storycode:copy id="f-d82ea3dcdcf9-AgentChanges">
import "./AgentChanges.css";
import { useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { acpApi, type AcpState, type AgentChange } from "../api/acp";
import { publishReading, forgetReading, openReading } from "../lib/readingViews";
import { DocumentReading } from "../views/DocumentReading";
import { AgentEditConflict } from "../lib/agentEdit";

export function AgentChanges({ doc, state, running, apply, update, submitDecision }: {
  doc: string; state: AcpState | null; running: boolean;
  submitDecision?: (body: Parameters<typeof acpApi.edits>[1]) => Promise<AcpState>;
  apply?: (change: AgentChange) => void | Promise<void>;
  update: (state: AcpState) => void;
}) {
  const [error, setError] = useState<string | null>(null);
  const inflight = useRef(new Set<string>());
  const decide = async (change: AgentChange, accepted: boolean, replacement?: string): Promise<UiOutcome> => {
    const revised = replacement === undefined ? change : { ...change, newText: replacement };
    if (inflight.current.has(change.id)) return { status: "unavailable", reason: "Decision already in flight" };
    inflight.current.add(change.id); setError(null);
    let failure: string | undefined;
    let currentText: string | undefined;
    try {
      if (accepted && change.editor) {
        if (!apply) throw new Error("Open this document's workspace editor before accepting its change.");
        await apply(revised);
      }
    } catch (e) { failure = e instanceof Error ? e.message : String(e); currentText = e instanceof AgentEditConflict ? e.currentText : undefined; setError(failure); }
    try {
      const body = { id: change.id, accepted: accepted && !failure, error: failure, ...(replacement !== undefined ? { new_text: replacement } : {}), ...(currentText !== undefined ? { current_text: currentText } : {}) };
      const actual = await (submitDecision ? submitDecision(body) : acpApi.edits(doc, body, state?.backend));
      update(actual);
      return failure ? { status: "unavailable", reason: failure } : { status: "completed", value: { id: change.id, status: actual.edits?.changes.find(c => c.id === change.id)?.status ?? "not present in returned host state", verification: "not established" } };
    }
    catch (e) { const reason = e instanceof Error ? e.message : String(e); setError(reason); return { status: "unavailable", reason: `${reason}; inspect the editor and host state before retrying: a local edit may already have occurred` }; }
    finally { inflight.current.delete(change.id); }
  };
  const latest = useRef(decide); latest.current = decide;
  const opened = useRef(new Set<string>());
  useEffect(() => {
    state?.edits?.changes.forEach(change => {
      const id = `proposal:${doc}:${change.id}`;
      publishReading(id, { title: "Proposed change", path: change.path ?? change.name,
        before: change.oldText, after: change.newText, status: change.status,
        editable: change.status === "pending", decide: (accepted, text) => latest.current(change, accepted, text) });
      if (change.status === "pending" && !opened.current.has(id)) {
        opened.current.add(id);
      }
    });
  }, [doc, state?.edits?.changes]);
  useEffect(() => () => { opened.current.forEach(forgetReading); opened.current.clear(); }, [doc]);
  useEffect(() => {
    if (state?.backend === "python") return;
    state?.edits?.changes.filter(c => c.status === "applying").forEach(c => { void decide(c, true); });
  }, [state?.edits?.changes]);
  const parent = `agent-changes:${useId()}`;
  const revision = useUiRevision(() => [doc, state?.backend, state?.edits, running, error]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled }), execute } });
  if (state?.edits) {
    add("read", "Read conversation edit review bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, true, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ doc, backend: state.backend, edits: state.edits, error, scope: "Proposals and actual recorded statuses; acceptance does not verify a change" }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside edit review" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    });
    add("open", "Open conversation change review", { id: { type: "string", maxLength: 4096 } }, true, async args => {
      const change = state.edits?.changes.find(c => c.id === args.id);
      if (!change) return { status: "unavailable", reason: "Observe a current proposal identity" };
      openReading(`proposal:${doc}:${change.id}`, `${change.path ?? change.name} · review`);
      return { status: "completed", value: { review_open_requested: change.id } };
    });
    add("decision", "Decide conversation change", { id: { type: "string", maxLength: 4096 }, accepted: { type: "boolean" } }, true, async (args, actor) => {
      const change = state.edits?.changes.find(c => c.id === args.id);
      if (!change || change.status !== "pending") return { status: "unavailable", reason: "Proposal is no longer pending; inspect current host status" };
      if (actor.kind === "agent") return { status: "requires_input", request: `edit-review:${doc}:${state.backend}:${change.id}`, reason: "The human reviews and decides this proposed edit in the actual change review; inspect its returned status afterward" };
      return decide(change, args.accepted as boolean);
    });
    add("mode", "Set conversation edit review mode", { mode: { type: "string", maxLength: 11, choices: ["review", "auto-accept"] } }, !running && state.backend !== "python", async (args, actor) => {
      if (actor.kind === "agent" && args.mode === "auto-accept") return { status: "requires_input", request: `edit-mode:${doc}:${state.backend}`, reason: "A human must authorize automatic acceptance of future edits" };
      try {
        const actual = await acpApi.edits(doc, { mode: args.mode as "review" | "auto-accept" }, state.backend); update(actual);
        return { status: "completed", value: { mode: actual.edits?.mode ?? null } };
      } catch (reason) { setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
    });
  }
  const invoke = useUiCommands(commands);
  if (!state?.edits) return null;
  return <section className="agent-changes" aria-label="Conversation document edits">
    <label>Document edits <select aria-label="Document edits" value={state.edits.mode} disabled={running || state.backend === "python"}
      onChange={e => void invoke(`${parent}:mode`, { mode: e.target.value })}><option value="review">Review</option>{state.backend !== "python" && <option value="auto-accept">Auto-accept</option>}</select></label>
    <p className="muted">Reviews cover Hickory document tools. {state.backend === "python" ? "Python document-edit previews require approval. Shared UI operations retain their separate host permissions. Acceptance does not verify the edit or change its authorship. Checks remain unrun." : "Native agent commands follow the adapter’s permissions and can bypass this review."}</p>
    {error && <p role="alert">{error}</p>}
    {state.edits.changes.map(change => <div key={change.id} className="agent-change-link">
      <span>{change.status === "pending" ? "Proposed change" : change.status === "applying" ? "Applying change" : `${change.status.charAt(0).toUpperCase()}${change.status.slice(1)} change`} · {change.path ?? change.name}</span>
      <DocumentReading id={`proposal:${doc}:${change.id}`} />
      <button className="btn" onClick={() => void invoke(`${parent}:open`, { id: change.id })}>Review in document</button>
    </div>)}
  </section>;
}
</storycode:copy>

### Assemble `apps/web/src/components/AgentChanges.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AgentChanges.tsx"><storycode:paste select="#f-d82ea3dcdcf9-AgentChanges"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-7ab5cdb69360"></a>
## `apps/web/src/components/AgentChanges.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Conversation edits use the client's review policy](../../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md), [Separate model snippets from trusted host operations](../../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md), [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-7ab5cdb69360-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement">
import { installMockHandler } from "../api/client";
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AgentChanges } from "./AgentChanges";
import { acpApi, type AcpState, type AgentChange } from "../api/acp";
vi.mock("../api/acp", () => ({ acpApi: { edits: vi.fn() } }));
</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-2">
installMockHandler(async () => ({}));
</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-3">
afterEach(() => { cleanup(); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-7ab5cdb69360-change"></a>
### `change`

<storycode:copy id="f-7ab5cdb69360-change">
const change: AgentChange = { id: "change", name: "Untitled", path: null, oldText: "# Old 📝\nkeep\n", newText: "# New 📝\nkeep\n", editor: true, status: "pending" };
</storycode:copy>
<a id="f-7ab5cdb69360-state"></a>
### `state`

<storycode:copy id="f-7ab5cdb69360-state">
const state: AcpState = { backend: "codex", ready: true, edits: { mode: "review", changes: [change] } };
</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-4">
it("uses the document comparison editor, waits for Accept, and applies before acknowledging", async () => {
  const apply = vi.fn(); const update = vi.fn();
  vi.mocked(acpApi.edits).mockImplementation(async () => { expect(apply).toHaveBeenCalledWith(change); return state; });
  const view = render(<AgentChanges doc="workspace" state={state} running apply={apply} update={update} />);
  await waitFor(() => expect(view.container.querySelector(".comparison-deleted")?.textContent).toContain("# Old 📝"));
  expect(view.container.querySelector(".comparison-added")?.textContent).toContain("New 📝");
  expect(view.container.querySelector(".cm-content")?.textContent).toContain("keep");
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: true, error: undefined }, "codex"));
});
</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-5">
it("rejects without changing the editor", async () => {
  const apply = vi.fn(); vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={state} running apply={apply} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Reject change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: false, error: undefined }, "codex"));
  expect(apply).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-6">
it("auto-accepts the same change without a user gesture", async () => {
  const apply = vi.fn(); vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={{ ...state, edits: { mode: "auto-accept", changes: [{ ...change, status: "applying" }] } }} running apply={apply} update={() => {}} />);
  await waitFor(() => expect(apply).toHaveBeenCalledTimes(1));
  expect(screen.queryByRole("button", { name: "Accept change" })).toBeNull();
});
</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-7">
it("reports a stale buffer to the agent as a failed edit", async () => {
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={state} running apply={() => { throw new Error("The document changed."); }} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: false, error: "The document changed." }, "codex"));
  expect(screen.getAllByRole("alert")[0].textContent).toBe("The document changed.");
});
</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-8">
it("changes the per-conversation policy and disables it during a turn", async () => {
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  const view = render(<AgentChanges doc="workspace" state={state} running={false} update={() => {}} />);
  fireEvent.change(screen.getByRole("combobox", { name: "Document edits" }), { target: { value: "auto-accept" } });
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { mode: "auto-accept" }, "codex"));
  view.rerender(<AgentChanges doc="workspace" state={state} running update={() => {}} />);
  expect((screen.getByRole("combobox", { name: "Document edits" }) as HTMLSelectElement).disabled).toBe(true);
});

</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-9">
it("returns newer editor text with a stale rejection so the agent can read again", async () => {
  const { AgentEditConflict } = await import("../lib/agentEdit");
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={state} running apply={() => { throw new AgentEditConflict("Changed during review", "My new sentence"); }} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: false, error: "Changed during review", current_text: "My new sentence" }, "codex"));
});

</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-10">
// Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
it("requires a human gesture for Python proposals and labels checks unrun", async () => {
  const pythonState: AcpState = { backend: "python", ready: true, edits: { mode: "review", changes: [{ ...change, editor: false }] } };
  const apply = vi.fn(); vi.mocked(acpApi.edits).mockResolvedValue(pythonState);
  render(<AgentChanges doc="workspace" state={pythonState} running={false} apply={apply} update={() => {}} />);
  expect((screen.getByRole("combobox", { name: "Document edits" }) as HTMLSelectElement).disabled).toBe(true);
  expect(screen.queryByRole("option", { name: "Auto-accept" })).toBeNull();
  expect(screen.getByText(/Checks remain unrun/)).toBeTruthy();
  expect(acpApi.edits).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: true, error: undefined }, "python"));
  expect(apply).not.toHaveBeenCalled(); // Rust applies approved file edits.
});

</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-11">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
it("accepts revised New bytes directly from the Agent pane", async () => {
  const { EditorView } = await import("@codemirror/view");
  const { act } = await import("@testing-library/react");
  const apply = vi.fn();
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  const view = render(<AgentChanges doc="workspace" state={state} running apply={apply} update={() => {}} />);
  await waitFor(() => expect(view.container.querySelector('[aria-label="New · proposed change"]')).toBeTruthy());
  const host = view.container.querySelector<HTMLElement>('[aria-label="New · proposed change"]')!;
  const editor = EditorView.findFromDOM(host)!;
  act(() => editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "My revised proposal\n" } }));
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(apply).toHaveBeenCalledWith({ ...change, newText: "My revised proposal\n" }));
  expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: true, error: undefined, new_text: "My revised proposal\n" }, "codex");
});

</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-12">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses exact proposals but requires the human decision and cannot self-enable auto-accept", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const { act } = await import("@testing-library/react");
  const { useState } = await import("react");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const apply = vi.fn();
  vi.mocked(acpApi.edits).mockResolvedValue({ ...state, edits: { mode: "review", changes: [{ ...change, status: "accepted" }] } });
  function Harness() { const [current, setCurrent] = useState(state); return <AgentChanges doc="workspace" state={current} running={false} apply={apply} update={setCurrent} />; }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(g => g.startsWith("agent-changes:"))!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain("# New");
  const reader = registry.observe(parent).actions.find(a => a.label === "Read conversation edit review bytes")!;
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: reader.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected reading");
  expect(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes))).toContain("# New 📝");
  const decision = registry.observe(parent).actions.find(a => a.label === "Decide conversation change")!;
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: decision.handle, arguments: { id: "change", accepted: true }, request_id: "ask-human" })).toMatchObject({ outcome: { status: "requires_input", request: "edit-review:workspace:codex:change" } }); });
  const mode = registry.observe(parent).actions.find(a => a.label === "Set conversation edit review mode")!;
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: mode.handle, arguments: { mode: "auto-accept" }, request_id: "no-self-approve" })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(apply).not.toHaveBeenCalled(); expect(acpApi.edits).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(apply).toHaveBeenCalledWith(change));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Accept change" })).toBeNull());
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: decision.handle, arguments: { id: "change", accepted: true }, request_id: "old-proposal" })).toMatchObject({ status: "stale" });
});

</storycode:copy>
<a id="f-7ab5cdb69360-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-7ab5cdb69360-expression_statement-13">
it("uses the host-selected private review transport without calling ordinary review APIs", async () => {
  const submitDecision = vi.fn().mockResolvedValue({ ...state, backend: "python" });
  render(<AgentChanges doc="private:bound" state={{ ...state, backend: "python", edits: { mode: "review", changes: [{ ...change, editor: false }] } }} running submitDecision={submitDecision} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Reject change" }));
  await waitFor(() => expect(submitDecision).toHaveBeenCalledWith({ id: "change", accepted: false, error: undefined }));
  expect(acpApi.edits).not.toHaveBeenCalled();
});
</storycode:copy>

### Assemble `apps/web/src/components/AgentChanges.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AgentChanges.test.tsx"><storycode:paste select="#f-7ab5cdb69360-expression_statement"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-2"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-3"/><storycode:paste select="#f-7ab5cdb69360-change"/><storycode:paste select="#f-7ab5cdb69360-state"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-4"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-5"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-6"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-7"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-8"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-9"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-10"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-11"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-12"/><storycode:paste select="#f-7ab5cdb69360-expression_statement-13"/></storycode:file>


</storycode:doc>
