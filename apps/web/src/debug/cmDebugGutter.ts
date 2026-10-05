import { GutterMarker } from "@codemirror/view";

/** Carries the source line even while CodeMirror is remeasuring decorated text. */
class BreakpointTarget extends GutterMarker {
  constructor(readonly line: number) { super(); }
  eq(other: GutterMarker) { return other instanceof BreakpointTarget && other.line === this.line; }
  toDOM() {
    const dot = document.createElement("span");
    dot.className = "cm-bp-ghost";
    dot.dataset.tip = "Click to set a breakpoint";
    dot.dataset.breakpointLine = String(this.line);
    dot.setAttribute("aria-label", `Set breakpoint on line ${this.line + 1}`);
    return dot;
  }
}
export const breakpointTarget = (line: number): GutterMarker => new BreakpointTarget(line);
