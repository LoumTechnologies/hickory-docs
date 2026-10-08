import type { MouseEvent } from "react";
import "./TableFillHandle.css";

export function TableFillHandle({ onStart, range }: { onStart: (event: MouseEvent) => void; range?: string }) {
  return <><button type="button" className="table-fill-handle" aria-label="Fill selected cells"
    data-tip="Drag up, down, left or right to fill; Escape cancels"
    onMouseDown={onStart} onClick={event => event.stopPropagation()}
    onDoubleClick={event => event.stopPropagation()}>+</button>{range && <output className="table-fill-range" aria-live="polite">Fill {range}</output>}</>;
}
