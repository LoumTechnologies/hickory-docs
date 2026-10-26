<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/pausedElsewhere.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f89ec0ad0c79"></a>
## `apps/web/src/lib/pausedElsewhere.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Has The Same Debugger A Document Has](../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md).

<a id="f-f89ec0ad0c79-PausedElsewhere"></a>
### `PausedElsewhere`

Where execution is paused, when that is not the file being debugged.
A plain file's debugger steps into `src/lib.rs`; the pane for `src/main.rs`
owns the session and cannot draw a line in a file it does not hold. So the
owner publishes "paused at this file, this line" here, the workspace opens
that file's tab, and that tab's pane draws the paused line as if the
session were its own — which, for reading, it is.
One value, not a map: a person is stopped in one place at a time. The
owner is recorded so that only the session that published a position can
clear it, and a neighbour's pane finishing its own run does not wipe the
arrow out from under a session that is still paused.

<storycode:copy id="f-f89ec0ad0c79-PausedElsewhere">
// Where execution is paused, when that is not the file being debugged.
//
// A plain file's debugger steps into `src/lib.rs`; the pane for `src/main.rs`
// owns the session and cannot draw a line in a file it does not hold. So the
// owner publishes "paused at this file, this line" here, the workspace opens
// that file's tab, and that tab's pane draws the paused line as if the
// session were its own — which, for reading, it is.
//
// One value, not a map: a person is stopped in one place at a time. The
// owner is recorded so that only the session that published a position can
// clear it, and a neighbour's pane finishing its own run does not wipe the
// arrow out from under a session that is still paused.

import { useSyncExternalStore } from "react";

export interface PausedElsewhere {
  /** Root-relative path of the file execution is paused in. */
  path: string;
  /** 0-based line in that file. */
  line: number;
  /** Who published it — the debugging pane's own path — so only it clears it. */
  owner: string;
}

</storycode:copy>
<a id="f-f89ec0ad0c79-current"></a>
### `current`

<storycode:copy id="f-f89ec0ad0c79-current">
let current: PausedElsewhere | null = null;
</storycode:copy>
<a id="f-f89ec0ad0c79-listeners"></a>
### `listeners`

<storycode:copy id="f-f89ec0ad0c79-listeners">
const listeners = new Set<() => void>();

</storycode:copy>
<a id="f-f89ec0ad0c79-emit"></a>
### `emit`

<storycode:copy id="f-f89ec0ad0c79-emit">
function emit() {
  for (const listener of listeners) listener();
}

</storycode:copy>
<a id="f-f89ec0ad0c79-publishPausedElsewhere"></a>
### `publishPausedElsewhere`

Publish where `owner`'s session is paused, or clear it with `null`.

<storycode:copy id="f-f89ec0ad0c79-publishPausedElsewhere">
/** Publish where `owner`'s session is paused, or clear it with `null`. */
export function publishPausedElsewhere(
  owner: string,
  at: { path: string; line: number } | null,
): void {
  if (at === null) {
    if (current?.owner !== owner) return;
    current = null;
  } else {
    if (current && current.owner === owner && current.path === at.path && current.line === at.line) {
      return;
    }
    current = { ...at, owner };
  }
  emit();
}

</storycode:copy>
<a id="f-f89ec0ad0c79-pausedElsewhere"></a>
### `pausedElsewhere`

<storycode:copy id="f-f89ec0ad0c79-pausedElsewhere">
export function pausedElsewhere(): PausedElsewhere | null {
  return current;
}

</storycode:copy>
<a id="f-f89ec0ad0c79-onPausedElsewhere"></a>
### `onPausedElsewhere`

<storycode:copy id="f-f89ec0ad0c79-onPausedElsewhere">
export function onPausedElsewhere(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

</storycode:copy>
<a id="f-f89ec0ad0c79-usePausedElsewhere"></a>
### `usePausedElsewhere`

<storycode:copy id="f-f89ec0ad0c79-usePausedElsewhere">
/** The current position, as React state. */
export function usePausedElsewhere(): PausedElsewhere | null {
  return useSyncExternalStore(onPausedElsewhere, pausedElsewhere, pausedElsewhere);
}

</storycode:copy>
<a id="f-f89ec0ad0c79-resetPausedElsewhere"></a>
### `resetPausedElsewhere`

<storycode:copy id="f-f89ec0ad0c79-resetPausedElsewhere">
/** Test seam. */
export function resetPausedElsewhere(): void {
  current = null;
  emit();
}
</storycode:copy>

### Assemble `apps/web/src/lib/pausedElsewhere.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/pausedElsewhere.ts"><storycode:paste select="#f-f89ec0ad0c79-PausedElsewhere"/><storycode:paste select="#f-f89ec0ad0c79-current"/><storycode:paste select="#f-f89ec0ad0c79-listeners"/><storycode:paste select="#f-f89ec0ad0c79-emit"/><storycode:paste select="#f-f89ec0ad0c79-publishPausedElsewhere"/><storycode:paste select="#f-f89ec0ad0c79-pausedElsewhere"/><storycode:paste select="#f-f89ec0ad0c79-onPausedElsewhere"/><storycode:paste select="#f-f89ec0ad0c79-usePausedElsewhere"/><storycode:paste select="#f-f89ec0ad0c79-resetPausedElsewhere"/></storycode:file>

</storycode:doc>
