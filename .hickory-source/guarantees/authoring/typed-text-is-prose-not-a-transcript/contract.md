<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Typed Text Is Prose, Not A Transcript

For: Authors editing executable documents and the files those documents produce.

Given text typed into the app's scratchpad, when it is saved, then it becomes a
note whose body is **prose** — no `<hick:transcript>`, no derived speaker turns,
no summary transforms — recording that it came from the scratchpad and, for
agent-assisted drafts, explicit agent-assisted authorship; and
when the text is empty, or contains hick markup, then it is refused with a
reason and the text is left where the person can act on it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/typed-text-is-prose-not-a-transcript.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-461823ba71cc"></a>
## `apps/web/src/components/ScratchpadPane.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Typed Text Is Prose, Not A Transcript](contract.md).

<a id="f-461823ba71cc-saveScratchpad"></a>
### `saveScratchpad`

<storycode:copy id="f-461823ba71cc-saveScratchpad">
// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ScratchpadPane } from "./ScratchpadPane";

const saveScratchpad = vi.fn();
</storycode:copy>
<a id="f-461823ba71cc-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-461823ba71cc-expression_statement">
vi.mock("../api/client", () => ({
  api: {
    saveScratchpad: (text: string, assisted?: boolean) => assisted === undefined ? saveScratchpad(text) : saveScratchpad(text, assisted),
  },
}));

</storycode:copy>
<a id="f-461823ba71cc-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-461823ba71cc-expression_statement-2">
afterEach(() => {
  cleanup();
  saveScratchpad.mockReset();
});

</storycode:copy>
<a id="f-461823ba71cc-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-461823ba71cc-expression_statement-3">
describe("the scratchpad", () => {
  it("will not save nothing", async () => {
    render(<ScratchpadPane />);
    const button = screen.getByRole("button", {
      name: /save as note/i,
    }) as HTMLButtonElement;
    expect(button.disabled).toBe(true);
  });

  it("saves what was typed and says where it landed", async () => {
    saveScratchpad.mockResolvedValue({ path: "2026-08-18-standup.md" });
    render(<ScratchpadPane />);

    fireEvent.change(screen.getByLabelText(/scratchpad/i), {
      target: { value: "All green." },
    });
    fireEvent.click(screen.getByRole("button", { name: /save as note/i }));

    await waitFor(() =>
      expect(saveScratchpad).toHaveBeenCalledWith("All green."),
    );
    expect(await screen.findByText(/2026-08-18-standup\.md/)).toBeTruthy();
  });

  it("clears itself after a save, so the next thought starts empty", async () => {
    saveScratchpad.mockResolvedValue({ path: "note.md" });
    render(<ScratchpadPane />);

    const box = screen.getByLabelText(/scratchpad/i) as HTMLTextAreaElement;
    fireEvent.change(box, { target: { value: "Something" } });
    fireEvent.click(screen.getByRole("button", { name: /save as note/i }));

    await waitFor(() => expect(box.value).toBe(""));
  });

  it("shows the refusal instead of losing the text", async () => {
    // The refusals this can hit all carry a next step, and the words the
    // person typed are still in the box to act on.
    saveScratchpad.mockRejectedValue(
      new Error("this text contains `<hick:` — Next step: reword it"),
    );
    render(<ScratchpadPane />);

    const box = screen.getByLabelText(/scratchpad/i) as HTMLTextAreaElement;
    fireEvent.change(box, { target: { value: "I used a hick tag" } });
    fireEvent.click(screen.getByRole("button", { name: /save as note/i }));

    expect(await screen.findByText(/Next step/)).toBeTruthy();
    expect(box.value).toBe("I used a hick tag");
  });
});

</storycode:copy>
<a id="f-461823ba71cc-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-461823ba71cc-expression_statement-4">
it("retains agent-assisted authorship when a human saves an agent-edited draft", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  saveScratchpad.mockResolvedValue({ path: "assisted.md" });
  render(<UiActionContext.Provider value={registry}><ScratchpadPane /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, {
    handle: actions().find(action => action.label === "Set scratchpad draft")!.handle, arguments: { text: "Agent draft" }, request_id: "draft",
  }); });
  fireEvent.change(screen.getByLabelText(/scratchpad/i), { target: { value: "Agent draft with human edits" } });
  fireEvent.click(screen.getByRole("button", { name: /save as note/i }));
  await waitFor(() => expect(saveScratchpad).toHaveBeenCalledExactlyOnceWith("Agent draft with human edits", true));
  expect(await screen.findByText("Saved to assisted.md")).toBeTruthy();
});

</storycode:copy>
<a id="f-461823ba71cc-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-461823ba71cc-expression_statement-5">
it("excludes duplicate scratchpad saves and retains a failed agent draft", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let fail!: (error: unknown) => void;
  saveScratchpad.mockImplementation(() => new Promise((_done, reject) => { fail = reject; }));
  render(<UiActionContext.Provider value={registry}><ScratchpadPane /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  fireEvent.change(screen.getByLabelText(/scratchpad/i), { target: { value: "Retain me" } });
  const actor = { id: "acp:turn", kind: "agent" as const };
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, actor, {
    handle: actions().find(action => action.label === "Save scratchpad as note")!.handle, arguments: {}, request_id: "save",
  }); });
  expect(await registry.act(registry.scope, { id: "python:other", kind: "agent" }, {
    handle: actions().find(action => action.label === "Save scratchpad as note")!.handle, arguments: {}, request_id: "other",
  })).toMatchObject({ status: "disabled" });
  await act(async () => { fail(Error("Engine update required")); await pending; });
  expect((screen.getByLabelText(/scratchpad/i) as HTMLTextAreaElement).value).toBe("Retain me");
  expect(saveScratchpad).toHaveBeenCalledExactlyOnceWith("Retain me", true);
});
</storycode:copy>

### Assemble `apps/web/src/components/ScratchpadPane.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ScratchpadPane.test.tsx"><storycode:paste select="#f-461823ba71cc-saveScratchpad"/><storycode:paste select="#f-461823ba71cc-expression_statement"/><storycode:paste select="#f-461823ba71cc-expression_statement-2"/><storycode:paste select="#f-461823ba71cc-expression_statement-3"/><storycode:paste select="#f-461823ba71cc-expression_statement-4"/><storycode:paste select="#f-461823ba71cc-expression_statement-5"/></storycode:file>

<a id="output-abb5a7592a08"></a>
## `apps/web/src/components/ScratchpadPane.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Typed Text Is Prose, Not A Transcript](contract.md).

<a id="f-abb5a7592a08-Result"></a>
### `Result`

<storycode:copy id="f-abb5a7592a08-Result">
import { useEffect, useId, useRef, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiActor, UiOutcome } from "../lib/uiActions";
import { api } from "../api/client";

/** Where the note landed, or why it did not. */
type Result =
  | { kind: "saved"; path: string }
  | { kind: "refused"; message: string }
  | null;

</storycode:copy>
<a id="f-abb5a7592a08-ScratchpadPane"></a>
### `ScratchpadPane`

The third way material gets into a notes folder, beside downloading a file
into the inbox and copying one there: type it here.
This is not a file-ingest surface and deliberately does not look like one.
Human-only text is prose a named human wrote, in this app, right now — the
strongest material there is on the attributable axis of
`docs/specs/freeform/provenance-and-standing.md`. It is never wrapped as a
transcript, and nothing about it is summarized, because there is no
machine step between the person and the words. Agent-assisted drafts instead
retain an explicit authorship marker and are never presented as human-only.
Saving is deliberately explicit rather than automatic. A scratchpad that
committed every keystroke to a note would turn a place to think into a place
that keeps a record, and people stop thinking in those.

<storycode:copy id="f-abb5a7592a08-ScratchpadPane">
/**
 * The third way material gets into a notes folder, beside downloading a file
 * into the inbox and copying one there: type it here.
 *
 * This is not a file-ingest surface and deliberately does not look like one.
 * Human-only text is prose a named human wrote, in this app, right now — the
 * strongest material there is on the attributable axis of
 * `docs/specs/freeform/provenance-and-standing.md`. It is never wrapped as a
 * transcript, and nothing about it is summarized, because there is no
 * machine step between the person and the words. Agent-assisted drafts instead
 * retain an explicit authorship marker and are never presented as human-only.
 *
 * Saving is deliberately explicit rather than automatic. A scratchpad that
 * committed every keystroke to a note would turn a place to think into a place
 * that keeps a record, and people stop thinking in those.
 */
export function ScratchpadPane() {
  const [text, setText] = useState("");
  const [agentAssisted, setAgentAssisted] = useState(false);
  const pending = useRef(false);
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const [saving, setSaving] = useState(false);
  const [result, setResult] = useState<Result>(null);

  const empty = text.trim().length === 0;

  const change = (value: string, agent: boolean) => {
    if (pending.current) return;
    if (!value) setAgentAssisted(false);
    else if (agent && value !== text) setAgentAssisted(true);
    setText(value);
  };
  async function save(actor: UiActor, signal: AbortSignal): Promise<UiOutcome> {
    if (empty || pending.current || signal.aborted || !mounted.current) return { status: "unavailable", reason: "Scratchpad is empty, closed or busy" };
    pending.current = true; setSaving(true); setResult(null);
    const assisted = agentAssisted || actor.kind === "agent";
    try {
      const saved = assisted ? await api.saveScratchpad(text, true) : await api.saveScratchpad(text);
      if (mounted.current) { setResult({ kind: "saved", path: saved.path }); setText(""); setAgentAssisted(false); }
      return { status: "completed", value: { path: saved.path, agent_assisted: assisted, saved: true, verified: false } };
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      if (mounted.current) setResult({ kind: "refused", message });
      return { status: "unavailable", reason: message };
    } finally { pending.current = false; if (mounted.current) setSaving(false); }
  }
  const parent = `scratchpad:${useId()}`;
  const revision = useUiRevision(() => [text, agentAssisted, pending.current, result]);
  const invoke = useUiCommands([
    { key: `${parent}:text`, command: { parent, subject: parent, label: "Set scratchpad draft", parameters: { text: { type: "string", maxLength: 1048576 } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !pending.current }), execute: async (args, actor) => {
        if (pending.current) return { status: "unavailable", reason: "Wait for the pending save" };
        change(args.text as string, actor.kind === "agent"); return { status: "completed", value: { draft_updated: true, saved: false } };
      } } },
    { key: `${parent}:save`, command: { parent, subject: parent, label: "Save scratchpad as note", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !empty && !pending.current }), execute: async (_args, actor, signal) => save(actor, signal) } },
  ]);
  const humanSave = () => void invoke(`${parent}:save`, {}).catch(() => {});
  useUiReading("scratchpad draft and save status", { text, agent_assisted: agentAssisted, saving, result,
    scope: "Explicit save; agent-assisted drafts retain an authorship marker, without declaring verification" });

  return (
    <div className="scratchpad-pane">
      <label className="scratchpad-label" htmlFor="scratchpad-text">
        Scratchpad
      </label>
      <p className="scratchpad-hint">
        Type a note. Saving writes it into this folder as a document — the first
        line becomes its title.
      </p>
      <textarea
        id="scratchpad-text"
        className="scratchpad-text"
        value={text}
        placeholder={"# Standup\n\nWhat happened, and what it means."}
        disabled={saving}
        onChange={(e) => change(e.target.value, false)}
        onKeyDown={(e) => {
          // Save without leaving the keyboard, the way every other editor does.
          if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
            e.preventDefault();
            humanSave();
          }
        }}
      />
      {agentAssisted && <p className="scratchpad-hint">Includes agent edits; this note will retain agent-assisted authorship.</p>}
      <div className="scratchpad-actions">
        <button
          type="button"
          className="scratchpad-save"
          disabled={empty || saving}
          onClick={humanSave}
        >
          {saving ? "Saving…" : "Save as note"}
        </button>
        {result?.kind === "saved" && (
          <span className="scratchpad-saved">Saved to {result.path}</span>
        )}
      </div>
      {result?.kind === "refused" && (
        // The refusals this can hit are all about hick markup in prose, and
        // they carry a next step — so show the message rather than a generic
        // failure that leaves the text stranded.
        <p className="scratchpad-refused">{result.message}</p>
      )}
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/ScratchpadPane.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ScratchpadPane.tsx"><storycode:paste select="#f-abb5a7592a08-Result"/><storycode:paste select="#f-abb5a7592a08-ScratchpadPane"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/components/ScratchpadPane.tsx](contract.md#output-abb5a7592a08)
- [apps/web/src/views/workspaceState.ts](../../../mechanisms/apps/web/src/views/workspaceState.ts.md#output-efd800f9ed5a)
- [crates/hickory-cli/src/ingest.rs](../../../mechanisms/crates/hickory-cli/src/ingest.rs.md#output-9e9d98a631d5)
- [crates/hickory-cli/src/serve/plain_file.rs](../../../mechanisms/crates/hickory-cli/src/serve/plain_file.rs.md#output-128423f88fa6)

### Tests and fixtures

- [apps/web/src/components/ScratchpadPane.test.tsx](contract.md#output-461823ba71cc)
- [apps/web/src/router.test.ts](../../../mechanisms/apps/web/src/router.test.ts.md#output-415811f3ab59)
- [apps/web/src/views/workspaceState.test.ts](../../../mechanisms/apps/web/src/views/workspaceState.ts.md#output-14d3dbc939ff)
- [crates/hickory-cli/tests/ingest.rs](../../../mechanisms/crates/hickory-cli/tests/ingest.rs.md#output-a492c9d4e3cc)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
