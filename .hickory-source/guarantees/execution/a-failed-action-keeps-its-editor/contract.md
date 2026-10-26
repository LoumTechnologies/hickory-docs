<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A failed action keeps its editor

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a loaded document or generated file, when a Run, Test, render, or
save request fails, then its editable work remains reachable and a local notice
names the failure and offers the corresponding retry. Only a failed initial
load can replace the document editor. A failed template listing offers retry
without leaving New Project; unsupported SDK installation remains a link.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-failed-action-keeps-its-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-26d328f740be"></a>
## `apps/web/src/components/ActionNotice.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A failed action keeps its editor](contract.md).

<a id="f-26d328f740be-ActionNotice"></a>
### `ActionNotice`

<storycode:copy id="f-26d328f740be-ActionNotice">
import { useId, useRef, useState, type ReactNode } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { ownedText } from "../lib/ownedText";
import type { UiCommand } from "../lib/uiActions";

/** Action feedback belongs beside the work that owns it, including recovery. */
export function ActionNotice({ kind, children, retry, onDismiss }: {
  kind: "pending" | "pass" | "fail";
  children: ReactNode;
  retry?: { label: string; run: () => void };
  onDismiss?: () => void;
}) {
  const parent = `action-notice:${useId()}`;
  const identity = useRef({ retry: retry?.run, dismiss: onDismiss, version: crypto.randomUUID() });
  if (identity.current.retry !== retry?.run || identity.current.dismiss !== onDismiss)
    identity.current = { retry: retry?.run, dismiss: onDismiss, version: crypto.randomUUID() };
  const [error, setError] = useState<string | null>(null);
  const message = ownedText(children);
  const revision = useUiRevision(() => [kind, message, retry?.label, identity.current.version, error]);
  const commands: { key: string; command: UiCommand }[] = [];
  for (const action of [
    ...(retry ? [{ id: "retry", label: retry.label, run: retry.run }] : []),
    ...(onDismiss ? [{ id: "dismiss", label: "Dismiss action notice", run: onDismiss }] : []),
  ]) commands.push({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }), execute: async () => {
      await action.run(); return { status: "completed", value: { callback_requested: true,
        scope: "Inspect the owning operation status; requesting recovery does not establish save or verification" } };
    },
  } });
  const invoke = useUiCommands(commands);
  const human = (id: string) => void invoke(`${parent}:${id}`, {}).catch(cause => setError(String(cause)));
  useUiReading("action notice", { kind, message, error, retry_label: retry?.label ?? null });
  return <div className={`banner banner-${kind} action-notice`} role={kind === "fail" ? "alert" : "status"}>
    <span>{children}</span>
    {retry && <button className="btn btn-small" onClick={() => human("retry")}>{retry.label}</button>}
    {onDismiss && <button className="btn btn-small" onClick={() => human("dismiss")}>Dismiss</button>}
    {error && <span role="alert">{error}</span>}
  </div>;
}
</storycode:copy>

### Assemble `apps/web/src/components/ActionNotice.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ActionNotice.tsx"><storycode:paste select="#f-26d328f740be-ActionNotice"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/ActionNotice.tsx](contract.md#output-26d328f740be)
- [apps/web/src/components/NewProjectDialog.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-65e48889b856)
- [apps/web/src/shell/views.tsx](../../../mechanisms/apps/web/src/shell/views.tsx.md#output-68a8fba10658)
- [apps/web/src/views/documentSession.tsx](../../../mechanisms/apps/web/src/views/documentSession.tsx.md#output-bfb59577f48b)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/components/NewProjectDialog.test.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-065ce188ea68)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
