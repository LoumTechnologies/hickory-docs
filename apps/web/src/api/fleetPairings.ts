import type { FleetMachine } from "./types";
export type FleetPairingJob = {
  id: string; request_id: string; mode: "host" | "dial";
  status: "queued" | "running" | "paired" | "cancelled" | "failed";
  phase: string; cancel_requested: boolean; rollback: false;
  result?: { machine: FleetMachine; their_fingerprint: string; our_fingerprint: string; note: string };
  error?: string;
};
export const pairingPending = (job: FleetPairingJob) => job.status === "queued" || job.status === "running";
