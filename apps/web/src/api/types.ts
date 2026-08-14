// Types mirroring docs/specs/freeform/api.md — the server contract. Change there first.

export interface User {
  id: string;
  email: string;
  plan?: string;
  /** Whether this address has been confirmed. */
  email_verified?: boolean;
  /**
   * Whether the deployment can send mail at all. When false there is nothing
   * a user could do about being unverified, so the UI must not prompt.
   */
  verification_required?: boolean;
}

/** One provider credential the account has stored (BYOK). The key itself is
 * never returned by the API — `last4` is all there is to show. */
export interface LlmKey {
  provider: string;
  last4: string;
  model?: string | null;
  /** Whether agent runs use this key. Exactly one key is active; with a
   * single key stored it is active without anyone choosing. */
  active: boolean;
  created_at: string;
  last_used_at?: string | null;
}

export interface LlmKeysResponse {
  keys: LlmKey[];
  /** False when the deployment has no KEY_ENCRYPTION_KEY: nothing the user
   * can fix, so the UI explains rather than offering a form that must fail. */
  storage_available: boolean;
  /** `byo_key` — this plan runs the agent on the account's own key.
   *  `metered_allowance` — the plan includes agent spend. */
  plan_agent: string;
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
// the pricing page)" without pinning the response shape; this is the shape the
// web app expects — a faithful projection of the repo's plans.json (plans in
// plan-set order, base prices per interval, features derived server-side from
// entitlements).
export interface PlanPrice {
  key: string;
  interval: "month" | "year";
  amount_cents: number;
  currency: string;
  per_seat?: boolean;
}

export interface Plan {
  key: string;
  name: string;
  description: string;
  trial_days?: number;
  highlight?: boolean;
  /** Active base (non-per-seat) prices; empty for the free tier. */
  prices: PlanPrice[];
  /** Human-readable feature bullets derived from entitlements. */
  features: string[];
}

export interface PlansResponse {
  plans: Plan[];
  enterprise?: { contact: boolean; description: string };
}

// Generated outputs & lineage (v0.2). All ranges/spans are BYTE offsets
// (UTF-8): `start`/`end` into the output file's content, `span` into the
// source document.

export type ProvenanceOrigin =
  | {
      kind: "literal" | "paste" | "exec" | "variable" | "substitution";
      doc_path: string;
      span: [number, number];
    }
  | { kind: "synthetic" };

export interface Provenance {
  start: number;
  end: number;
  origin: ProvenanceOrigin;
}

export interface OutputFileMeta {
  path: string;
  language: string;
}

export interface OutputsResponse {
  files: OutputFileMeta[];
}

export interface OutputFile {
  path: string;
  language: string;
  content: string;
  provenance: Provenance[];
}

export interface OutputEdit {
  start: number;
  end: number;
  text: string;
}

export interface SourceEdit {
  doc_path: string;
  span: [number, number];
  text: string;
}

export interface OutputEditResponse {
  source_edits: SourceEdit[];
  applied: true;
}

/** Body of the 422 returned when an edit overlaps a synthetic range. */
export interface SyntheticRangeError {
  error: string;
  range: { start: number; end: number };
}

export interface Health {
  ok: boolean;
  executor: "local" | "canopy";
  db: boolean;
}

/** GET /api/executor — where cells run (environment cards). `images` is the
 * image ref → Nix store path map on canopy; null on local, where the image
 * attribute is recorded provenance, not an enforced sandbox. */
export interface ExecutorInfo {
  kind: "local" | "canopy";
  images: Record<string, string> | null;
}

/** One exchange in a document's agent conversation.
 *
 * The conversation is a TREE: `parent_id` is the turn this one continues from,
 * so rewinding and sending again forks a branch instead of destroying history. */
export interface AgentTurn {
  id: string;
  parent_id: string | null;
  prompt: string;
  answer: string | null;
  status: "running" | "ok" | "error" | string;
  error: string | null;
  created_at: string;
}

// --- structural navigation ---------------------------------------------------
// Definitions and references found by tree-sitter (crates/hick-structure).
// Resolution is by NAME, so `candidates` says how many definitions a link
// could have meant; the view draws an ambiguous link differently rather than
// implying certainty.

export interface StructureDefinition {
  name: string;
  kind: string;
  start_line: number;
  end_line: number;
}

export interface StructureReference {
  name: string;
  line: number;
}

export interface FileStructure {
  path: string;
  language: string;
  definitions: StructureDefinition[];
  references: StructureReference[];
}

export interface StructuralLink {
  from_path: string;
  from_line: number;
  to_path: string;
  to_line: number;
  name: string;
  candidates: number;
}

export interface StructureResponse {
  files: FileStructure[];
  links: StructuralLink[];
}
