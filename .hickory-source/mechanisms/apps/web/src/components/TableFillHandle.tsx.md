<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/TableFillHandle.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-afeae8f13474"></a>
## `apps/web/src/components/TableFillHandle.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-afeae8f13474-TableFillHandle"></a>
### `TableFillHandle`

<storycode:copy id="f-afeae8f13474-TableFillHandle">
import type { MouseEvent } from "react";
import "./TableFillHandle.css";

export function TableFillHandle({ onStart, range }: { onStart: (event: MouseEvent) => void; range?: string }) {
  return <><button type="button" className="table-fill-handle" aria-label="Fill selected cells"
    data-tip="Drag up, down, left or right to fill; Escape cancels"
    onMouseDown={onStart} onClick={event => event.stopPropagation()}
    onDoubleClick={event => event.stopPropagation()}>+</button>{range && <output className="table-fill-range" aria-live="polite">Fill {range}</output>}</>;
}
</storycode:copy>

### Assemble `apps/web/src/components/TableFillHandle.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/TableFillHandle.tsx"><storycode:paste select="#f-afeae8f13474-TableFillHandle"/></storycode:file>

</storycode:doc>
