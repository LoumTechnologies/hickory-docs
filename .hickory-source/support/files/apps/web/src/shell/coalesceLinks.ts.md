<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/coalesceLinks.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Ribbon Is Drawn Where It Is Asked For](../../../../../../guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for/contract.md)

Captured SHA-256: `df384bfb9e94248f66493daafbf94215e6302609e7b629399753e85c7b8e94fd`.

<book:entry path="apps/web/src/shell/coalesceLinks.ts">
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
</book:entry>

</book:doc>
