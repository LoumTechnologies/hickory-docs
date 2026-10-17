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

export function waitForWorkspace(): Promise<void> {
  if (pending) return pending;
  publish("Preparing generated files… You can keep writing your note.");
  pending = (async () => {
    for (;;) {
      const response = await fetch("/api/startup");
      const state = await response.json() as { ready?: boolean; error?: string };
      if (!response.ok) throw new Error(state.error ?? "Could not prepare the workspace. Reopen the folder.");
      if (typeof state.ready !== "boolean") throw new Error("The workspace returned an invalid startup status. Reopen the folder.");
      if (state.ready === true) return;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  })().then(() => { publish(null); }, (error: unknown) => {
    publish(error instanceof Error ? error.message : "Could not prepare the workspace. Reopen the folder.");
    throw error;
  }).finally(() => { pending = null; });
  return pending;
}
