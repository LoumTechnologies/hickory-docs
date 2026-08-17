// Types mirroring docs/specs/freeform/api.md — the server contract. Change there first.

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

/** Agent-loop events share the run channel (`exec_id: "agent"`, `run_id` =
 * session id). Serialized from `hickory_agent::AgentEvent`; only the fields
 * the dock renders are typed — the rest are kind-only so `kind` narrowing
 * stays exhaustive against `TranscriptEvent` (no member here carries `t`). */
export type AgentWsEvent =
  | { kind: "token"; data: string }
  | { kind: "script_started"; lang: string; data: string }
  | {
      kind: "script_finished";
      result: { exit_code: number | null; stdout: string; stderr: string };
    }
  | { kind: "tool_started"; name: string; data: string }
  | { kind: "tool_finished"; name: string; ok: boolean; text: string }
  | { kind: "error"; message: string }
  | {
      kind:
        | "session_started"
        | "user_message"
        | "thinking"
        | "response_complete"
        | "reprompt"
        | "turn_usage"
        | "done";
    };

// WS channel 0x01 payloads.
export type RunWsMessage =
  | { run_id: string; exec_id: string; event: TranscriptEvent | AgentWsEvent }
  | { run_id: string; status: RunStatus }
  // The in-app up-loop re-wove this document and its output files changed on
  // disk (crates/hickory-cli/src/serve/watch.rs::notify_files_changed).
  | { files_changed: true; doc: string };

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

/** One entry of the open folder's file tree (GET /api/files). Directories
 * come first, each level alphabetical; `path` is root-relative; `doc_id` is
 * present only on `.hick` documents. */
export interface FileNode {
  name: string;
  path: string;
  dir: boolean;
  children?: FileNode[];
  doc_id?: string;
}

export interface FilesResponse {
  /** The open folder, as the server names it (absolute or display path). */
  root: string;
  tree: FileNode[];
  /** Set when the walk stopped early (a huge folder); the tree is a prefix. */
  truncated?: boolean;
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
/** Four-way token usage, split the way providers bill it. */
export interface AgentUsage {
  input_tokens: number;
  cache_creation_input_tokens: number;
  cache_read_input_tokens: number;
  output_tokens: number;
}

export interface AgentTurn {
  id: string;
  parent_id: string | null;
  prompt: string;
  answer: string | null;
  status: "running" | "ok" | "error" | string;
  error: string | null;
  created_at: string;
  /** Provider selector this turn ran on ("anthropic", "openai", …). */
  provider: string;
  /** Model id this turn ran on — recorded per turn, so a mid-conversation
   * model change stays visible. */
  model: string;
  /** The turn's final token usage; null while running or after a failure. */
  usage: AgentUsage | null;
}

/** Session-wide spend across a document's turns, each turn priced on the
 * model it ran on. `usd` is null when any turn's model has no known price. */
export interface AgentTotals {
  usd: number | null;
  input: number;
  output: number;
  cache_read: number;
  cache_write: number;
}

export interface AgentTurnsResponse {
  turns: AgentTurn[];
  /** The provider the next turn would run on, defaults resolved. */
  provider: string;
  /** The model the next turn would run on, defaults resolved. */
  model: string;
  totals: AgentTotals;
}

// --- project search ----------------------------------------------------------
// GET /api/search — ranked hits across the served folder: .hick documents and
// generated files alike. Lines are 1-based; `path` is relative to the folder.

export interface SearchHit {
  path: string;
  start_line: number;
  end_line: number;
  score: number;
  snippet: string;
}

export interface SearchResponse {
  /** False when only lexical ranking is available — the semantic model is an
   * optional download (`hick search --install-model`), never a requirement. */
  semantic: boolean;
  hits: SearchHit[];
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

// --- settings: LLM API keys --------------------------------------------------
// GET/PUT /api/settings/keys. The server NEVER returns a full key — only
// whether one is configured, and a masked hint for telling keys apart.

export type ProviderId = "anthropic" | "openai" | "openrouter" | "deepseek" | "xai";

export interface ProviderKey {
  id: ProviderId;
  label: string;
  configured: boolean;
  /** e.g. "sk-a…f3" when configured; null otherwise. Never the full key. */
  masked: string | null;
}

export interface SettingsKeysResponse {
  providers: ProviderKey[];
}

/** PUT body: only the providers being changed — a string sets, null clears.
 * Untouched providers are simply absent. */
export type SettingsKeysPatch = Partial<Record<ProviderId, string | null>>;

// --- settings: UI ------------------------------------------------------------
// GET/PUT /api/settings/ui. Persisted server-side (ui.json beside
// llm-keys.json) so the desktop shell can read the custom window title at
// launch, before any page has loaded.

export interface UiSettings {
  /** Custom window title, or null for the default (folder / file name). */
  window_title: string | null;
}
