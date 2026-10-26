<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/notebookOutput.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7fedff6cdb75"></a>
## `apps/web/src/lib/notebookOutput.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-7fedff6cdb75-NotebookOutput"></a>
### `NotebookOutput`

<storycode:copy id="f-7fedff6cdb75-NotebookOutput">
import type { NotebookTable } from "../components/ReadOnlyTable";
export type NotebookOutput = NotebookTable | { kind: "image"; mime: string; data: string } | { kind: "text"; text: string };
</storycode:copy>
<a id="f-7fedff6cdb75-DISPLAY_PREFIX"></a>
### `DISPLAY_PREFIX`

<storycode:copy id="f-7fedff6cdb75-DISPLAY_PREFIX">
export const DISPLAY_PREFIX = "\x1eHICKORY_DISPLAY ";
</storycode:copy>
<a id="f-7fedff6cdb75-imageTypes"></a>
### `imageTypes`

<storycode:copy id="f-7fedff6cdb75-imageTypes">
const imageTypes = new Set(["image/png", "image/jpeg", "image/webp", "image/svg+xml"]);
</storycode:copy>
<a id="f-7fedff6cdb75-valid"></a>
### `valid`

<storycode:copy id="f-7fedff6cdb75-valid">
function valid(value: unknown): value is NotebookOutput {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  if (v.kind === "text") return typeof v.text === "string";
  if (v.kind === "image") return typeof v.mime === "string" && imageTypes.has(v.mime) && typeof v.data === "string" && /^[A-Za-z0-9+/]*={0,2}$/.test(v.data);
  return v.kind === "table" && Array.isArray(v.schema) && v.schema.every(c => c && typeof c.name === "string" && typeof c.dtype === "string") && Array.isArray(v.rows) && v.rows.every(Array.isArray);
}
</storycode:copy>
<a id="f-7fedff6cdb75-notebookOutputs"></a>
### `notebookOutputs`

Parse after joining stdout chunks: a protocol record may cross events.

<storycode:copy id="f-7fedff6cdb75-notebookOutputs">
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
</storycode:copy>

### Assemble `apps/web/src/lib/notebookOutput.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/notebookOutput.ts"><storycode:paste select="#f-7fedff6cdb75-NotebookOutput"/><storycode:paste select="#f-7fedff6cdb75-DISPLAY_PREFIX"/><storycode:paste select="#f-7fedff6cdb75-imageTypes"/><storycode:paste select="#f-7fedff6cdb75-valid"/><storycode:paste select="#f-7fedff6cdb75-notebookOutputs"/></storycode:file>

</storycode:doc>
