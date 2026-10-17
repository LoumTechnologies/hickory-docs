import { parseHickDoc } from "../editor/hickDoc";

/** The clicked construction, never a global project-version preference. */
export function documentDebugContext(path: string, source: string, at?: number): string {
  const doc = parseHickDoc(source);
  if (at === undefined || !doc.blocks.some(b => b.name === "project")) return "new";
  const block = doc.blocks.find(b => b.name === "file" && b.from === at);
  if (!block) throw new Error("This file block changed. Press its Debug icon again.");
  // Parser spans exclude the optional BOM; editor positions include it.
  const byte = new TextEncoder().encode(source.slice(source.startsWith("\uFEFF") ? 1 : 0, at)).length;
  const id = block.attrs.id ?? `${path.split("/").pop()}:${byte}`;
  return `latest-file:${JSON.stringify([id, source])}`;
}
