// Types mirroring docs/specs/freeform/api.md — the server contract. Change there first.

export interface User {
  id: string;
  email: string;
  plan?: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface Project {
  id: string;
  name: string;
  visibility: "public" | "private";
  created_at: string;
}

export interface DocSummary {
  id: string;
  path: string;
  updated_at: string;
}

export interface Doc {
  id: string;
  path: string;
  source: string;
  updated_at: string;
}

export type TranscriptEvent =
  | { t: number; kind: "cmd"; data: string }
  | { t: number; kind: "out" | "err"; data: string }
  | { t: number; kind: "exit"; code: number };

export type ExecStatus = "ok" | "failed" | "stale" | "never-run";

export type Block =
  | { kind: "prose"; html: string; span: [number, number] }
  | {
      kind: "exec";
      id: string;
      container: string;
      image?: string;
      command: string;
      span: [number, number];
      transcript?: TranscriptEvent[];
      expect?: { match: "exact" | "regex-lines"; body: string };
      status?: ExecStatus;
    }
  | { kind: "file"; path: string; language: string; body: string; span: [number, number] }
  | {
      kind: "session-user" | "session-assistant" | "session-observation";
      body: string;
      span: [number, number];
    };

export type ExecBlock = Extract<Block, { kind: "exec" }>;

export interface RenderResponse {
  blocks: Block[];
}

export type RunStatus = "queued" | "running" | "ok" | "failed";

export interface Run {
  id: string;
  status: RunStatus;
  started_at: string;
  blocks: { exec_id: string; status: ExecStatus; transcript: TranscriptEvent[] }[];
}

// WS channel 0x01 payloads.
export type RunWsMessage =
  | { run_id: string; exec_id: string; event: TranscriptEvent }
  | { run_id: string; status: RunStatus };

// Billing. The contract says "the active plan set from plans.json (shaped for
// the pricing page)" without pinning the shape; this is the shape the web app
// expects the server to emit.
export interface Plan {
  key: string;
  name: string;
  description: string;
  price_key: string;
  amount_cents: number;
  currency: string;
  interval: "month" | "year";
  features: string[];
  highlight?: boolean;
}

export interface PlansResponse {
  plans: Plan[];
}

export interface Health {
  ok: boolean;
  executor: "local" | "canopy";
  db: boolean;
}
