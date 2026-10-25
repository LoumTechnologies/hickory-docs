<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/fileProblems.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Plain File Has The Same Language Server A Document Has](../../../../../contract.md)

Captured SHA-256: `cca11d9c842830abd3db1e804519afe4ed5dd481ed17691ec0bff6c11978aaa3`.

<book:entry path="apps/web/src/lib/fileProblems.ts">
// The problems in plain files, for the count and the list.
//
// A document's diagnostics live on its session, and the status bar reads them
// from the registry. A plain file has no session — its pane is the whole of
// it — so its diagnostics are put here, keyed by path, and the workspace adds
// them in. Set only from the pane that owns the file, cleared when it goes:
// a count that includes a file nobody has open is a count that cannot be
// expanded into anything.

import { useSyncExternalStore } from "react";
import type { LspDiagnostic } from "../lsp/client";

const problems = new Map<string, LspDiagnostic[]>();
const listeners = new Set<() => void>();
let version = 0;

function bump() {
  version++;
  for (const listener of listeners) listener();
}

/** Record what the server said about `path`; an empty list clears it. */
export function setFileProblems(path: string, diagnostics: LspDiagnostic[]): void {
  if (diagnostics.length === 0) {
    if (!problems.delete(path)) return;
  } else {
    problems.set(path, diagnostics);
  }
  bump();
}

/** The pane closed: nothing about this file is known any more. */
export function forgetFileProblems(path: string): void {
  if (problems.delete(path)) bump();
}

/** Every file with problems, for the workspace to add to its own. */
export function allFileProblems(): { path: string; diagnostics: LspDiagnostic[] }[] {
  return [...problems.entries()].map(([path, diagnostics]) => ({ path, diagnostics }));
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** A version that changes whenever any file's problems do. */
export function useFileProblemsVersion(): number {
  return useSyncExternalStore(subscribe, () => version, () => version);
}

/** Test seam. */
export function resetFileProblems(): void {
  problems.clear();
  bump();
}
</book:entry>

</book:doc>
