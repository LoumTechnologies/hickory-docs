<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/spatialNavigation.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-aba5508441a4"></a>
## `apps/web/src/shell/spatialNavigation.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md).

<a id="f-aba5508441a4-revealSpatialSource"></a>
### `revealSpatialSource`

<storycode:copy id="f-aba5508441a4-revealSpatialSource">
import { revealLine, claimReveal } from "../lib/revealLine";
import { samePath } from "../lib/paths";
import type { RibbonOpenIntent } from "./ribbonNavigation";
import type { RibbonFile, RibbonSource, RibbonTarget } from "./Ribbons";

/** Follow an already-mounted lens without opening a second document over it. */
export function revealSpatialSource(container: HTMLElement | null, sources: readonly RibbonSource[], files: readonly RibbonFile[], target: RibbonTarget): boolean {
  if (target.kind !== "path" || !container?.querySelector("[data-spatial-shell]")) return false;
  const view = sources.find(source => source.view && samePath(source.docPath, target.path))?.view ?? files.find(file => file.view && samePath(file.file.path, target.path))?.view;
  if (!view?.dom.closest("[data-spatial-key]")) return false;
  const line = target.lines?.[0] ?? 1;
  revealLine(target.path, line); claimReveal(target.path);
  const position = view.state.doc.line(Math.max(1, Math.min(view.state.doc.lines, line))).from;
  view.dispatch({ selection: { anchor: position }, scrollIntoView: true });
  requestAnimationFrame(() => { if (view.dom.isConnected) { view.focus(); view.requestMeasure(); } });
  return true;
}

</storycode:copy>
<a id="f-aba5508441a4-SPATIAL_RIBBON_OPEN"></a>
### `SPATIAL_RIBBON_OPEN`

<storycode:copy id="f-aba5508441a4-SPATIAL_RIBBON_OPEN">
export const SPATIAL_RIBBON_OPEN = "hickory:spatial-ribbon-open";
</storycode:copy>
<a id="f-aba5508441a4-SpatialRibbonOpen"></a>
### `SpatialRibbonOpen`

<storycode:copy id="f-aba5508441a4-SpatialRibbonOpen">
export interface SpatialRibbonOpen { path: string; side?: "left" | "right"; central: boolean }
</storycode:copy>
<a id="f-aba5508441a4-requestSpatialRibbonOpen"></a>
### `requestSpatialRibbonOpen`

Capture the destination before normal tab opening and reveal change focus.

<storycode:copy id="f-aba5508441a4-requestSpatialRibbonOpen">
/** Capture the destination before normal tab opening and reveal change focus. */
export function requestSpatialRibbonOpen(container: HTMLElement | null, target: RibbonTarget, intent?: RibbonOpenIntent): boolean {
  const shell = container?.querySelector<HTMLElement>("[data-spatial-shell]");
  if (!shell) return false;
  const destination = [...shell.querySelectorAll<HTMLElement>("[data-spatial-key]")].find(el => samePath(el.dataset.shellTarget ?? "", target.path));
  const side = destination?.dataset.spatialSide as "left" | "right" | undefined;
  shell.dispatchEvent(new CustomEvent<SpatialRibbonOpen>(SPATIAL_RIBBON_OPEN, { detail: {
    path: target.path, central: destination?.dataset.spatialCenter === "true" || destination?.dataset.spatialTiled === "true",
    side: side ?? (intent?.direction === "right" ? "right" : "left"),
  } }));
  return true;
}
</storycode:copy>

### Assemble `apps/web/src/shell/spatialNavigation.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/spatialNavigation.ts"><storycode:paste select="#f-aba5508441a4-revealSpatialSource"/><storycode:paste select="#f-aba5508441a4-SPATIAL_RIBBON_OPEN"/><storycode:paste select="#f-aba5508441a4-SpatialRibbonOpen"/><storycode:paste select="#f-aba5508441a4-requestSpatialRibbonOpen"/></storycode:file>

</storycode:doc>
