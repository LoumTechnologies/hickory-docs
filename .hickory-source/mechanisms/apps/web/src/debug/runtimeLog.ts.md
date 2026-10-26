<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/runtimeLog.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3b60e9e2ef5e"></a>
## `apps/web/src/debug/runtimeLog.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-3b60e9e2ef5e-Contributor"></a>
### `Contributor`

<storycode:copy id="f-3b60e9e2ef5e-Contributor">
import type { RuntimeEvent, RuntimeRun } from "./runtimeObservations";
import type { RibbonLink } from "../shell/Ribbons";
export interface Contributor { event: string; chunk_bytes: { start: number; end: number }; line_bytes: { start: number; end: number } }
</storycode:copy>
<a id="f-3b60e9e2ef5e-OutputLine"></a>
### `OutputLine`

<storycode:copy id="f-3b60e9e2ef5e-OutputLine">
export interface OutputLine { bytes: number[]; contributors: Contributor[]; terminated: boolean }
</storycode:copy>
<a id="f-3b60e9e2ef5e-outputLines"></a>
### `outputLines`

Same lossless LF framing as hick-observe::output_lines; decode after assembly.

<storycode:copy id="f-3b60e9e2ef5e-outputLines">
/** Same lossless LF framing as hick-observe::output_lines; decode after assembly. */
export function outputLines(run: RuntimeRun, stream: string, category: string): OutputLine[] {
  const result: OutputLine[] = [];
  let line: OutputLine = { bytes: [], contributors: [], terminated: false };
  for (const event of run.events) {
    const output = event.observation;
    if (output.kind !== "output" || output.stream !== stream || output.category !== category) continue;
    for (let start = 0; start < output.bytes.length;) {
      const newline = output.bytes.indexOf(10, start);
      const end = newline < 0 ? output.bytes.length : newline + 1;
      const at = line.bytes.length;
      line.bytes = line.bytes.concat(output.bytes.slice(start, end));
      line.contributors.push({ event: event.id, chunk_bytes: { start, end }, line_bytes: { start: at, end: line.bytes.length } });
      start = end;
      if (newline >= 0) { line.terminated = true; result.push(line); line = { bytes: [], contributors: [], terminated: false }; }
    }
  }
  if (line.bytes.length) result.push(line);
  return result;
}
</storycode:copy>
<a id="f-3b60e9e2ef5e-printable"></a>
### `printable`

<storycode:copy id="f-3b60e9e2ef5e-printable">
export const printable = (bytes: number[]) => new TextDecoder().decode(new Uint8Array(bytes)).replaceAll("\r", "\\r").replaceAll("\u001b", "\\x1b");
</storycode:copy>
<a id="f-3b60e9e2ef5e-logLinks"></a>
### `logLinks`

<storycode:copy id="f-3b60e9e2ef5e-logLinks">
export function logLinks(run: RuntimeRun, lines: OutputLine[], logPath: string, sourcePath: string, sourceId?: string): RibbonLink[] {
  const source = run.sources.find(source => source.id === sourceId) ?? run.sources[0];
  const text = new TextDecoder().decode(new Uint8Array(source.bytes));
  return lines.flatMap((line, index) => line.contributors.flatMap(contributor => {
    const event = run.events.find(item => item.id === contributor.event)!;
    const targets = [{ kind: "emission", location: event.emission }, ...event.callers.map(location => ({ kind: "calling context", location }))];
    return targets.flatMap(({ kind, location }, caller) => {
      if (!location?.bytes || location.source !== source.id || location.precision === "unknown" || location.precision === "block_only") return [];
      const sourceLine = new TextDecoder().decode(new Uint8Array(source.bytes.slice(0, location.bytes.start))).split("\n").length;
      return [{ key: `${run.id}:${event.id}:line:${index}:${caller}`, family: "execution" as const,
        from: { path: logPath, lines: [index + 1, index + 1] as [number, number] }, to: { path: sourcePath, lines: [sourceLine, sourceLine] as [number, number] },
        title: `Execution ${kind} · run ${run.id} · event ${event.id} · revision ${source.revision} · ${location.precision}`,
        review: { kind: "runtime-relationship", path: source.uri, quote: text.split("\n")[sourceLine - 1], source_hash: source.revision,
          lines: [sourceLine, sourceLine] as [number, number], relationship: { kind, run: run.id, event, contributor, source_revision: source.revision } } }];
    });
  }));
}
</storycode:copy>
<a id="f-3b60e9e2ef5e-eventById"></a>
### `eventById`

<storycode:copy id="f-3b60e9e2ef5e-eventById">
export function eventById(run: RuntimeRun, id: string): RuntimeEvent | undefined { return run.events.find(event => event.id === id); }
</storycode:copy>

### Assemble `apps/web/src/debug/runtimeLog.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/runtimeLog.ts"><storycode:paste select="#f-3b60e9e2ef5e-Contributor"/><storycode:paste select="#f-3b60e9e2ef5e-OutputLine"/><storycode:paste select="#f-3b60e9e2ef5e-outputLines"/><storycode:paste select="#f-3b60e9e2ef5e-printable"/><storycode:paste select="#f-3b60e9e2ef5e-logLinks"/><storycode:paste select="#f-3b60e9e2ef5e-eventById"/></storycode:file>

</storycode:doc>
