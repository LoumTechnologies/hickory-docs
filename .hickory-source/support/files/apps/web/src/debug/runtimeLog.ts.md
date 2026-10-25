<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/runtimeLog.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `14ae4fb006843994271f5e48964bfbfff3a9d281e944186735cdaa0d170c28cf`.

<book:entry path="apps/web/src/debug/runtimeLog.ts">
import type { RuntimeEvent, RuntimeRun } from "./runtimeObservations";
import type { RibbonLink } from "../shell/Ribbons";
export interface Contributor { event: string; chunk_bytes: { start: number; end: number }; line_bytes: { start: number; end: number } }
export interface OutputLine { bytes: number[]; contributors: Contributor[]; terminated: boolean }
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
export const printable = (bytes: number[]) => new TextDecoder().decode(new Uint8Array(bytes)).replaceAll("\r", "\\r").replaceAll("\u001b", "\\x1b");
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
export function eventById(run: RuntimeRun, id: string): RuntimeEvent | undefined { return run.events.find(event => event.id === id); }
</book:entry>

</book:doc>
