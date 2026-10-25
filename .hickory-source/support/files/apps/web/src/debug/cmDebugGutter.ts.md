<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/cmDebugGutter.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `188dde80371992b52bfe4e1c87c2635ccb58b6da3821ad7db36c7a8333c9ef42`.

<book:entry path="apps/web/src/debug/cmDebugGutter.ts">
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
</book:entry>

</book:doc>
