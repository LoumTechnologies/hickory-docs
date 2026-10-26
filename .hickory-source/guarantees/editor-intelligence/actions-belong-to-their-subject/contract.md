<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep editor actions attached to the view they affect

For: Authors using completion, diagnostics and refactoring inside literate source.

Move or focus a spatial pane. Its action controls must still target that pane without moving its editor buffer.

## What must happen

Actions retain their owning document/file/view identity and current availability. Current metadata and document/file controls use each spatial pane's existing header. View changes preserve the mounted editor and source identity. A source revision or subject change invalidates stale action context.

## Limits of the evidence

Subject identity and visual placement do not establish that every native action has a registered agent equivalent.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/actions-belong-to-their-subject.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Apply the minimalist-editor amendment while retaining the explicitly preserved visible ruler and action-rail collision behavior.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-63699185668b"></a>
## `apps/web/src/components/MoreActions.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../agent/ui-action-dispatch/contract.md), [Keep editor actions attached to the view they affect](contract.md).

<a id="f-63699185668b-MoreActions"></a>
### `MoreActions`

<storycode:copy id="f-63699185668b-MoreActions">
import { useId, useRef, useState, type ReactNode } from "react";
import { UiActionsVisibility, useUiCommands, useUiPendingHumanActions } from "../lib/uiActionBridge";

/** Occasional tab actions stay discoverable without occupying the toolbar. */
export function MoreActions({ children, label = "More actions" }: { children: ReactNode; label?: string }) {
  const id = useId();
  const details = useRef<HTMLDetailsElement>(null);
  const [open, setOpen] = useState(false);
  const [error, setError] = useState("");
  const pending = useUiPendingHumanActions();
  const beforeClick = useRef(new Set<string>());
  const epoch = useRef(0);
  const key = `menu:${id}/toggle`;
  const invoke = useUiCommands([{ key, command: {
    label, subject: `menu:${id}`, parent: "menus", parameters: {}, prerequisites: [],
    read: () => ({ revision: String(details.current?.open ?? false), enabled: !!details.current, value: details.current?.open ?? false }),
    execute: async () => {
      if (!details.current) return { status: "unavailable", reason: "Menu was closed" };
      const next = !details.current.open;
      epoch.current++;
      details.current.open = next; setOpen(next); setError("");
      return { status: "completed", value: { open: next, coverage: "Disclosed menu children use their own registered descriptors" } };
    },
  } }]);
  const close = () => { if (details.current) details.current.open = false; setOpen(false); };
  return <details ref={details} open={open} className="more-actions" onClickCapture={() => { beforeClick.current = new Set(pending().map(request => request.id)); }} onKeyDown={event => {
    if (event.key === "Escape") { close(); details.current?.querySelector("summary")?.focus(); event.stopPropagation(); }
  }}><summary aria-label={label} onClick={event => {
    event.preventDefault(); void invoke(key).catch(reason => setError(String(reason)));
  }}>⋯</summary>
    {error && <span role="alert">{error}</span>}
    <div className="more-actions__items" onClick={event => {
      if (!(event.target as HTMLElement).closest("button")) return;
      const generation = epoch.current;
      const requests = pending().filter(request => !beforeClick.current.has(request.id));
      if (!requests.length) close();
      else void Promise.allSettled(requests.map(request => request.result)).then(() => { if (epoch.current === generation) close(); });
    }}><UiActionsVisibility visible={open}>{children}</UiActionsVisibility></div>
  </details>;
}
</storycode:copy>

### Assemble `apps/web/src/components/MoreActions.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MoreActions.tsx"><storycode:paste select="#f-63699185668b-MoreActions"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/MoreActions.tsx](contract.md#output-63699185668b)
- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/lib/railActions.ts](../../../mechanisms/apps/web/src/lib/railActions.ts.md#output-499155452cf6)
- [apps/web/src/views/RepresentationPane.tsx](../../../mechanisms/apps/web/src/views/RepresentationPane.tsx.md#output-cd66c1c2250e)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/lib/railActions.test.ts](../../../mechanisms/apps/web/src/lib/railActions.ts.md#output-49410ea38e0d)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
