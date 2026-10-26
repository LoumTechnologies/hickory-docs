<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A write exposes its proposed effects

For: Authors editing executable documents and the files those documents produce.

Given Insert, New Project, an environment command, a filesystem Dry Run,
an agent proposal, or unsaved literate-view edits, when the person inspects the
operation, then proposed bytes or effects are visible before committing the
write. A preview writes nothing. Literate previews compare the arrangement and
changed backing files; closed previews do not mount extra editors. The write
keeps its existing concurrency checks and reports the result locally.
Ordinary typing still autosaves without a confirmation for every edit.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-write-exposes-its-proposed-effects.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-9f0dbf2758da"></a>
## `apps/web/src/components/ChangePreview.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A write exposes its proposed effects](contract.md).

<a id="f-9f0dbf2758da-ChangePreview"></a>
### `ChangePreview`

<storycode:copy id="f-9f0dbf2758da-ChangePreview">
import { useState, type ReactNode } from "react";
import { EvidenceDisclosure } from "./EvidenceDisclosure";
import { useUiReading } from "../lib/uiReading";
import { ownedText } from "../lib/ownedText";

/** Inspect the proposed bytes and effects before choosing the write. */
export function ChangePreview({ heading, effects, children, collapsible = false }: {
  heading: string; effects?: ReactNode; children: ReactNode; collapsible?: boolean;
}) {
  const [expanded, setExpanded] = useState(false);
  const content = <>{children}{effects && <p className="insert-menu__hint">{effects}</p>}</>;
  const value = { heading, preview: ownedText(children), effects: ownedText(effects) };
  return collapsible
    ? <EvidenceDisclosure className="change-preview" label={heading} value={value} onOpenChange={setExpanded}>{expanded && content}</EvidenceDisclosure>
    : <section className="insert-menu__preview change-preview" aria-label={heading}>
      <PreviewReading value={value} />
      <h3 className="insert-menu__preview-title">{heading}</h3>{content}
    </section>;
}

</storycode:copy>
<a id="f-9f0dbf2758da-PreviewReading"></a>
### `PreviewReading`

<storycode:copy id="f-9f0dbf2758da-PreviewReading">
function PreviewReading({ value }: { value: unknown }) {
  useUiReading("change preview", value);
  return null;
}
</storycode:copy>

### Assemble `apps/web/src/components/ChangePreview.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ChangePreview.tsx"><storycode:paste select="#f-9f0dbf2758da-ChangePreview"/><storycode:paste select="#f-9f0dbf2758da-PreviewReading"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/ChangePreview.tsx](contract.md#output-9f0dbf2758da)
- [apps/web/src/components/InsertMenu.tsx](../../../mechanisms/apps/web/src/components/InsertMenu.tsx.md#output-c9a0adfc5a83)
- [apps/web/src/components/NewProjectDialog.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-65e48889b856)
- [apps/web/src/environments/EnvironmentPanel.tsx](../../../mechanisms/apps/web/src/environments/EnvironmentPanel.tsx.md#output-a428d5761b36)
- [apps/web/src/shell/FilesystemTreeEditor.tsx](../../../mechanisms/apps/web/src/shell/FilesystemTreeEditor.tsx.md#output-d30e9ea77e15)
- [apps/web/src/views/DocumentReading.tsx](../../../mechanisms/apps/web/src/views/DocumentReading.tsx.md#output-d10820ba127c)
- [apps/web/src/views/RepresentationPane.tsx](../../../mechanisms/apps/web/src/views/RepresentationPane.tsx.md#output-cd66c1c2250e)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/components/InsertMenu.test.tsx](../../../mechanisms/apps/web/src/components/InsertMenu.tsx.md#output-d0524ef3ebdd)
- [apps/web/src/components/NewProjectDialog.test.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-065ce188ea68)
- [apps/web/src/environments/EnvironmentPanel.test.tsx](../../../mechanisms/apps/web/src/environments/EnvironmentPanel.tsx.md#output-f41d08c2423e)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
