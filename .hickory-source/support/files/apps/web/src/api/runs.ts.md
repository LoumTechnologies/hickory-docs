<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/runs.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `02dc21d8bbeef7dbd2d960926a064ac67f745f7309bedc21d13ea0ef790fd2ef`.

<book:entry path="apps/web/src/api/runs.ts">
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
</book:entry>

</book:doc>
