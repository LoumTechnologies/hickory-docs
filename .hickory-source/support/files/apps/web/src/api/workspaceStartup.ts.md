<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/workspaceStartup.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Editor Does Not Wait For Workspace Preparation](../../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md)

Captured SHA-256: `7729938315ef6afb11feb680358ab024c0e48b56bda088234750c2b9690254e6`.

<book:entry path="apps/web/src/api/workspaceStartup.ts">
// One wait shared by requests refused BEFORE dispatch during initial weaving.
// Never retry a request that might already have changed a file.
let pending: Promise<void> | null = null;
let notice: string | null = null;
const listeners = new Set<() => void>();

export function startupNotice(): string | null { return notice; }
export function subscribeStartup(listener: () => void): () => void {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}
function publish(message: string | null) {
  notice = message;
  listeners.forEach((listener) => listener());
}

// Only the read-only startup probe is retried on transport failure. A hung
// response from an earlier engine generation must not hold the shared wait.
const STATUS_TIMEOUT_MS = 5000;
const MAX_TRANSPORT_FAILURES = 3;
class StartupTimeout extends Error {}
interface StartupState {
  ready?: boolean; error?: string; phase?: string; completed?: number; total?: number;
  document?: string | null; elapsed_ms?: number; stage_elapsed_ms?: number;
}
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
</book:entry>

</book:doc>
