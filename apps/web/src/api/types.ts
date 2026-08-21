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
/** A plain file — anything in the folder that is neither a document nor a
 * woven output: read whole, saved whole. `hash` fingerprints the content the
 * read served; the save passes it back as `base_hash` so a file rewritten on
 * disk underneath the buffer is a 409, never a silent overwrite. */
export interface PlainFile {
  path: string;
  language: string;
  content: string;
  hash: string;
}

/** What `PUT /api/file` answers: the saved content's fresh hash, which
 * becomes the next save's `base_hash`. */
export interface PlainFileSaved {
  path: string;
  hash: string;
}

/** What `POST /api/adopt` answers: the document that now owns the file. */
/** Where a scratchpad note landed, relative to the open folder. */
export interface ScratchpadSaved {
  path: string;
}

export interface AdoptResponse {
  doc_id: string;
  doc_path: string;
  file_path: string;
  /** How the weave names the output: relative to the DOCUMENT's directory
   * (the `<hick:file path>` value), which is the key generated panes fetch
   * by — not the root-relative tree path. */
  output_path: string;
  /** True when a new document was created beside the file; false when a
   * block was appended to an existing one. */
  created: boolean;
}

/** One output that no longer matches the pinned refactor baseline. */
export interface RefactorDiff {
  path: string;
  kind: "changed" | "added" | "removed";
}

/** The refactor baseline's live verdict — `active: false` when none is
 * pinned for the document. Weave-only on the server: checking never
 * executes a cell. */
export type RefactorStatus =
  | { active: false }
  | { active: true; started_at: string; clean: boolean; diffs: RefactorDiff[] };

export interface FileNode {
  name: string;
  path: string;
  dir: boolean;
  children?: FileNode[];
  doc_id?: string;
  /** The id of the document that generates this file, when one does. Absent
   * on documents themselves, on directories, and on files nobody writes. */
  generated_by?: string;
}

export interface FilesResponse {
  /** The open folder, as the server names it (absolute or display path). */
  root: string;
  /** The open folder's absolute path on this machine, in the platform's own
   * spelling — what "copy absolute path" copies. Absent from older answers
   * and from the mock, so every reader must cope without it. */
  root_path?: string;
  /** The separator that joins `root_path` to a node's (always forward-slashed)
   * path: `"/"` everywhere but Windows. */
  separator?: string;
  /** What this desktop calls its file manager — "Finder", "File Explorer",
   * or the generic "file manager" — so a menu item can say the real name. */
  file_manager?: string;
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

// --- terminals ---------------------------------------------------------------
// /api/terminals. A terminal here is a SESSION: named work, in a directory,
// on a branch, that knows whether it is busy, blocked, or done — and keeps
// knowing while its pane is closed. See crates/hick-term.

/** The five states. Working and idle make no claim on your attention. */
export type SessionState = "needs-you" | "working" | "idle" | "finished" | "failed";

/** Where a prompt came from, which decides how far it may be trusted:
 * "declared" is structural (the program said so, with its choices),
 * "guessed" is us recognising the shape of a question on a screen. */
export type PromptSource = "declared" | "guessed";

export interface PromptChoice {
  label: string;
  /** Exactly what gets written to the terminal when it is pressed. */
  send: string;
  destructive: boolean;
}

export interface TerminalPrompt {
  question: string;
  /** Empty for a guessed prompt: we can see that something is being asked,
   * not what the answers are. The card then offers a plain input line. */
  choices: PromptChoice[];
  source: PromptSource;
}

export interface TerminalSession {
  id: string;
  title: string;
  cwd: string;
  /** A support process — dev server, watcher, log tail. Lives in the dock,
   * never claims attention, never takes focus. */
  monitor: boolean;
  state: SessionState;
  /** When it entered this state, epoch millis. Oldest waiter leads its band. */
  since_ms: number;
  branch: string | null;
  dirty: boolean;
  /** The last line it printed — what a folded row shows. */
  preview: string;
  /** Whether `cwd` came from the shell (OSC 7) or is where the session was
   * started. The two are not equally trustworthy: a started-in directory is
   * stale the moment somebody `cd`s. */
  cwd_is_live?: boolean;
  prompt: TerminalPrompt | null;
  exit_code: number | null;
}

export interface TerminalsResponse {
  sessions: TerminalSession[];
  /** Session ids claiming attention, most-claiming first. The server owns
   * this order (hick_term::attention) so every surface agrees on it. */
  attention: string[];
  turbo: boolean;
}

/** POST /api/terminals. Everything is optional: no body at all opens a shell
 * in the open folder. */
export interface OpenTerminal {
  title?: string;
  /** Relative to the open folder, or absolute. */
  cwd?: string;
  argv?: string[];
  monitor?: boolean;
  /** Run in a fresh git worktree on this new branch instead of the open
   * folder, so two agents cannot fight over one checkout. */
  worktree_branch?: string;
}

// --- Workspace state and drafts (api.md, "Workspace state and drafts") ----

/** One buffer's unsaved contents, and what it was unsaved *from*. */
export interface WorkspaceDraft {
  /** Project-relative path of the file being edited. */
  path: string;
  /** The buffer as the reader left it. */
  contents: string;
  /** The file's contents when this editing session began — the common
   * ancestor a three-way merge needs. Empty when the buffer had no file. */
  base: string;
  /** Milliseconds since the epoch. Advisory: it orders drafts for display. */
  saved_at: number;
}

/** The stored window layout. Opaque on the wire; `lib/uiState.ts` owns its
 * shape, and is the only thing that should ever narrow this type. */
export type WorkspaceUiState = unknown;

// --- Exhaustive find and replace (api.md, "Find and replace") -------------
//
// Deliberately separate from SearchResponse, which is RANKED. A ranked answer
// is a sample, and replacing across a sample changes some of the occurrences.

export interface FindMatch {
  /** 1-based. */
  line: number;
  /** The whole line, so a row shows context without a second request. */
  text: string;
  /** Every match on this line, as a byte column and a length. */
  at: { column: number; length: number }[];
}

export interface FindFile {
  path: string;
  matches: FindMatch[];
  /** The document that writes this file, when one does. Replace refuses it,
   * and the UI greys it out before the button is pressed. */
  generated_by?: string;
}

export interface FindResponse {
  files: FindFile[];
  /** The match cap was hit; there are more than these. */
  truncated: boolean;
}

export interface FindOptions {
  regex?: boolean;
  case?: boolean;
  whole_word?: boolean;
}

export interface ReplaceResponse {
  changed: { path: string; matches: number }[];
  skipped: { path: string; matches: number; reason: string; document?: string }[];
  replacements: number;
}
