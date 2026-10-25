<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/notebookOutput.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `84816fd6b37f70e676553f6bf897573b64b471debac8bbb6e9e78a4abc359fbf`.

<book:entry path="apps/web/src/lib/notebookOutput.ts">
import type { NotebookTable } from "../components/ReadOnlyTable";
export type NotebookOutput = NotebookTable | { kind: "image"; mime: string; data: string } | { kind: "text"; text: string };
export const DISPLAY_PREFIX = "\x1eHICKORY_DISPLAY ";
const imageTypes = new Set(["image/png", "image/jpeg", "image/webp", "image/svg+xml"]);
function valid(value: unknown): value is NotebookOutput {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  if (v.kind === "text") return typeof v.text === "string";
  if (v.kind === "image") return typeof v.mime === "string" && imageTypes.has(v.mime) && typeof v.data === "string" && /^[A-Za-z0-9+/]*={0,2}$/.test(v.data);
  return v.kind === "table" && Array.isArray(v.schema) && v.schema.every(c => c && typeof c.name === "string" && typeof c.dtype === "string") && Array.isArray(v.rows) && v.rows.every(Array.isArray);
}
/** Parse after joining stdout chunks: a protocol record may cross events. */
export function notebookOutputs(stdout: string): { outputs: NotebookOutput[]; text: string } {
  const outputs: NotebookOutput[] = [];
  const text: string[] = [];
  for (const line of stdout.split("\n")) {
    if (line.startsWith(DISPLAY_PREFIX) && line.length < 8 * 1024 * 1024) {
      try {
        const value: unknown = JSON.parse(line.slice(DISPLAY_PREFIX.length));
        if (valid(value)) { outputs.push(value); continue; }
      } catch { /* Invalid records remain visible as ordinary output. */ }
    }
    text.push(line);
  }
  return { outputs, text: text.join("\n") };
}
</book:entry>

</book:doc>
