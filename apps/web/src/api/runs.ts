import type { Block } from "./types";

export type RunStatus = "queued" | "running" | "ok" | "failed" | "stopped";

export interface Run {
  id: string;
  status: RunStatus;
  started_at: string;
  /** Execution/setup failure, including nonzero exits and timeouts. */
  error?: string | null;
  /**
   * The run's own block model — the same shape `/render` returns, with the
   * transcripts this run produced.
   *
   * Was typed as `{exec_id, status, transcript}[]`, which the server has
   * never sent: it stores `block_model_json(&run)["blocks"]`. The mistyping
   * was invisible because nothing read the field.
   */
  blocks: Block[];
}
