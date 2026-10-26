<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/workspaceStartup.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-073d5b2b2a12"></a>
## `apps/web/src/api/workspaceStartup.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Editor Does Not Wait For Workspace Preparation](../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md).

<a id="f-073d5b2b2a12-pending"></a>
### `pending`

One wait shared by requests refused BEFORE dispatch during initial weaving.
Never retry a request that might already have changed a file.

<storycode:copy id="f-073d5b2b2a12-pending">
// One wait shared by requests refused BEFORE dispatch during initial weaving.
// Never retry a request that might already have changed a file.
let pending: Promise<void> | null = null;
</storycode:copy>
<a id="f-073d5b2b2a12-notice"></a>
### `notice`

<storycode:copy id="f-073d5b2b2a12-notice">
let notice: string | null = null;
</storycode:copy>
<a id="f-073d5b2b2a12-listeners"></a>
### `listeners`

<storycode:copy id="f-073d5b2b2a12-listeners">
const listeners = new Set<() => void>();

</storycode:copy>
<a id="f-073d5b2b2a12-startupNotice"></a>
### `startupNotice`

<storycode:copy id="f-073d5b2b2a12-startupNotice">
export function startupNotice(): string | null { return notice; }
</storycode:copy>
<a id="f-073d5b2b2a12-subscribeStartup"></a>
### `subscribeStartup`

<storycode:copy id="f-073d5b2b2a12-subscribeStartup">
export function subscribeStartup(listener: () => void): () => void {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}
</storycode:copy>
<a id="f-073d5b2b2a12-publish"></a>
### `publish`

<storycode:copy id="f-073d5b2b2a12-publish">
function publish(message: string | null) {
  notice = message;
  listeners.forEach((listener) => listener());
}

</storycode:copy>
<a id="f-073d5b2b2a12-STATUS_TIMEOUT_MS"></a>
### `STATUS_TIMEOUT_MS`

Only the read-only startup probe is retried on transport failure. A hung
response from an earlier engine generation must not hold the shared wait.

<storycode:copy id="f-073d5b2b2a12-STATUS_TIMEOUT_MS">
// Only the read-only startup probe is retried on transport failure. A hung
// response from an earlier engine generation must not hold the shared wait.
const STATUS_TIMEOUT_MS = 5000;
</storycode:copy>
<a id="f-073d5b2b2a12-MAX_TRANSPORT_FAILURES"></a>
### `MAX_TRANSPORT_FAILURES`

<storycode:copy id="f-073d5b2b2a12-MAX_TRANSPORT_FAILURES">
const MAX_TRANSPORT_FAILURES = 3;
</storycode:copy>
<a id="f-073d5b2b2a12-StartupTimeout"></a>
### `StartupTimeout`

<storycode:copy id="f-073d5b2b2a12-StartupTimeout">
class StartupTimeout extends Error {}
</storycode:copy>
<a id="f-073d5b2b2a12-StartupState"></a>
### `StartupState`

<storycode:copy id="f-073d5b2b2a12-StartupState">
interface StartupState {
  ready?: boolean; error?: string; phase?: string; completed?: number; total?: number;
  document?: string | null; elapsed_ms?: number; stage_elapsed_ms?: number;
}
</storycode:copy>
<a id="f-073d5b2b2a12-progressNotice"></a>
### `progressNotice`

<storycode:copy id="f-073d5b2b2a12-progressNotice">
function progressNotice(state: StartupState): string {
  const seconds = Math.floor((state.elapsed_ms ?? 0) / 1000);
  const stageSeconds = Math.floor((state.stage_elapsed_ms ?? 0) / 1000);
  const phases: Record<string, string> = { queued: "Waiting for workspace preparation", watching: "Registering file watcher", baselines: "Establishing output baselines", weaving: "Preparing generated files", reconciling: "Reconciling document rooms" };
  const progress = Number.isInteger(state.total) && Number.isInteger(state.completed)
    ? ` ${state.completed} of ${state.total} documents finished.` : "";
  const document = state.document ? ` Reading ${state.document}.` : "";
  const stalled = stageSeconds >= 15 ? ` This stage has taken ${stageSeconds}s; preparation is still pending, not complete.` : "";
  return `${phases[state.phase ?? ""] ?? "Preparing generated files"}.${progress}${document} Elapsed ${seconds}s.${stalled} You can keep writing your note.`;
}
</storycode:copy>
<a id="f-073d5b2b2a12-readStartupStatus"></a>
### `readStartupStatus`

<storycode:copy id="f-073d5b2b2a12-readStartupStatus">
async function readStartupStatus() {
  const controller = new AbortController();
  let timer: ReturnType<typeof setTimeout>;
  const timeout = new Promise<never>((_resolve, reject) => {
    timer = setTimeout(() => {
      reject(new StartupTimeout("Workspace status timed out"));
      controller.abort();
    }, STATUS_TIMEOUT_MS);
  });
  try {
    return await Promise.race([
      (async () => {
        const response = await fetch("/api/startup", { signal: controller.signal });
        const state = await response.json() as StartupState;
        return { response, state };
      })(),
      timeout,
    ]);
  } finally {
    clearTimeout(timer!);
  }
}

</storycode:copy>
<a id="f-073d5b2b2a12-waitForWorkspace"></a>
### `waitForWorkspace`

<storycode:copy id="f-073d5b2b2a12-waitForWorkspace">
export function waitForWorkspace(): Promise<void> {
  if (pending) return pending;
  publish("Preparing generated files… You can keep writing your note.");
  pending = (async () => {
    let failures = 0;
    for (;;) {
      let status: Awaited<ReturnType<typeof readStartupStatus>>;
      try {
        status = await readStartupStatus();
        failures = 0;
      } catch (error) {
        const transport = error instanceof StartupTimeout || error instanceof TypeError
          || (error instanceof DOMException && error.name === "AbortError");
        if (!transport) throw error;
        if (++failures >= MAX_TRANSPORT_FAILURES) {
          throw new Error("Cannot reach workspace preparation status. Reopen the folder to reconnect.");
        }
        await new Promise((resolve) => setTimeout(resolve, 100));
        continue;
      }
      const { response, state } = status;
      if (!response.ok) throw new Error(state.error ?? "Could not prepare the workspace. Reopen the folder.");
      if (typeof state.ready !== "boolean") throw new Error("The workspace returned an invalid startup status. Reopen the folder.");
      if (state.ready === true) return;
      publish(progressNotice(state));
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  })().then(() => { publish(null); }, (error: unknown) => {
    publish(error instanceof Error ? error.message : "Could not prepare the workspace. Reopen the folder.");
    throw error;
  }).finally(() => { pending = null; });
  return pending;
}
</storycode:copy>

### Assemble `apps/web/src/api/workspaceStartup.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/workspaceStartup.ts"><storycode:paste select="#f-073d5b2b2a12-pending"/><storycode:paste select="#f-073d5b2b2a12-notice"/><storycode:paste select="#f-073d5b2b2a12-listeners"/><storycode:paste select="#f-073d5b2b2a12-startupNotice"/><storycode:paste select="#f-073d5b2b2a12-subscribeStartup"/><storycode:paste select="#f-073d5b2b2a12-publish"/><storycode:paste select="#f-073d5b2b2a12-STATUS_TIMEOUT_MS"/><storycode:paste select="#f-073d5b2b2a12-MAX_TRANSPORT_FAILURES"/><storycode:paste select="#f-073d5b2b2a12-StartupTimeout"/><storycode:paste select="#f-073d5b2b2a12-StartupState"/><storycode:paste select="#f-073d5b2b2a12-progressNotice"/><storycode:paste select="#f-073d5b2b2a12-readStartupStatus"/><storycode:paste select="#f-073d5b2b2a12-waitForWorkspace"/></storycode:file>

</storycode:doc>
