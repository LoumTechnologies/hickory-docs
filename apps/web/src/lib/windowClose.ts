import type { UiOutcome } from "./uiActions";

export const WORKSPACE_CLOSE = "hickory-workspace-close-request";
export interface WorkspaceCloseRequest {
  signal?: AbortSignal;
  accept(): void;
  respond(outcome: UiOutcome): void;
}

/** An actual mounted workspace accepts responsibility synchronously, then replies. */
export function requestWorkspaceClose(signal?: AbortSignal, target: Window = window): Promise<UiOutcome> {
  return new Promise(resolve => {
    let accepted = false;
    target.dispatchEvent(new CustomEvent<WorkspaceCloseRequest>(WORKSPACE_CLOSE, {
      detail: { signal, accept: () => { accepted = true; }, respond: resolve },
    }));
    if (!accepted) resolve({ status: "unavailable", reason: "The workspace close owner is not mounted. No fallback host close was submitted; reopen the workspace and retry." });
  });
}

/** Navigation reveals existing guarded prompts; it never bypasses their owner. */
export async function requestApplicationClose(options: {
  workspace: string | null; inWorkspace: boolean; signal?: AbortSignal;
  reveal(path: string): void; workspaceClose(signal?: AbortSignal): Promise<UiOutcome>;
  hostClose(): Promise<{ ok: true }>;
}): Promise<UiOutcome> {
  if (options.signal?.aborted) return { status: "completed", value: { cancelled: true, host_close_submitted: false } };
  if (options.workspace !== null) {
    try {
      if (!options.inWorkspace) options.reveal(options.workspace);
      return await options.workspaceClose(options.signal);
    } catch (error) {
      return { status: "unavailable", reason: `Workspace close failed without fallback: ${error instanceof Error ? error.message : String(error)}` };
    }
  }
  try {
    const reply = await options.hostClose();
    if (reply?.ok !== true) throw Error("Invalid host close acknowledgement");
    return { status: "completed", value: { host_close_request_acknowledged: true, window_destruction_observed: false,
      owner_cancellation_requested: !!options.signal?.aborted, scope: "Acknowledged callback only; window shutdown can prevent response delivery. A lost response does not establish closure or rollback." } };
  } catch (error) {
    return { status: "unavailable", reason: `Host close success is unacknowledged: ${error instanceof Error ? error.message : String(error)}. The window may already be closing.` };
  }
}
