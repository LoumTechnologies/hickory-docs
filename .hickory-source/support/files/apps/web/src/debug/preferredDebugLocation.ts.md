<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/preferredDebugLocation.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `307c166741bc1e26372242cc4fb3bcfa959c2c4d70669771eef03d557dd896ff`.

<book:entry path="apps/web/src/debug/preferredDebugLocation.ts">
import { representationOffset, type RepresentedFile } from "../lsp/representationMapping";
import type { Frame } from "./client";

type Reveal = (frame: Frame) => boolean;
const surfaces = new Map<string, Reveal>();
const preferred = new Map<string, string>();
const projectSurfaces = new Map<string, () => void>();

/** A launch remembers its reading surface throughout stepping across files. */
export function preferDebugSurface(owner: string, surface: string): void {
  preferred.set(owner, surface);
}

export function clearPreferredDebugSurface(owner: string): void {
  preferred.delete(owner);
  projectSurfaces.get(owner)?.();
  projectSurfaces.delete(owner);
}

export function registerDebugSurface(id: string, reveal: Reveal): () => void {
  surfaces.set(id, reveal);
  return () => { if (surfaces.get(id) === reveal) surfaces.delete(id); };
}

/** Only an exact mapping may consume navigation; otherwise open the source. */
export function revealPreferredDebugLocation(owner: string, frame: Frame): boolean {
  const id = preferred.get(owner);
  return id !== undefined && (surfaces.get(id)?.(frame) ?? false);
}

/** DAP lines must exist; the editor position helpers otherwise clamp them. */
export function representedFrameOffset(source: string, files: readonly RepresentedFile[], frame: Frame): number | null {
  const file = files.find(file => file.path === frame.source);
  const line = frame.source_line;
  if (!file || typeof line !== "number" || !Number.isInteger(line) || line < 0 || line >= file.content.split("\n").length) return null;
  return representationOffset(source, file, { line, character: 0 });
}

/** Captured project frames retain their exact byte origins in authored notes. */
export function preferProjectDebugSurface(owner: string, reveal: Reveal, dispose: () => void): void {
  clearPreferredDebugSurface(owner);
  const id = `project:${owner}`;
  const unregister = registerDebugSurface(id, reveal);
  projectSurfaces.set(owner, () => { unregister(); dispose(); });
  preferDebugSurface(owner, id);
}
</book:entry>

</book:doc>
