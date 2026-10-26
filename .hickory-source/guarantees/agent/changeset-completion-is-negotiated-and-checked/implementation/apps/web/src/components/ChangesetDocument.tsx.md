<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/ChangesetDocument.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-60692076babd"></a>
## `apps/web/src/components/ChangesetDocument.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Finish an agent turn against its captured project boundary](../../../../../contract.md).

<a id="f-60692076babd-noCells"></a>
### `noCells`

<storycode:copy id="f-60692076babd-noCells">
import { useUiReading } from "../lib/uiReading";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { computeEdits } from "../lib/diff";
import { LocalRealtime } from "../api/realtime";
import { DocumentEditor } from "../editor/DocumentEditor";
import { materializeProject, type ProjectEntry } from "../editor/hickLang";

const noCells = new Set<string>();
</storycode:copy>
<a id="f-60692076babd-ChangesetDocument"></a>
### `ChangesetDocument`

Accepted artifacts are immutable records. Their patches use ordinary note rendering.

<storycode:copy id="f-60692076babd-ChangesetDocument">
/** Accepted artifacts are immutable records. Their patches use ordinary note rendering. */
export function ChangesetDocument({ entry, sources, input, tools = true, filesystemOnly = false, onEdit, onPending, identity = "final" }: {
  identity?: string; entry: string; sources: Record<string,string>; input: ProjectEntry[]; tools?: boolean; filesystemOnly?: boolean; onEdit?: (sources: Record<string,string>) => void; onPending?: (pending: boolean) => void;
}) {
  const [activePatch,setActivePatch] = useState<string | null>(null);
  const draftPending = useCallback((id: string, pending: boolean) => {
    setActivePatch(pending ? id : null);
    onPending?.(pending);
  },[onPending]);
  const realtime = useMemo(() => new LocalRealtime(), []);
  const documents = useMemo(() => [entry,...Object.keys(sources).filter(path => path !== entry)].map(document => ({
    document, project: {entry,document,sources,input,tools,filesystemOnly,editPatch:onEdit,draftPending,activePatch},
  })),[entry,sources,input,tools,filesystemOnly,onEdit,draftPending,activePatch]);
  useEffect(() => () => realtime.close(),[realtime]);
  // Check the closure locally before exposing native tools. Never substitute disk bytes.
  const error = useMemo(() => {
    try { materializeProject(entry,sources,input); return ""; }
    catch (e) { return e instanceof Error ? e.message : String(e); }
  },[entry,sources,input]);
  useUiReading("final changeset rendering status", { identity, entry, error: error || null, message: error ? `Could not render final changeset: ${error}` : null, source_read: false }, identity);
  if (error) return <p role="alert">Could not render final changeset: {error}</p>;
  return <section aria-label="Final literate changeset">
    {documents.map(({document,project}) => <div key={document}>
      {documents.length > 1 && <h3>{document}</h3>}
      <CapturedSource docId={`${identity}:${document}`} initialSource={sources[document]} realtime={realtime}
        capturedProject={project} readOnly preserveBytes execBlocks={[]}
        runningCells={noCells} onRunCell={() => {}} />
    </div>)}
  </section>;
}

</storycode:copy>
<a id="f-60692076babd-CapturedSource"></a>
### `CapturedSource`

<storycode:copy id="f-60692076babd-CapturedSource">
function CapturedSource(props: React.ComponentProps<typeof DocumentEditor>) {
  const view = useRef<EditorView | null>(null);
  const sync = useCallback((target: EditorView | null) => {
    view.current = target;
    if (!target) return;
    const previous = target.state.doc.toString();
    if (previous !== props.initialSource) target.dispatch({changes:computeEdits(previous,props.initialSource).map(e => ({from:e.start,to:e.end,insert:e.text}))});
  },[props.initialSource]);
  useEffect(() => { sync(view.current); },[sync]);
  return <DocumentEditor {...props} onViewReady={sync} />;
}
</storycode:copy>

### Assemble `apps/web/src/components/ChangesetDocument.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ChangesetDocument.tsx"><storycode:paste select="#f-60692076babd-noCells"/><storycode:paste select="#f-60692076babd-ChangesetDocument"/><storycode:paste select="#f-60692076babd-CapturedSource"/></storycode:file>

</storycode:doc>
