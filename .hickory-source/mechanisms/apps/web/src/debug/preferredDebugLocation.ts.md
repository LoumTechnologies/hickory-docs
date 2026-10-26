<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/preferredDebugLocation.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-de2c3565a1cd"></a>
## `apps/web/src/debug/preferredDebugLocation.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-de2c3565a1cd-Reveal"></a>
### `Reveal`

<storycode:copy id="f-de2c3565a1cd-Reveal">
import { representationOffset, type RepresentedFile } from "../lsp/representationMapping";
import type { Frame } from "./client";

type Reveal = (frame: Frame) => boolean;
</storycode:copy>
<a id="f-de2c3565a1cd-surfaces"></a>
### `surfaces`

<storycode:copy id="f-de2c3565a1cd-surfaces">
const surfaces = new Map<string, Reveal>();
</storycode:copy>
<a id="f-de2c3565a1cd-preferred"></a>
### `preferred`

<storycode:copy id="f-de2c3565a1cd-preferred">
const preferred = new Map<string, string>();
</storycode:copy>
<a id="f-de2c3565a1cd-projectSurfaces"></a>
### `projectSurfaces`

<storycode:copy id="f-de2c3565a1cd-projectSurfaces">
const projectSurfaces = new Map<string, () => void>();

</storycode:copy>
<a id="f-de2c3565a1cd-preferDebugSurface"></a>
### `preferDebugSurface`

A launch remembers its reading surface throughout stepping across files.

<storycode:copy id="f-de2c3565a1cd-preferDebugSurface">
/** A launch remembers its reading surface throughout stepping across files. */
export function preferDebugSurface(owner: string, surface: string): void {
  preferred.set(owner, surface);
}

</storycode:copy>
<a id="f-de2c3565a1cd-clearPreferredDebugSurface"></a>
### `clearPreferredDebugSurface`

<storycode:copy id="f-de2c3565a1cd-clearPreferredDebugSurface">
export function clearPreferredDebugSurface(owner: string): void {
  preferred.delete(owner);
  projectSurfaces.get(owner)?.();
  projectSurfaces.delete(owner);
}

</storycode:copy>
<a id="f-de2c3565a1cd-registerDebugSurface"></a>
### `registerDebugSurface`

<storycode:copy id="f-de2c3565a1cd-registerDebugSurface">
export function registerDebugSurface(id: string, reveal: Reveal): () => void {
  surfaces.set(id, reveal);
  return () => { if (surfaces.get(id) === reveal) surfaces.delete(id); };
}

</storycode:copy>
<a id="f-de2c3565a1cd-revealPreferredDebugLocation"></a>
### `revealPreferredDebugLocation`

Only an exact mapping may consume navigation; otherwise open the source.

<storycode:copy id="f-de2c3565a1cd-revealPreferredDebugLocation">
/** Only an exact mapping may consume navigation; otherwise open the source. */
export function revealPreferredDebugLocation(owner: string, frame: Frame): boolean {
  const id = preferred.get(owner);
  return id !== undefined && (surfaces.get(id)?.(frame) ?? false);
}

</storycode:copy>
<a id="f-de2c3565a1cd-representedFrameOffset"></a>
### `representedFrameOffset`

DAP lines must exist; the editor position helpers otherwise clamp them.

<storycode:copy id="f-de2c3565a1cd-representedFrameOffset">
/** DAP lines must exist; the editor position helpers otherwise clamp them. */
export function representedFrameOffset(source: string, files: readonly RepresentedFile[], frame: Frame): number | null {
  const file = files.find(file => file.path === frame.source);
  const line = frame.source_line;
  if (!file || typeof line !== "number" || !Number.isInteger(line) || line < 0 || line >= file.content.split("\n").length) return null;
  return representationOffset(source, file, { line, character: 0 });
}

</storycode:copy>
<a id="f-de2c3565a1cd-preferProjectDebugSurface"></a>
### `preferProjectDebugSurface`

Captured project frames retain their exact byte origins in authored notes.

<storycode:copy id="f-de2c3565a1cd-preferProjectDebugSurface">
/** Captured project frames retain their exact byte origins in authored notes. */
export function preferProjectDebugSurface(owner: string, reveal: Reveal, dispose: () => void): void {
  clearPreferredDebugSurface(owner);
  const id = `project:${owner}`;
  const unregister = registerDebugSurface(id, reveal);
  projectSurfaces.set(owner, () => { unregister(); dispose(); });
  preferDebugSurface(owner, id);
}
</storycode:copy>

### Assemble `apps/web/src/debug/preferredDebugLocation.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/preferredDebugLocation.ts"><storycode:paste select="#f-de2c3565a1cd-Reveal"/><storycode:paste select="#f-de2c3565a1cd-surfaces"/><storycode:paste select="#f-de2c3565a1cd-preferred"/><storycode:paste select="#f-de2c3565a1cd-projectSurfaces"/><storycode:paste select="#f-de2c3565a1cd-preferDebugSurface"/><storycode:paste select="#f-de2c3565a1cd-clearPreferredDebugSurface"/><storycode:paste select="#f-de2c3565a1cd-registerDebugSurface"/><storycode:paste select="#f-de2c3565a1cd-revealPreferredDebugLocation"/><storycode:paste select="#f-de2c3565a1cd-representedFrameOffset"/><storycode:paste select="#f-de2c3565a1cd-preferProjectDebugSurface"/></storycode:file>

</storycode:doc>
