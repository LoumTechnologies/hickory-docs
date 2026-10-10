import { revealLine, claimReveal } from "../lib/revealLine";
import { samePath } from "../lib/paths";
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
