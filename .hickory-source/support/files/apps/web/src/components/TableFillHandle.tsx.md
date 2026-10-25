<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/TableFillHandle.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `39d663e6efbc62252adac2431caaeee9f16ec6299ddc447716c8c78298f9432f`.

<book:entry path="apps/web/src/components/TableFillHandle.tsx">
import type { MouseEvent } from "react";
import "./TableFillHandle.css";

export function TableFillHandle({ onStart, range }: { onStart: (event: MouseEvent) => void; range?: string }) {
  return <><button type="button" className="table-fill-handle" aria-label="Fill selected cells"
    data-tip="Drag up, down, left or right to fill; Escape cancels"
    onMouseDown={onStart} onClick={event => event.stopPropagation()}
    onDoubleClick={event => event.stopPropagation()}>+</button>{range && <output className="table-fill-range" aria-live="polite">Fill {range}</output>}</>;
}
</book:entry>

</book:doc>
