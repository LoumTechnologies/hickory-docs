<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/coalesceRibbons.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Ribbon Is Drawn Where It Is Asked For](../../../../../../guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for/contract.md)

Captured SHA-256: `2f1f25d8530795b9459e68db133cd15c29447fb3d81f5a678cec1a9b236fcf9a`.

<book:entry path="apps/web/src/lib/coalesceRibbons.ts">
import type { Ribbon } from "./ribbons";

/** Presentation only: never alter the byte-precise provenance used for edits. */
export function coalesceRibbons(ribbons: readonly Ribbon[], source: string, output: string, terminal = false): Ribbon[] {
  const groups: Ribbon[] = [];
  for (const ribbon of [...ribbons].sort((a, b) => a.sourceSpan[0] - b.sourceSpan[0])) {
    const previous = [...groups].reverse().find(candidate => candidate.kind === ribbon.kind
      && candidate.whitespaceOnly === ribbon.whitespaceOnly
      && candidate.sourceSpan[1] <= ribbon.sourceSpan[0]
      && source.slice(candidate.sourceSpan[1], ribbon.sourceSpan[0]).trim() === ""
      && (terminal || (candidate.outputRange[1] <= ribbon.outputRange[0]
        && output.slice(candidate.outputRange[1], ribbon.outputRange[0]).trim() === "")
        || candidate.outputRange.every((offset, i) => offset === ribbon.outputRange[i])));
    if (!previous) { groups.push({ ...ribbon }); continue; }
    previous.sourceSpan = [previous.sourceSpan[0], ribbon.sourceSpan[1]];
    previous.sourceByteSpan = [previous.sourceByteSpan[0], ribbon.sourceByteSpan[1]];
    if (!terminal) previous.outputRange = [previous.outputRange[0], ribbon.outputRange[1]];
    previous.bytes += ribbon.bytes;
  }
  return groups;
}
</book:entry>

</book:doc>
