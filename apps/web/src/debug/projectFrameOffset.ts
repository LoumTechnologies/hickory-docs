import type { ProjectEntry } from "../editor/hickLang";
import type { Frame } from "./client";
import { byteToChar } from "../lib/offsets";

/** Resolve only a unique, byte-exact surviving origin from this capture. */
export function projectFrameOffset(prefix: string, entries: readonly ProjectEntry[], document: string, source: string, frame: Frame): number | null {
  if (!frame.source?.startsWith(prefix) || !Number.isInteger(frame.source_line) || frame.source_line! < 0) return null;
  const file = entries.find(e => `${prefix}${e.path}` === frame.source);
  if (!file) return null;
  const text = new TextDecoder("utf-8", {fatal:true}).decode(new Uint8Array(file.bytes));
  const lines = text.split("\n"), line = frame.source_line!;
  if (line >= lines.length) return null;
  const at = new TextEncoder().encode(lines.slice(0, line).map(s => s + "\n").join("")).length;
  const origins = file.origins?.filter(o => o.output[0] <= at && at < o.output[1]);
  if (origins?.length !== 1) return null;
  const origin = origins[0];
  if (origin.document !== document || origin.output[1] - origin.output[0] !== origin.source[1] - origin.source[0]) return null;
  const bytes = new TextEncoder().encode(source);
  if (file.bytes.slice(...origin.output).some((b, i) => bytes[origin.source[0] + i] !== b)) return null;
  return byteToChar(source, origin.source[0] + at - origin.output[0]);
}
