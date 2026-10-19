import type { RibbonLink, RibbonSource } from "./Ribbons";
import { samePath } from "../lib/paths";

/** Adjacent document passages citing the same destination share one shape. */
export function coalesceLinks(links: readonly RibbonLink[], sources: readonly RibbonSource[]): RibbonLink[] {
  const groups: RibbonLink[] = [];
  for (const link of [...links].sort((a, b) => a.from.lines[0] - b.from.lines[0])) {
    const source = sources.find(s => samePath(s.docPath, link.from.path));
    const text = source?.view?.state.doc.toString() ?? source?.docSource;
    const previous = text === undefined || link.anchor || link.family === "execution" ? undefined : [...groups].reverse().find(candidate =>
      !candidate.anchor && candidate.family === link.family && candidate.alwaysVisible === link.alwaysVisible
      && samePath(candidate.from.path, link.from.path) && samePath(candidate.to.path, link.to.path)
      && candidate.to.kind === link.to.kind && String(candidate.to.lines) === String(link.to.lines)
      && candidate.from.lines[1] <= link.from.lines[0]
      && text.split("\n").slice(candidate.from.lines[1], link.from.lines[0] - 1).join("\n").trim() === "");
    if (previous) previous.from = { ...previous.from, lines: [previous.from.lines[0], link.from.lines[1]] };
    else groups.push({ ...link, from: { ...link.from } });
  }
  return groups;
}
