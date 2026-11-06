export interface CheckRun {
  id: string;
  parent: string | null;
  name: string;
  status: string;
  revision: string | null;
  created_ms: number;
  code: number | null;
}
export interface ChecksSnapshot {
  availability: "ready" | "disabled" | "unavailable";
  message: string | null;
  runs: CheckRun[];
}
