<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/cmDebugGutter.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6d9600b313f3"></a>
## `apps/web/src/debug/cmDebugGutter.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-6d9600b313f3-BreakpointTarget"></a>
### `BreakpointTarget`

<storycode:copy id="f-6d9600b313f3-BreakpointTarget">
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
</storycode:copy>
<a id="f-6d9600b313f3-breakpointTarget"></a>
### `breakpointTarget`

<storycode:copy id="f-6d9600b313f3-breakpointTarget">
export const breakpointTarget = (line: number): GutterMarker => new BreakpointTarget(line);
</storycode:copy>

### Assemble `apps/web/src/debug/cmDebugGutter.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/cmDebugGutter.ts"><storycode:paste select="#f-6d9600b313f3-BreakpointTarget"/><storycode:paste select="#f-6d9600b313f3-breakpointTarget"/></storycode:file>

</storycode:doc>
