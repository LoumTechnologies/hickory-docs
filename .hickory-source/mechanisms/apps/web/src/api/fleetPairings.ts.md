<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/fleetPairings.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-530d78e6d9aa"></a>
## `apps/web/src/api/fleetPairings.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-530d78e6d9aa-FleetPairingJob"></a>
### `FleetPairingJob`

<storycode:copy id="f-530d78e6d9aa-FleetPairingJob">
import type { FleetMachine } from "./types";
export type FleetPairingJob = {
  id: string; request_id: string; mode: "host" | "dial";
  status: "queued" | "running" | "paired" | "cancelled" | "failed";
  phase: string; cancel_requested: boolean; rollback: false;
  result?: { machine: FleetMachine; their_fingerprint: string; our_fingerprint: string; note: string };
  error?: string;
};
</storycode:copy>
<a id="f-530d78e6d9aa-pairingPending"></a>
### `pairingPending`

<storycode:copy id="f-530d78e6d9aa-pairingPending">
export const pairingPending = (job: FleetPairingJob) => job.status === "queued" || job.status === "running";
</storycode:copy>

### Assemble `apps/web/src/api/fleetPairings.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/fleetPairings.ts"><storycode:paste select="#f-530d78e6d9aa-FleetPairingJob"/><storycode:paste select="#f-530d78e6d9aa-pairingPending"/></storycode:file>

</storycode:doc>
