<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/NotebookOutputs.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-144e02a7d7e9"></a>
## `apps/web/src/components/NotebookOutputs.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-144e02a7d7e9-NotebookOutputs"></a>
### `NotebookOutputs`

<storycode:copy id="f-144e02a7d7e9-NotebookOutputs">
import type { NotebookOutput } from "../lib/notebookOutput";
import { ReadOnlyTable } from "./ReadOnlyTable";
export function NotebookOutputs({ outputs }: { outputs: NotebookOutput[] }) {
  return <div className="notebook-outputs">{outputs.map((output, i) => output.kind === "table" ? <ReadOnlyTable key={i} table={output} /> : output.kind === "image" ? <figure key={i}><img alt="Python cell output" src={`data:${output.mime};base64,${output.data}`} /></figure> : <pre key={i}>{output.text}</pre>)}</div>;
}
</storycode:copy>

### Assemble `apps/web/src/components/NotebookOutputs.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/NotebookOutputs.tsx"><storycode:paste select="#f-144e02a7d7e9-NotebookOutputs"/></storycode:file>

</storycode:doc>
