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
