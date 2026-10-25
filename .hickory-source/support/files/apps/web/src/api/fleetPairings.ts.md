<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/fleetPairings.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ec3c11ea1d9a571d1df285eca7d1b4fb019cc5622d301b681119772f4ba3d3fb`.

<book:entry path="apps/web/src/api/fleetPairings.ts">
import type { FleetMachine } from "./types";
export type FleetPairingJob = {
  id: string; request_id: string; mode: "host" | "dial";
  status: "queued" | "running" | "paired" | "cancelled" | "failed";
  phase: string; cancel_requested: boolean; rollback: false;
  result?: { machine: FleetMachine; their_fingerprint: string; our_fingerprint: string; note: string };
  error?: string;
};
export const pairingPending = (job: FleetPairingJob) => job.status === "queued" || job.status === "running";
</book:entry>

</book:doc>
