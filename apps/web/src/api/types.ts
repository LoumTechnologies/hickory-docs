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

export type ExecStatus = "ok" | "failed" | "stale" | "unrecorded";

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
      /** Present when this cell owns a `<hick:ingested>` child — the same
       * fingerprint the woven markdown's "Ingested from …" caption reads,
       * surfaced here so the Document view's card can say it too instead of
       * only the raw tag sitting as unstyled text after the card (the exec
       * card's own rendering stops before ingested content on purpose — see
       * apps/web/src/editor/rendered.ts). */
      ingested?: {
        from: string;
        at: string;
        sha256: string;
        files: string;
        skipped: string;
      };
      status?: ExecStatus;
    }
  | {
      kind: "file";
      path: string;
      language: string;
      body: string;
      span: [number, number];
    }
  | {
      kind: "diagram";
      renderer: string;
      /** Body with this document's `<hick:paste>` fragments inlined — what
       * the panel draws. The raw source, paste tags and all, stays in the
       * editor buffer. */
      body: string;
      /** Ids named by `asserts`, `#` stripped. */
      asserts: string[];
      span: [number, number];
    }
  | {
      kind: "session-user" | "session-assistant" | "session-observation";
      body: string;
      span: [number, number];
    };

export type ExecBlock = Extract<Block, { kind: "exec" }>;
export type DiagramBlock = Extract<Block, { kind: "diagram" }>;

export interface RenderResponse {
  blocks: Block[];
}

export type RunStatus = "queued" | "running" | "ok" | "failed" | "stopped";

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
  | { kind: "reasoning"; data: string }
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
  // Bytes a tool outside the document wrote, ingested into it (`hick ingest
  // --from`). Present, byte-precise and therefore EDITABLE — that is the
  // point — but never `literal`, because "you wrote this" and "this arrived
  // from that run" are different claims and drawing them alike is the lie
  // this origin exists to prevent. `run` is the `<hick:ingested sha256=>`
  // fingerprint: one run, N files.
  // docs/specs/freeform/owning-what-a-scaffolder-wrote.md
  | {
      kind: "ingested";
      doc_path: string;
      span: [number, number];
      run: string;
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

/** Where a picked sample landed. */
export interface SampleCreated {
  doc_id: string;
  doc_path: string;
  /** The path as the document names it — relative to the document. */
  path: string;
  from: number;
  to: number;
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
  /** Why the disk does not hold what the document produces, when it does
   * not. See docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md */
  diverged?: string;
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

/** One produced file whose disk bytes are not what its document produces —
 * axis 3 of docs/specs/freeform/three-axes.md. */
export interface DivergedOutput {
  /** `held`: somebody wrote it and it could not be carried back. `kept`: the
   * document cannot reproduce it yet (an unrecorded cell). */
  kind: "held" | "kept";
  reason: string;
  /** The last bytes both sides agreed on. */
  base: string;
  /** What the document produces now; empty for `kept`. */
  theirs: string;
}

/** GET /api/outputs/diverged, root-relative path → the file's state. */
export interface DivergedOutputs {
  diverged: Record<string, DivergedOutput>;
}

/** One input that was in front of the model when it wrote (context provenance). */
export type ContextInput =
  | {
      kind: "file";
      path: string;
      commit: string | null;
      sha256: string;
      first_line: number;
      last_line: number;
      session_line: number;
    }
  | {
      kind: "conversation";
      element: "user" | "observation" | "tool-result" | string;
      id: string | null;
      source: string | null;
      summary: string;
      sha256: string;
      lines: number;
      session_line: number;
    };

/** A run of lines an agent wrote, with everything that preceded the write in
 * its session. `current_lines` is where those lines are NOW, or null when the
 * file has moved on. Derived from the session record, never declared. */
export interface ContextWrite {
  session: string;
  session_line: number;
  file: string;
  first_line: number;
  last_line: number;
  hashes: string[];
  inputs: ContextInput[];
  current_lines: [number, number] | null;
}

export interface ContextResponse {
  writes: ContextWrite[];
}

/** A place a declared citation comes from or points at. */
export interface CitePlace {
  path: string;
  first_line: number;
  last_line: number;
  element: string;
  id: string | null;
}

/** One `cites="…"`: the author's assertion of what an element rests on,
 * resolved to places. Declared, forgeable, drawn as an assertion. */
export interface DeclaredCite {
  select: string;
  from: CitePlace;
  to: CitePlace[];
}

export interface CitesResponse {
  cites: DeclaredCite[];
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
  /** The session file this turn is recorded in, relative to the folder. */
  session?: string;
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

export type ProviderId =
  "anthropic" | "openai" | "openrouter" | "deepseek" | "xai" | "gab";

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
  /** Run the file's formatter when Save is chosen. */
  format_on_save: boolean;
}

// --- terminals ---------------------------------------------------------------
// /api/terminals. A terminal here is a SESSION: named work, in a directory,
// on a branch, that knows whether it is busy, blocked, or done — and keeps
// knowing while its pane is closed. See crates/hick-term.

/** The five states. Working and idle make no claim on your attention. */
export type SessionState =
  "needs-you" | "working" | "idle" | "finished" | "failed";

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

/**
 * A terminal that is writing into a document.
 *
 * The persistent binding from
 * `docs/specs/freeform/a-terminal-that-writes-the-document.md`. The anchor is
 * the CONTAINER, not the document: several `hick:exec` blocks naming one
 * container is already how the language says "commands that share state", so
 * a terminal bound to `sdk` in a document is that shell. Say "anchored to a
 * container", never "attached to a document".
 */
export interface TerminalAnchor {
  /** The document id the cell lives in. */
  doc: string;
  /** The container the cell names — the anchor itself. */
  container: string;
  /** Why recording stopped, when it has. Suspension is sticky: the terminal
   * keeps working and the document stops receiving, because a cell with a
   * hole in it claims a run that cannot reproduce. */
  suspended: string | null;
  /** What is reading the keys instead of the shell, while something is.
   * NOT a suspension: nothing has to be resumed, and it clears itself when
   * the shell gets the terminal back. */
  foreign: string | null;
  /** Lines written since anchoring. */
  recorded: number;
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
  skipped: {
    path: string;
    matches: number;
    reason: string;
    document?: string;
  }[];
  replacements: number;
}

/** One line's authorship, from `GET /api/blame`. */
export interface BlameLine {
  line: number;
  commit: string;
  author: string;
  email: string;
  /** Unix seconds; formatted by the client, in the reader's own locale. */
  time: number;
  summary: string;
  uncommitted: boolean;
}

/** What a table's formulas came to. Keyed by A1 label, and only for cells
 * that ARE formulas — echoing literals back would make the response the size
 * of the table for no reason. */
export interface FormulaResults {
  values: Record<string, string>;
  errors: Record<string, string>;
}

/** What one reference was worth when the cell that used it ran. `kind` is
 * carried because `empty` is not the empty string — a blank cell is skipped
 * by `sum` where `""` is not, and a debugger showing both as nothing would
 * hide the difference. */
export interface FormulaBinding {
  cell: string;
  text: string;
  kind: "number" | "text" | "bool" | "empty";
}

/** One cell's turn, as the table's debugger steps through it. */
export interface FormulaStep {
  cell: string;
  /** Which batch it went out in. Cells sharing a level cannot depend on each
   * other, so their order among themselves means nothing. */
  level: number;
  expression: string;
  bindings: FormulaBinding[];
  value: string | null;
  error: string | null;
}

/** The same evaluation `evaluateFormulas` does, with every step kept. Steps
 * are empty for a sheet with a cycle: a circle has no order, and inventing
 * one to step through would be the debugger telling its first lie. */
export interface FormulaTrace extends FormulaResults {
  steps: FormulaStep[];
}

/** One completion drawn from the project's own text. `semantic` is false when
 * the local model is not installed — the ranking is then frequency alone,
 * which is said rather than implied. */
export interface ProjectSuggestion {
  text: string;
  detail: string;
  score: number;
  semantic: boolean;
}

// --- Git (api.md, "History") ---------------------------------------------

export interface GitFileChange {
  path: string;
  /** Where it came from, on a rename. */
  from?: string;
  /** `A`, `M`, `D`, `R`. */
  status: string;
  /** Absent for a binary file, where lines are not a meaningful count —
   * which is different from zero and must not render as zero. */
  added?: number;
  removed?: number;
}

/** A commit's recipe, from its `Hick-Recipe` / `Hick-Image` / `Hick-Output`
 * trailers. A DECLARED claim in the commit's own words: nothing has checked
 * it until a replay does. docs/specs/freeform/lenses.md */
export interface GitRecipe {
  command: string;
  image?: string;
  /** The git tree hash of what the command produced, as the commit claims. */
  output?: string;
  /** Where that tree sits in the repository. */
  output_path?: string;
  /** Whether the commit's own tree at `output_path` is `output` — checked by
   * git, no replay. False means the commit was edited before it was
   * committed and cannot be upgraded by replay. Absent when the trailer
   * names no path to check. */
  output_matches?: boolean;
  /** For a replay commit: the commit it replayed. */
  replay_of?: string;
  /** For a replay commit: whether it produced the same tree the replayed
   * commit recorded. Evidence — a run happened — not a claim. */
  replay_same?: boolean;
}

/** What `POST /api/git/replay` did. */
export interface GitReplay {
  of: string;
  sha: string;
  short: string;
  same: boolean;
  /** `rebase` above the floor, `merge` below it. */
  moved: "rebase" | "merge";
  head: string;
  said: string[];
}

/** What `POST /api/git/recipe` made: a recipe commit, now HEAD. */
export interface GitRecipeRun {
  sha: string;
  short: string;
  output_tree: string;
  said: string[];
}

/** One commit as a card (`GET /api/git/commit?sha=`): its diff, its files,
 * and which of those files a later commit changed. */
export interface GitCommitDetail {
  sha: string;
  diff: string;
  files: string[];
  edited_since: { path: string; sha: string; short: string; subject: string }[];
}

export interface GitCommit {
  /** Above the publication floor: still a draft. */
  draft?: boolean;
  /** The command that produced this commit's tree, when its trailers say. */
  recipe?: GitRecipe;
  sha: string;
  short: string;
  parents: string[];
  author: string;
  email: string;
  /** Unix seconds; formatted by the client in the reader's own locale. */
  time: number;
  subject: string;
  body: string;
  /** Branch and tag names pointing here. */
  refs: string[];
  files: GitFileChange[];
  added: number;
  removed: number;
}

export interface GitLog {
  /** False for a folder that is not under version control, which is an
   * entirely normal thing for a folder of notes to be. */
  repository: boolean;
  commits: GitCommit[];
  /** Null where the repository has published nothing to compute one against.
   * See docs/specs/freeform/expression-and-log.md. */
  floor: PublicationFloor | null;
}

/** The publication floor: `merge-base(HEAD, <published ref>)`.
 *
 * Below it, commits are RECORDS — someone else may be holding them, and
 * nothing may re-produce one. Above it is the frontier, which is derived:
 * edit the document, re-emit, and those commits are replaced. Computed on
 * every read, never stored — merging moves the floor, and a recorded one
 * would be a claim the next fetch falsifies. */
export interface PublicationFloor {
  /** The ref the floor was computed against, when there is one. */
  published_ref?: string;
  /** The merge-base itself. Absent when nothing is published. */
  sha?: string;
  source: "upstream" | "remote-default" | "remote-named" | "none";
  /** Commits reachable from HEAD but not from the floor. */
  drafts: string[];
  /** One sentence a person can read. */
  summary: string;
}

/** One commit a document's time slider can stop at. */
export interface ReplayCommit {
  sha: string;
  short: string;
  time: number;
  author: string;
  subject: string;
  /** The path the document had AT this commit — `--follow` walks renames. */
  path: string;
}

/** A replay of one document at one commit.
 *
 * `grammar_boundary` is not a failure: replay weaves an old document with
 * today's binary, and this product reserves the right to change the grammar.
 * Past that point the honest claim is that replay works back to the last
 * grammar change, and `message` says exactly that. */
export type ReplayResponse =
  | {
      commit: string;
      path: string;
      grammar_boundary: true;
      message: string;
    }
  | {
      commit: string;
      path: string;
      grammar_boundary: false;
      source: string;
      outputs: string[];
      output?: { path: string; content: string; provenance: Provenance[] };
    };

/** Whether `.hick` documents merge through hick in THIS clone.
 *
 * The routing (`*.hick merge=hick`) is committed; the driver definition
 * cannot be, because git will not let a repository hand a clone an executable
 * command. An undefined driver makes git fall back to its line merge
 * silently, which is why this is checked at project open. */
export interface MergeDriverStatus {
  repository: boolean;
  attributes: boolean;
  configured: boolean;
  summary: string;
}

export interface GitStatus {
  repository: boolean;
  branch?: string;
  staged?: number;
  unstaged?: number;
  untracked?: number;
}

/** One changed file in the working tree: what the index says of it and what
 * the tree says, as git's two porcelain columns (` `, `M`, `A`, `D`, `R`,
 * `?`). A file can be on both sides at once. */
export interface GitChangeFile {
  path: string;
  from?: string;
  index: string;
  tree: string;
}

/** GET /api/git/changes. */
export interface GitChanges {
  repository: boolean;
  branch?: string;
  upstream?: string | null;
  ahead?: number;
  behind?: number;
  files: GitChangeFile[];
}

/** GET /api/git/diff — one file's diff, as git prints it. */
export interface GitDiff {
  path: string;
  diff: string;
  binary: boolean;
}

export interface GitBranch {
  name: string;
  upstream: string | null;
  current: boolean;
}

/** What a git verb answered: `ok`, and what git said on the way. */
export interface GitSaid {
  ok: boolean;
  said?: string;
  branch?: string;
}

export interface GitCommitResult {
  sha: string;
  short: string;
  subject: string;
}

/** Where an image dropped into a note was written (POST /api/asset). */
export interface SavedAsset {
  /** Root-relative — what the tree calls it, and what `GET /api/asset` takes. */
  path: string;
  /** Relative to the document that references it: what goes inside the
   * markdown parentheses, so the note survives the folder being moved. */
  relative: string;
  bytes: number;
}

// ---- a session file, read back as the conversation it records ------------

export type SessionStep =
  | { kind: "reasoning"; text: string; session_line: number }
  | { kind: "prose"; text: string; session_line: number }
  | { kind: "action"; lang: string; code: string; session_line: number }
  | {
      kind: "observation";
      id: string | null;
      source: string | null;
      exit: string | null;
      text: string;
      session_line: number;
    }
  | { kind: "tool"; name: string; args: [string, string][]; input: string | null; session_line: number }
  | { kind: "tool-result"; id: string | null; name: string; ok: boolean; text: string; session_line: number }
  | { kind: "read"; file: string; commit: string | null; sha256: string; lines: string; session_line: number }
  | { kind: "wrote"; file: string; lines: string; session_line: number }
  | { kind: "context"; context_kind: string; text: string; session_line: number };

export interface SessionTurn {
  id: string;
  parent: string | null;
  prompt: string;
  provider: string | null;
  model: string | null;
  steps: SessionStep[];
  answer: string | null;
  usage: { input: number; cache_write: number; cache_read: number; output: number; cost_usd: number | null } | null;
  session_line: number;
}

export interface SessionView {
  start: string | null;
  doc: string | null;
  turns: SessionTurn[];
}

export interface SessionViewResponse {
  path: string;
  view: SessionView;
}

// ---- the merged view -----------------------------------------------------

/** One worktree a merged view can be opened over.
 *
 * "Across branches" means across WORKTREES: a branch that is not checked out
 * cannot be written to without going behind the working tree into the object
 * database, which bypasses hooks and produces commits nobody watched.
 * docs/specs/freeform/the-merged-view.md */
export interface WorktreeInfo {
  path: string;
  name: string;
  branch?: string;
  current: boolean;
}

/** A run of lines every source agrees on, or a place where they differ.
 *
 * The sources are PEERS: shared means agreed by ALL of them, and there is no
 * order, because the view removes the question. */
export type MergedRegion =
  | { kind: "shared"; text: string }
  | { kind: "variant"; by_source: Record<string, string> };

export interface MergedViewResponse {
  repository: boolean;
  path?: string;
  sources: WorktreeInfo[];
  regions: MergedRegion[];
  /** Worktrees that do not have this file at all — named, not dropped. */
  missing: string[];
  shared_lines?: number;
  variants?: number;
  /** Always true in this step: the alignment is proved before anything is
   * written through it. */
  read_only?: boolean;
}

/** Continuity — the fourth provenance family, off by default. The whole of it
 * rides this one switch: no ribbon, no journal, no pre-commit repair. */
export interface ContinuitySettings {
  enabled: boolean;
  journal_path: string;
  entries: number;
  /** Whether the journal is tracked by git — only a committed journal is a
   * thing CI could check. */
  committed: boolean;
}

// ---- the fleet -----------------------------------------------------------

/** One machine whose key this one holds.
 *
 * A machine is a keypair; a fleet is a mutual list of public keys under the
 * user's own state directory. There is no account and no server, and
 * revocation is deleting a key.
 * docs/specs/freeform/one-engineer-many-machines.md */
export interface FleetMachine {
  name: string;
  public_key: string;
  kind: "desktop" | "phone";
  /** `view` and `edit` by default; `execute` never by default. */
  grants: ("view" | "edit" | "execute")[];
  added: string;
}

export interface FleetResponse {
  this_machine: { name: string; fingerprint: string };
  machines: FleetMachine[];
  /** How this machine would be reachable. `default` means number0's relays
   * and address publishing — not a server we run, but somebody's, so it is
   * stated rather than inherited. */
  reach: "default" | "own" | "direct" | "invalid";
  reach_note: string;
  note: string;
}

// ---- New Project ---------------------------------------------------------

/** One row of `dotnet new list`.
 * crates/hickory-cli/src/scaffold.rs */
export interface ScaffoldTemplate {
  /** Every short name the row lists; the first is what commands use. */
  short_names: string[];
  name: string;
  languages: string[];
  /** The language `dotnet` picks when none is given. `null` for a template
   * that takes no `--language` at all (`gitignore`, `editorconfig`) — which
   * is not the same as a template with exactly one. */
  default_language: string | null;
  tags: string[];
}

export interface ScaffoldCatalog {
  /** `dotnet` today. Named so a second scaffolder is a variant of this
   * screen rather than a rewrite of it. */
  kind: string;
  sdk_version: string;
  /** The SDK image matching that version, for the document's container. */
  image: string;
  templates: ScaffoldTemplate[];
}

export interface ScaffoldChoice {
  value: string;
  description: string;
}

/** One template option, as a form field. */
export interface TemplateOption {
  /** Every spelling, as help prints them (`["-au", "--auth"]`). */
  names: string[];
  /** The one a generated command uses: the longest. */
  flag: string;
  kind: "bool" | "choice" | "integer" | "float" | "text";
  choices: ScaffoldChoice[];
  /** What `dotnet` uses when the flag is absent; `null` when it has no
   * default at all, which is a different answer from the empty string. */
  default: string | null;
  description: string;
  /** The template engine's own condition for this option mattering, verbatim.
   * Shown, never evaluated — a form that greys out the wrong field is worse
   * than one that says what the condition is. */
  enabled_if: string | null;
}

export interface ScaffoldTemplateDetail {
  title: string;
  author: string;
  description: string;
  options: TemplateOption[];
  /** Languages the help footer says to ask for separately. */
  other_languages: string[];
}

/** One `--flag value` the form decided on. `value` absent is a bare switch. */
export interface ChosenOption {
  flag: string;
  value?: string;
}

/** What the dialog is asking to be scaffolded. */
export interface ScaffoldSpec {
  template: string;
  title: string;
  language: string | null;
  name: string;
  output: string;
  image: string;
  options: ChosenOption[];
}

/** The commit a New Project would make, without making it. */
export interface ScaffoldPreview {
  output: string;
  command: string;
  /** The message, trailers included; the tree hash is a placeholder. */
  message: string;
}

/** What `POST /api/scaffold` made: a commit carrying its recipe. */
export interface ScaffoldCreated {
  sha: string;
  short: string;
  /** The folder the scaffold landed in. */
  output: string;
  files: string[];
  message: string;
  /** The `Hick-Output` tree hash. */
  output_tree: string;
}
