<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/spatialNavigation.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md)

Captured SHA-256: `aad7a5ae9fff5d4fee3a43a1c1006e80bba208026d56798ade5afa77ff043b0e`.

<book:entry path="apps/web/src/shell/spatialNavigation.ts">
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

export const SPATIAL_RIBBON_OPEN = "hickory:spatial-ribbon-open";
export interface SpatialRibbonOpen { path: string; side?: "left" | "right"; central: boolean }
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
</book:entry>

</book:doc>
