<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/types.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8a4c3188ff6c"></a>
## `apps/web/src/api/types.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A running agent can be stopped](../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md), [Keep divergent output until an explicit decision names the write](../../../../../guarantees/authoring/an-output-that-cannot-be-carried-back-is-held/contract.md), [A Missing Merge Driver Is A Button, Not A Command To Go And Type](../../../../../guarantees/collaboration/a-missing-merge-driver-is-a-button/contract.md), [The Publication Floor Is Computed On Every Read And Shown](../../../../../guarantees/collaboration/the-publication-floor-is-computed-and-shown/contract.md), [An Ingested Cell Names Itself In The Document View](../../../../../guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view/contract.md), [Ingested Bytes Are Editable And Never Report As Yours](../../../../../guarantees/lineage/ingested-bytes-are-not-yours/contract.md).

<a id="f-8a4c3188ff6c-Project"></a>
### `Project`

Types mirroring docs/specs/freeform/api.md — the server contract. Change there first.

<storycode:copy id="f-8a4c3188ff6c-Project">
// Types mirroring docs/specs/freeform/api.md — the server contract. Change there first.

export interface Project {
  id: string;
  name: string;
  visibility: "public" | "private";
  created_at: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-DocSummary"></a>
### `DocSummary`

<storycode:copy id="f-8a4c3188ff6c-DocSummary">
export interface DocSummary {
  id: string;
  path: string;
  updated_at: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-Doc"></a>
### `Doc`

<storycode:copy id="f-8a4c3188ff6c-Doc">
export interface Doc {
  id: string;
  path: string;
  source: string;
  updated_at: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-TranscriptEvent"></a>
### `TranscriptEvent`

<storycode:copy id="f-8a4c3188ff6c-TranscriptEvent">
export type TranscriptEvent =
  | { t: number; kind: "cmd"; data: string }
  | { t: number; kind: "out" | "err"; data: string }
  | { t: number; kind: "exit"; code: number };

</storycode:copy>
<a id="f-8a4c3188ff6c-ExecStatus"></a>
### `ExecStatus`

<storycode:copy id="f-8a4c3188ff6c-ExecStatus">
export type ExecStatus = "ok" | "failed" | "stale" | "unrecorded";

</storycode:copy>
<a id="f-8a4c3188ff6c-Block"></a>
### `Block`

<storycode:copy id="f-8a4c3188ff6c-Block">
export type Block =
  | { kind: "dataset"; src: string; format: string; span: [number, number] }
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
      /** UI-only failed run attempt with no authoritative per-cell receipt. */
      runError?: string;
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
  // The session vocabulary, as `hick_literate::session_elements` renders it.
  | { kind: "session-user"; turn?: string; body: string; span: [number, number] }
  | { kind: "session-assistant"; body: string; span: [number, number] }
  | { kind: "session-tool"; name?: string; call?: string; args: [string, string][]; span: [number, number] }
  | { kind: "session-tool-result"; name?: string; ok?: boolean; body: string; span: [number, number] }
  | { kind: "session-read"; file?: string; lines?: string; sha256?: string; commit?: string; span: [number, number] }
  | { kind: "session-wrote"; file?: string; lines?: string; span: [number, number] }
  | { kind: "session-context"; context_kind?: string; body: string; span: [number, number] }
  | { kind: "session-observation"; source?: string; exit?: string; body: string; span: [number, number] }
  | { kind: "session-action"; lang?: string; body: string; span: [number, number] }
  | { kind: "session-reasoning"; body: string; span: [number, number] }
  | { kind: "session-input"; name?: string; body: string; span: [number, number] }
  /** The harness's bookkeeping (`usage`, `next`): drawn as nothing. */
  | { kind: "session-meta"; element: string; span: [number, number] };

</storycode:copy>
<a id="f-8a4c3188ff6c-export_statement"></a>
### `export_statement`

Recorded links and independently checked output evidence for the session lens.

<storycode:copy id="f-8a4c3188ff6c-export_statement">
/** Recorded links and independently checked output evidence for the session lens. */
export type { SessionLink } from "./sessionTypes";

</storycode:copy>
<a id="f-8a4c3188ff6c-ExecBlock"></a>
### `ExecBlock`

<storycode:copy id="f-8a4c3188ff6c-ExecBlock">
export type ExecBlock = Extract<Block, { kind: "exec" }>;
</storycode:copy>
<a id="f-8a4c3188ff6c-DiagramBlock"></a>
### `DiagramBlock`

<storycode:copy id="f-8a4c3188ff6c-DiagramBlock">
export type DiagramBlock = Extract<Block, { kind: "diagram" }>;

</storycode:copy>
<a id="f-8a4c3188ff6c-RenderResponse"></a>
### `RenderResponse`

<storycode:copy id="f-8a4c3188ff6c-RenderResponse">
export interface RenderResponse {
  blocks: Block[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-export_statement-2"></a>
### `export_statement`

<storycode:copy id="f-8a4c3188ff6c-export_statement-2">
export type { FileOpRequest } from "./files";

</storycode:copy>
<a id="f-8a4c3188ff6c-export_statement-3"></a>
### `export_statement`

<storycode:copy id="f-8a4c3188ff6c-export_statement-3">
export type { BlockActionOutcome, ElementDescription } from "./elements";

</storycode:copy>
<a id="f-8a4c3188ff6c-import_statement"></a>
### `import_statement`

<storycode:copy id="f-8a4c3188ff6c-import_statement">
import type { RunStatus } from "./runs";
</storycode:copy>
<a id="f-8a4c3188ff6c-export_statement-4"></a>
### `export_statement`

<storycode:copy id="f-8a4c3188ff6c-export_statement-4">
export type { Run, RunStatus } from "./runs";

</storycode:copy>
<a id="f-8a4c3188ff6c-AgentWsEvent"></a>
### `AgentWsEvent`

Agent-loop events share the run channel (`exec_id: "agent"`, `run_id` =
session id). Serialized from `hickory_agent::AgentEvent`; only the fields
the dock renders are typed — the rest are kind-only so `kind` narrowing
stays exhaustive against `TranscriptEvent` (no member here carries `t`).

<storycode:copy id="f-8a4c3188ff6c-AgentWsEvent">
/** Agent-loop events share the run channel (`exec_id: "agent"`, `run_id` =
 * session id). Serialized from `hickory_agent::AgentEvent`; only the fields
 * the dock renders are typed — the rest are kind-only so `kind` narrowing
 * stays exhaustive against `TranscriptEvent` (no member here carries `t`). */
export type AgentWsEvent =
  | { kind: "acp_update"; update: import("./acp").AcpUpdate }
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

</storycode:copy>
<a id="f-8a4c3188ff6c-RunWsMessage"></a>
### `RunWsMessage`

<storycode:copy id="f-8a4c3188ff6c-RunWsMessage">
// WS channel 0x01 payloads.
export type RunWsMessage =
  | { run_id: string; exec_id: string; event: TranscriptEvent | AgentWsEvent }
  | { run_id: string; status: RunStatus }
  // The in-app up-loop re-wove this document and its output files changed on
  // disk (crates/hickory-cli/src/serve/watch.rs::notify_files_changed).
  | { files_changed: true; doc: string };

</storycode:copy>
<a id="f-8a4c3188ff6c-ProvenanceOrigin"></a>
### `ProvenanceOrigin`

Generated outputs & lineage (v0.2). All ranges/spans are BYTE offsets
(UTF-8): `start`/`end` into the output file's content, `span` into the
source document.

<storycode:copy id="f-8a4c3188ff6c-ProvenanceOrigin">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-Provenance"></a>
### `Provenance`

<storycode:copy id="f-8a4c3188ff6c-Provenance">
export interface Provenance {
  start: number;
  end: number;
  origin: ProvenanceOrigin;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-PlainFile"></a>
### `PlainFile`

One entry of the open folder's file tree (GET /api/files). Directories
come first, each level alphabetical; `path` is root-relative; `doc_id` is
present only on `.md` documents.
A plain file — anything in the folder that is neither a document nor a
woven output: read whole, saved whole. `hash` fingerprints the content the
read served; the save passes it back as `base_hash` so a file rewritten on
disk underneath the buffer is a 409, never a silent overwrite.

<storycode:copy id="f-8a4c3188ff6c-PlainFile">
/** One entry of the open folder's file tree (GET /api/files). Directories
 * come first, each level alphabetical; `path` is root-relative; `doc_id` is
 * present only on `.md` documents. */
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

</storycode:copy>
<a id="f-8a4c3188ff6c-PlainFileSaved"></a>
### `PlainFileSaved`

What `PUT /api/file` answers: the saved content's fresh hash, which
becomes the next save's `base_hash`.

<storycode:copy id="f-8a4c3188ff6c-PlainFileSaved">
/** What `PUT /api/file` answers: the saved content's fresh hash, which
 * becomes the next save's `base_hash`. */
export interface PlainFileSaved {
  path: string;
  hash: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ScratchpadSaved"></a>
### `ScratchpadSaved`

What `POST /api/adopt` answers: the document that now owns the file.
Where a scratchpad note landed, relative to the open folder.

<storycode:copy id="f-8a4c3188ff6c-ScratchpadSaved">
/** What `POST /api/adopt` answers: the document that now owns the file. */
/** Where a scratchpad note landed, relative to the open folder. */
export interface ScratchpadSaved {
  path: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-SampleCreated"></a>
### `SampleCreated`

<storycode:copy id="f-8a4c3188ff6c-SampleCreated">
/** Where a picked sample landed. */
export interface SampleCreated {
  doc_id: string;
  doc_path: string;
  /** The path as the document names it — relative to the document. */
  path: string;
  from: number;
  to: number;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-AdoptResponse"></a>
### `AdoptResponse`

<storycode:copy id="f-8a4c3188ff6c-AdoptResponse">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-RefactorDiff"></a>
### `RefactorDiff`

One output that no longer matches the pinned refactor baseline.

<storycode:copy id="f-8a4c3188ff6c-RefactorDiff">
/** One output that no longer matches the pinned refactor baseline. */
export interface RefactorDiff {
  path: string;
  kind: "changed" | "added" | "removed";
}

</storycode:copy>
<a id="f-8a4c3188ff6c-RefactorStatus"></a>
### `RefactorStatus`

The refactor baseline's live verdict — `active: false` when none is
pinned for the document. Weave-only on the server: checking never
executes a cell.

<storycode:copy id="f-8a4c3188ff6c-RefactorStatus">
/** The refactor baseline's live verdict — `active: false` when none is
 * pinned for the document. Weave-only on the server: checking never
 * executes a cell. */
export type RefactorStatus =
  | { active: false }
  | { active: true; started_at: string; clean: boolean; diffs: RefactorDiff[] };

</storycode:copy>
<a id="f-8a4c3188ff6c-FileNode"></a>
### `FileNode`

<storycode:copy id="f-8a4c3188ff6c-FileNode">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-FilesResponse"></a>
### `FilesResponse`

<storycode:copy id="f-8a4c3188ff6c-FilesResponse">
export interface FilesResponse {
  /** False for a single-file window; its parent is only an execution base. */
  folder_open?: boolean;
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
  truncated?: boolean;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-OutputFileMeta"></a>
### `OutputFileMeta`

<storycode:copy id="f-8a4c3188ff6c-OutputFileMeta">
export interface OutputFileMeta {
  path: string;
  language: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-DivergedOutput"></a>
### `DivergedOutput`

One produced file whose disk bytes are not what its document produces —
axis 3 of docs/specs/freeform/three-axes.md.

<storycode:copy id="f-8a4c3188ff6c-DivergedOutput">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-DivergedOutputs"></a>
### `DivergedOutputs`

GET /api/outputs/diverged, root-relative path → the file's state.

<storycode:copy id="f-8a4c3188ff6c-DivergedOutputs">
/** GET /api/outputs/diverged, root-relative path → the file's state. */
export interface DivergedOutputs {
  diverged: Record<string, DivergedOutput>;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ContextInput"></a>
### `ContextInput`

One input that was in front of the model when it wrote (context provenance).

<storycode:copy id="f-8a4c3188ff6c-ContextInput">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-ContextWrite"></a>
### `ContextWrite`

A run of lines an agent wrote, with everything that preceded the write in
its session. `current_lines` is where those lines are NOW, or null when the
file has moved on. Derived from the session record, never declared.

<storycode:copy id="f-8a4c3188ff6c-ContextWrite">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-ContextResponse"></a>
### `ContextResponse`

<storycode:copy id="f-8a4c3188ff6c-ContextResponse">
export interface ContextResponse {
  writes: ContextWrite[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-CitePlace"></a>
### `CitePlace`

A place a declared citation comes from or points at.

<storycode:copy id="f-8a4c3188ff6c-CitePlace">
/** A place a declared citation comes from or points at. */
export interface CitePlace {
  path: string;
  first_line: number;
  last_line: number;
  element: string;
  id: string | null;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-DeclaredCite"></a>
### `DeclaredCite`

One `cites="…"`: the author's assertion of what an element rests on,
resolved to places. Declared, forgeable, drawn as an assertion.

<storycode:copy id="f-8a4c3188ff6c-DeclaredCite">
/** One `cites="…"`: the author's assertion of what an element rests on,
 * resolved to places. Declared, forgeable, drawn as an assertion. */
export interface DeclaredCite {
  select: string;
  from: CitePlace;
  to: CitePlace[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-CitesResponse"></a>
### `CitesResponse`

<storycode:copy id="f-8a4c3188ff6c-CitesResponse">
export interface CitesResponse {
  cites: DeclaredCite[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-OutputsResponse"></a>
### `OutputsResponse`

<storycode:copy id="f-8a4c3188ff6c-OutputsResponse">
export interface OutputsResponse {
  files: OutputFileMeta[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-OutputFile"></a>
### `OutputFile`

<storycode:copy id="f-8a4c3188ff6c-OutputFile">
export interface OutputFile {
  path: string;
  language: string;
  content: string;
  provenance: Provenance[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-OutputEdit"></a>
### `OutputEdit`

<storycode:copy id="f-8a4c3188ff6c-OutputEdit">
export interface OutputEdit {
  start: number;
  end: number;
  text: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-SourceEdit"></a>
### `SourceEdit`

<storycode:copy id="f-8a4c3188ff6c-SourceEdit">
export interface SourceEdit {
  doc_path: string;
  span: [number, number];
  text: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-OutputEditResponse"></a>
### `OutputEditResponse`

<storycode:copy id="f-8a4c3188ff6c-OutputEditResponse">
export interface OutputEditResponse {
  source_edits: SourceEdit[];
  applied: true;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-SyntheticRangeError"></a>
### `SyntheticRangeError`

Body of the 422 returned when an edit overlaps a synthetic range.

<storycode:copy id="f-8a4c3188ff6c-SyntheticRangeError">
/** Body of the 422 returned when an edit overlaps a synthetic range. */
export interface SyntheticRangeError {
  error: string;
  range: { start: number; end: number };
}

</storycode:copy>
<a id="f-8a4c3188ff6c-Health"></a>
### `Health`

<storycode:copy id="f-8a4c3188ff6c-Health">
export interface Health {
  ok: boolean;
  executor: "local" | "canopy";
  db: boolean;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ExecutorInfo"></a>
### `ExecutorInfo`

GET /api/executor — where cells run (environment cards). `images` is the
image ref → Nix store path map on canopy; null on local, where the image
attribute is recorded provenance, not an enforced sandbox.

<storycode:copy id="f-8a4c3188ff6c-ExecutorInfo">
/** GET /api/executor — where cells run (environment cards). `images` is the
 * image ref → Nix store path map on canopy; null on local, where the image
 * attribute is recorded provenance, not an enforced sandbox. */
export interface ExecutorInfo {
  kind: "local" | "canopy";
  images: Record<string, string> | null;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-export_statement-5"></a>
### `export_statement`

One exchange in a document's agent conversation.
The conversation is a TREE: `parent_id` is the turn this one continues from,
so rewinding and sending again forks a branch instead of destroying history.

<storycode:copy id="f-8a4c3188ff6c-export_statement-5">
/** One exchange in a document's agent conversation.
 *
 * The conversation is a TREE: `parent_id` is the turn this one continues from,
 * so rewinding and sending again forks a branch instead of destroying history. */
export type { AgentUsage, AgentTurn, AgentTotals, AgentTurnsResponse } from "./agentTypes";

</storycode:copy>
<a id="f-8a4c3188ff6c-SearchHit"></a>
### `SearchHit`

--- project search ----------------------------------------------------------
GET /api/search — ranked hits across the served folder: .md documents and
generated files alike. Lines are 1-based; `path` is relative to the folder.

<storycode:copy id="f-8a4c3188ff6c-SearchHit">
// --- project search ----------------------------------------------------------
// GET /api/search — ranked hits across the served folder: .md documents and
// generated files alike. Lines are 1-based; `path` is relative to the folder.

export interface SearchHit {
  path: string;
  start_line: number;
  end_line: number;
  score: number;
  snippet: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-SearchResponse"></a>
### `SearchResponse`

<storycode:copy id="f-8a4c3188ff6c-SearchResponse">
export interface SearchResponse {
  /** False when only lexical ranking is available — the semantic model is an
   * optional download (`hick search --install-model`), never a requirement. */
  semantic: boolean;
  hits: SearchHit[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-StructureDefinition"></a>
### `StructureDefinition`

--- structural navigation ---------------------------------------------------
Definitions and references found by tree-sitter (crates/hick-structure).
Resolution is by NAME, so `candidates` says how many definitions a link
could have meant; the view draws an ambiguous link differently rather than
implying certainty.

<storycode:copy id="f-8a4c3188ff6c-StructureDefinition">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-StructureReference"></a>
### `StructureReference`

<storycode:copy id="f-8a4c3188ff6c-StructureReference">
export interface StructureReference {
  name: string;
  line: number;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-FileStructure"></a>
### `FileStructure`

<storycode:copy id="f-8a4c3188ff6c-FileStructure">
export interface FileStructure {
  path: string;
  language: string;
  definitions: StructureDefinition[];
  references: StructureReference[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-StructuralLink"></a>
### `StructuralLink`

<storycode:copy id="f-8a4c3188ff6c-StructuralLink">
export interface StructuralLink {
  from_path: string;
  from_line: number;
  to_path: string;
  to_line: number;
  name: string;
  candidates: number;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-StructureResponse"></a>
### `StructureResponse`

<storycode:copy id="f-8a4c3188ff6c-StructureResponse">
export interface StructureResponse {
  files: FileStructure[];
  links: StructuralLink[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ProviderId"></a>
### `ProviderId`

--- settings: LLM API keys --------------------------------------------------
GET/PUT /api/settings/keys. The server NEVER returns a full key — only
whether one is configured, and a masked hint for telling keys apart.

<storycode:copy id="f-8a4c3188ff6c-ProviderId">
// --- settings: LLM API keys --------------------------------------------------
// GET/PUT /api/settings/keys. The server NEVER returns a full key — only
// whether one is configured, and a masked hint for telling keys apart.
export type ProviderId =
  "anthropic" | "openai" | "openrouter" | "deepseek" | "xai" | "gab";
</storycode:copy>
<a id="f-8a4c3188ff6c-ProviderKey"></a>
### `ProviderKey`

<storycode:copy id="f-8a4c3188ff6c-ProviderKey">
export interface ProviderKey {
  id: ProviderId;
  label: string;
  configured: boolean;
  /** e.g. "sk-a…f3" when configured; null otherwise. Never the full key. */
  masked: string | null;
}
</storycode:copy>
<a id="f-8a4c3188ff6c-SettingsKeysResponse"></a>
### `SettingsKeysResponse`

<storycode:copy id="f-8a4c3188ff6c-SettingsKeysResponse">
export interface SettingsKeysResponse {
  providers: ProviderKey[];
}
</storycode:copy>
<a id="f-8a4c3188ff6c-SettingsKeysPatch"></a>
### `SettingsKeysPatch`

PUT body: only the providers being changed — a string sets, null clears.
Untouched providers are simply absent.

<storycode:copy id="f-8a4c3188ff6c-SettingsKeysPatch">
/** PUT body: only the providers being changed — a string sets, null clears.
 * Untouched providers are simply absent. */
export type SettingsKeysPatch = Partial<Record<ProviderId, string | null>>;
</storycode:copy>
<a id="f-8a4c3188ff6c-UiSettings"></a>
### `UiSettings`

--- settings: UI ------------------------------------------------------------
GET/PUT /api/settings/ui. Persisted server-side (ui.json beside
llm-keys.json) so the desktop shell can read the custom window title at
launch, before any page has loaded.

<storycode:copy id="f-8a4c3188ff6c-UiSettings">
// --- settings: UI ------------------------------------------------------------
// GET/PUT /api/settings/ui. Persisted server-side (ui.json beside
// llm-keys.json) so the desktop shell can read the custom window title at
// launch, before any page has loaded.
export interface UiSettings {
  /** Custom window title, or null for the default (folder / file name). */
  window_title: string | null;
  /** Run the file's formatter when Save is chosen. */
  format_on_save: boolean;
  /** Recover unsaved buffers for files that already have a name. */
  retain_unsaved_saved_files?: boolean;
  /** The keyboard profile and overrides, as lib/keymap.ts keeps them. */
  keymap?: { profile?: unknown; overrides?: Record<string, unknown> } | null;
  /** The menu bar's resolved accelerators, by menu id; the shell reads
   * these at launch. */
  native_accelerators?: Record<string, string | null>;
}
</storycode:copy>
<a id="f-8a4c3188ff6c-SessionState"></a>
### `SessionState`

--- terminals ---------------------------------------------------------------
/api/terminals. A terminal here is a SESSION: named work, in a directory,
on a branch, that knows whether it is busy, blocked, or done — and keeps
knowing while its pane is closed. See crates/hick-term.
The five states. Working and idle make no claim on your attention.

<storycode:copy id="f-8a4c3188ff6c-SessionState">
// --- terminals ---------------------------------------------------------------
// /api/terminals. A terminal here is a SESSION: named work, in a directory,
// on a branch, that knows whether it is busy, blocked, or done — and keeps
// knowing while its pane is closed. See crates/hick-term.
/** The five states. Working and idle make no claim on your attention. */
export type SessionState =
  "needs-you" | "working" | "idle" | "finished" | "failed";
</storycode:copy>
<a id="f-8a4c3188ff6c-PromptSource"></a>
### `PromptSource`

Where a prompt came from, which decides how far it may be trusted:
"declared" is structural (the program said so, with its choices),
"guessed" is us recognising the shape of a question on a screen.

<storycode:copy id="f-8a4c3188ff6c-PromptSource">
/** Where a prompt came from, which decides how far it may be trusted:
 * "declared" is structural (the program said so, with its choices),
 * "guessed" is us recognising the shape of a question on a screen. */
export type PromptSource = "declared" | "guessed";

</storycode:copy>
<a id="f-8a4c3188ff6c-PromptChoice"></a>
### `PromptChoice`

<storycode:copy id="f-8a4c3188ff6c-PromptChoice">
export interface PromptChoice {
  label: string;
  /** Exactly what gets written to the terminal when it is pressed. */
  send: string;
  destructive: boolean;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-TerminalPrompt"></a>
### `TerminalPrompt`

<storycode:copy id="f-8a4c3188ff6c-TerminalPrompt">
export interface TerminalPrompt {
  question: string;
  /** Empty for a guessed prompt: we can see that something is being asked,
   * not what the answers are. The card then offers a plain input line. */
  choices: PromptChoice[];
  source: PromptSource;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-TerminalAnchor"></a>
### `TerminalAnchor`

A terminal that is writing into a document.
The persistent binding from
`docs/specs/freeform/a-terminal-that-writes-the-document.md`. The anchor is
the CONTAINER, not the document: several `hick:exec` blocks naming one
container is already how the language says "commands that share state", so
a terminal bound to `sdk` in a document is that shell. Say "anchored to a
container", never "attached to a document".

<storycode:copy id="f-8a4c3188ff6c-TerminalAnchor">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-TerminalSession"></a>
### `TerminalSession`

<storycode:copy id="f-8a4c3188ff6c-TerminalSession">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-TerminalsResponse"></a>
### `TerminalsResponse`

<storycode:copy id="f-8a4c3188ff6c-TerminalsResponse">
export interface TerminalsResponse {
  sessions: TerminalSession[];
  /** Session ids claiming attention, most-claiming first. The server owns
   * this order (hick_term::attention) so every surface agrees on it. */
  attention: string[];
  turbo: boolean;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-OpenTerminal"></a>
### `OpenTerminal`

POST /api/terminals. Everything is optional: no body at all opens a shell
in the open folder.

<storycode:copy id="f-8a4c3188ff6c-OpenTerminal">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-WorkspaceDraft"></a>
### `WorkspaceDraft`

--- Workspace state and drafts (api.md, "Workspace state and drafts") ----
One buffer's unsaved contents, and what it was unsaved *from*.

<storycode:copy id="f-8a4c3188ff6c-WorkspaceDraft">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-WorkspaceUiState"></a>
### `WorkspaceUiState`

The stored window layout. Opaque on the wire; `lib/uiState.ts` owns its
shape, and is the only thing that should ever narrow this type.

<storycode:copy id="f-8a4c3188ff6c-WorkspaceUiState">
/** The stored window layout. Opaque on the wire; `lib/uiState.ts` owns its
 * shape, and is the only thing that should ever narrow this type. */
export type WorkspaceUiState = unknown;

</storycode:copy>
<a id="f-8a4c3188ff6c-FindMatch"></a>
### `FindMatch`

--- Exhaustive find and replace (api.md, "Find and replace") -------------
Deliberately separate from SearchResponse, which is RANKED. A ranked answer
is a sample, and replacing across a sample changes some of the occurrences.

<storycode:copy id="f-8a4c3188ff6c-FindMatch">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-FindFile"></a>
### `FindFile`

<storycode:copy id="f-8a4c3188ff6c-FindFile">
export interface FindFile {
  path: string;
  source_hash?: string;
  matches: FindMatch[];
  /** The document that writes this file, when one does. Replace refuses it,
   * and the UI greys it out before the button is pressed. */
  generated_by?: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-FindResponse"></a>
### `FindResponse`

<storycode:copy id="f-8a4c3188ff6c-FindResponse">
export interface FindResponse {
  files: FindFile[];
  /** The match cap was hit; there are more than these. */
  truncated: boolean;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-FindOptions"></a>
### `FindOptions`

<storycode:copy id="f-8a4c3188ff6c-FindOptions">
export interface FindOptions {
  regex?: boolean;
  case?: boolean;
  whole_word?: boolean;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ReplaceResponse"></a>
### `ReplaceResponse`

<storycode:copy id="f-8a4c3188ff6c-ReplaceResponse">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-BlameLine"></a>
### `BlameLine`

<storycode:copy id="f-8a4c3188ff6c-BlameLine">
/** One line's authorship, from `GET /api/blame`. */
export interface BlameLine {
  text?: string;
  origin?: "human" | "agent" | "imported" | "unknown";
  provider?: string;
  line: number;
  commit: string;
  author: string;
  email: string;
  /** Unix seconds; formatted by the client, in the reader's own locale. */
  time: number;
  summary: string;
  uncommitted: boolean;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-FormulaResults"></a>
### `FormulaResults`

What a table's formulas came to. Keyed by A1 label, and only for cells
that ARE formulas — echoing literals back would make the response the size
of the table for no reason.

<storycode:copy id="f-8a4c3188ff6c-FormulaResults">
/** What a table's formulas came to. Keyed by A1 label, and only for cells
 * that ARE formulas — echoing literals back would make the response the size
 * of the table for no reason. */
export interface FormulaResults {
  values: Record<string, string>;
  errors: Record<string, string>;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-FormulaBinding"></a>
### `FormulaBinding`

What one reference was worth when the cell that used it ran. `kind` is
carried because `empty` is not the empty string — a blank cell is skipped
by `sum` where `""` is not, and a debugger showing both as nothing would
hide the difference.

<storycode:copy id="f-8a4c3188ff6c-FormulaBinding">
/** What one reference was worth when the cell that used it ran. `kind` is
 * carried because `empty` is not the empty string — a blank cell is skipped
 * by `sum` where `""` is not, and a debugger showing both as nothing would
 * hide the difference. */
export interface FormulaBinding {
  cell: string;
  text: string;
  kind: "number" | "text" | "bool" | "empty";
}

</storycode:copy>
<a id="f-8a4c3188ff6c-FormulaStep"></a>
### `FormulaStep`

One cell's turn, as the table's debugger steps through it.

<storycode:copy id="f-8a4c3188ff6c-FormulaStep">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-FormulaTrace"></a>
### `FormulaTrace`

The same evaluation `evaluateFormulas` does, with every step kept. Steps
are empty for a sheet with a cycle: a circle has no order, and inventing
one to step through would be the debugger telling its first lie.

<storycode:copy id="f-8a4c3188ff6c-FormulaTrace">
/** The same evaluation `evaluateFormulas` does, with every step kept. Steps
 * are empty for a sheet with a cycle: a circle has no order, and inventing
 * one to step through would be the debugger telling its first lie. */
export interface FormulaTrace extends FormulaResults {
  steps: FormulaStep[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ProjectSuggestion"></a>
### `ProjectSuggestion`

One completion drawn from the project's own text. `semantic` is false when
the local model is not installed — the ranking is then frequency alone,
which is said rather than implied.

<storycode:copy id="f-8a4c3188ff6c-ProjectSuggestion">
/** One completion drawn from the project's own text. `semantic` is false when
 * the local model is not installed — the ranking is then frequency alone,
 * which is said rather than implied. */
export interface ProjectSuggestion {
  text: string;
  detail: string;
  score: number;
  semantic: boolean;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-GitFileChange"></a>
### `GitFileChange`

<storycode:copy id="f-8a4c3188ff6c-GitFileChange">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-GitRecipe"></a>
### `GitRecipe`

A commit's recipe, from its `Hick-Recipe` / `Hick-Image` / `Hick-Output`
trailers. A DECLARED claim in the commit's own words: nothing has checked
it until a replay does. docs/specs/freeform/lenses.md

<storycode:copy id="f-8a4c3188ff6c-GitRecipe">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-GitReplay"></a>
### `GitReplay`

<storycode:copy id="f-8a4c3188ff6c-GitReplay">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-GitRecipeRun"></a>
### `GitRecipeRun`

What `POST /api/git/recipe` made: a recipe commit, now HEAD.

<storycode:copy id="f-8a4c3188ff6c-GitRecipeRun">
/** What `POST /api/git/recipe` made: a recipe commit, now HEAD. */
export interface GitRecipeRun {
  sha: string;
  short: string;
  output_tree: string;
  said: string[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-export_statement-6"></a>
### `export_statement`

One commit as a card (`GET /api/git/commit?sha=`): its diff, its files,
and which of those files a later commit changed.

<storycode:copy id="f-8a4c3188ff6c-export_statement-6">
/** One commit as a card (`GET /api/git/commit?sha=`): its diff, its files,
 * and which of those files a later commit changed. */
export type { GitCommitDetail, GitCommit, GitLog, PublicationFloor } from "./gitHistoryTypes";

</storycode:copy>
<a id="f-8a4c3188ff6c-ReplayCommit"></a>
### `ReplayCommit`

One commit a document's time slider can stop at.

<storycode:copy id="f-8a4c3188ff6c-ReplayCommit">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-ReplayResponse"></a>
### `ReplayResponse`

A replay of one document at one commit.
`grammar_boundary` is not a failure: replay weaves an old document with
today's binary, and this product reserves the right to change the grammar.
Past that point the honest claim is that replay works back to the last
grammar change, and `message` says exactly that.

<storycode:copy id="f-8a4c3188ff6c-ReplayResponse">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-export_statement-7"></a>
### `export_statement`

The Git pane's types live in ./git.ts; importers keep finding them here.

<storycode:copy id="f-8a4c3188ff6c-export_statement-7">
// The Git pane's types live in ./git.ts; importers keep finding them here.
export type {
  MergeDriverStatus,
  InitOutcome,
  GitStatus,
  GitChangeFile,
  GitChanges,
  GitDiff,
  GitBranch,
  GitSaid,
  GitCommitResult,
} from "./git";

</storycode:copy>
<a id="f-8a4c3188ff6c-SavedAsset"></a>
### `SavedAsset`

Where an image dropped into a note was written (POST /api/asset).

<storycode:copy id="f-8a4c3188ff6c-SavedAsset">
/** Where an image dropped into a note was written (POST /api/asset). */
export interface SavedAsset {
  /** Root-relative — what the tree calls it, and what `GET /api/asset` takes. */
  path: string;
  /** Relative to the document that references it: what goes inside the
   * markdown parentheses, so the note survives the folder being moved. */
  relative: string;
  bytes: number;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-SessionStep"></a>
### `SessionStep`

---- a session file, read back as the conversation it records ------------

<storycode:copy id="f-8a4c3188ff6c-SessionStep">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-SessionTurn"></a>
### `SessionTurn`

<storycode:copy id="f-8a4c3188ff6c-SessionTurn">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-SessionView"></a>
### `SessionView`

<storycode:copy id="f-8a4c3188ff6c-SessionView">
export interface SessionView {
  start: string | null;
  doc: string | null;
  turns: SessionTurn[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-export_statement-8"></a>
### `export_statement`

<storycode:copy id="f-8a4c3188ff6c-export_statement-8">
export type { SessionViewResponse } from "./sessionTypes";

</storycode:copy>
<a id="f-8a4c3188ff6c-export_statement-9"></a>
### `export_statement`

<storycode:copy id="f-8a4c3188ff6c-export_statement-9">
export type { WorktreeInfo, MergedRegion, MergedViewResponse } from "./mergedTypes";

</storycode:copy>
<a id="f-8a4c3188ff6c-ContinuitySettings"></a>
### `ContinuitySettings`

Continuity — the fourth provenance family, off by default. The whole of it
rides this one switch: no ribbon, no journal, no pre-commit repair.

<storycode:copy id="f-8a4c3188ff6c-ContinuitySettings">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-FleetMachine"></a>
### `FleetMachine`

---- the fleet -----------------------------------------------------------
One machine whose key this one holds.
A machine is a keypair; a fleet is a mutual list of public keys under the
user's own state directory. There is no account and no server, and
revocation is deleting a key.
docs/specs/freeform/one-engineer-many-machines.md

<storycode:copy id="f-8a4c3188ff6c-FleetMachine">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-FleetResponse"></a>
### `FleetResponse`

<storycode:copy id="f-8a4c3188ff6c-FleetResponse">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldTemplate"></a>
### `ScaffoldTemplate`

---- New Project ---------------------------------------------------------
One row of `dotnet new list`.
crates/hickory-cli/src/scaffold.rs

<storycode:copy id="f-8a4c3188ff6c-ScaffoldTemplate">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldToolchain"></a>
### `ScaffoldToolchain`

One scaffolder this machine actually has.

<storycode:copy id="f-8a4c3188ff6c-ScaffoldToolchain">
/** One scaffolder this machine actually has. */
export interface ScaffoldToolchain {
  /** `dotnet`, `uv`, `cargo` — what a request names it by. */
  id: string;
  /** What the picker calls it: ".NET", "Python (uv)", "Rust (cargo)". */
  label: string;
  /** The language a project made with it is written in. */
  language: string;
  /** What the tool reported for `--version`, verbatim. */
  version: string;
  /** Whether it has a template catalogue (dotnet) or one shape chosen by
   * flags (uv, cargo). A single-shape tool draws no template list. */
  templated: boolean;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldCatalog"></a>
### `ScaffoldCatalog`

<storycode:copy id="f-8a4c3188ff6c-ScaffoldCatalog">
export interface ScaffoldCatalog {
  /** Which scaffolder this catalogue is for. */
  kind: string;
  /** Every scaffolder this machine has, measured rather than declared —
   * so the picker can never offer one that would fail after the form is
   * filled in. */
  toolchains: ScaffoldToolchain[];
  sdk_version: string;
  /** The SDK image matching that version, for the document's container. */
  image: string;
  /** The folder the app has open, absolute — what the location field starts
   * on. A project may be made anywhere on the machine; this is only where
   * the field opens. */
  location: string;
  /** This machine's path separator, for joining a location to a folder name
   * in the one place the dialog shows the two together. */
  separator: string;
  /** Whether the program hosting the engine can show a native folder
   * chooser. False under `hick up` in a browser, where the location can only
   * be typed — and where the ellipsis is therefore not drawn at all. */
  can_pick_folder: boolean;
  templates: ScaffoldTemplate[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldChoice"></a>
### `ScaffoldChoice`

<storycode:copy id="f-8a4c3188ff6c-ScaffoldChoice">
export interface ScaffoldChoice {
  value: string;
  description: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-TemplateOption"></a>
### `TemplateOption`

<storycode:copy id="f-8a4c3188ff6c-TemplateOption">
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

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldTemplateDetail"></a>
### `ScaffoldTemplateDetail`

<storycode:copy id="f-8a4c3188ff6c-ScaffoldTemplateDetail">
export interface ScaffoldTemplateDetail {
  title: string;
  author: string;
  description: string;
  options: TemplateOption[];
  /** Languages the help footer says to ask for separately. */
  other_languages: string[];
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ChosenOption"></a>
### `ChosenOption`

One `--flag value` the form decided on. `value` absent is a bare switch.

<storycode:copy id="f-8a4c3188ff6c-ChosenOption">
/** One `--flag value` the form decided on. `value` absent is a bare switch. */
export interface ChosenOption {
  flag: string;
  value?: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldSpec"></a>
### `ScaffoldSpec`

What the dialog is asking to be scaffolded.

<storycode:copy id="f-8a4c3188ff6c-ScaffoldSpec">
/** What the dialog is asking to be scaffolded. */
export interface ScaffoldSpec {
  template: string;
  title: string;
  language: string | null;
  name: string;
  /** The project's folder *name* on the way out. The server answers with the
   * same folder relative to the repository that will record it, which is not
   * the same string once the location is somewhere else on the machine. */
  output: string;
  /** Where that folder is made: any folder on this machine, absolute or
   * `~`-prefixed; relative is read against the open folder. */
  location: string;
  /** Make a git repository in the location first, if it is not in one. */
  init_repository: boolean;
  /** Where to open the project once it is committed — never before, and
   * never at all if the scaffolder failed. */
  open: OpenWhere;
  image: string;
  options: ChosenOption[];
  /** Which scaffolder. Absent means `dotnet`, which is what every request
   * meant before there was a choice. */
  toolchain?: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-OpenWhere"></a>
### `OpenWhere`

Where a folder is opened. `this-window` is a restart: a session is a
process, so everything in this window goes, terminals included.

<storycode:copy id="f-8a4c3188ff6c-OpenWhere">
/** Where a folder is opened. `this-window` is a restart: a session is a
 * process, so everything in this window goes, terminals included. */
export type OpenWhere = "none" | "new-window" | "this-window";

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldPreview"></a>
### `ScaffoldPreview`

The commit a New Project would make, without making it.

<storycode:copy id="f-8a4c3188ff6c-ScaffoldPreview">
/** The commit a New Project would make, without making it. */
export interface ScaffoldPreview {
  /** The folder, relative to the repository that would record it. */
  output: string;
  /** That folder, absolute. Null when there is no repository to place it in
   * relative to. */
  folder: string | null;
  /** The repository the recipe commit would be made in, or `null` when the
   * location is inside none. */
  repository: string | null;
  /** Set with `repository: null`: the folder a `git init` would run in. */
  needs_repository?: string | null;
  /** Why there is no repository, in the server's own words. */
  problem?: string;
  command: string;
  /** The message, trailers included; the tree hash is a placeholder. */
  message: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldStarted"></a>
### `ScaffoldStarted`

What `POST /api/scaffold` started: a terminal running the scaffolder.
Not a commit — the commit happens when that terminal's command exits, and
`scaffoldResult` is how the app learns it did. The person learns it from
the terminal, which says so in its own scrollback.

<storycode:copy id="f-8a4c3188ff6c-ScaffoldStarted">
/** What `POST /api/scaffold` started: a terminal running the scaffolder.
 *
 * Not a commit — the commit happens when that terminal's command exits, and
 * `scaffoldResult` is how the app learns it did. The person learns it from
 * the terminal, which says so in its own scrollback. */
export interface ScaffoldStarted {
  session: TerminalSession;
  output: string;
  folder: string;
  repository: string;
}

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldResult"></a>
### `ScaffoldResult`

<storycode:copy id="f-8a4c3188ff6c-ScaffoldResult">
/** How a started scaffold ended. */
export type ScaffoldResult =
  | { state: "running" }
  | { state: "failed"; error: string }
  | ({ state: "committed" } & ScaffoldCreated);

</storycode:copy>
<a id="f-8a4c3188ff6c-ScaffoldCreated"></a>
### `ScaffoldCreated`

The commit a scaffold made, carrying its recipe.

<storycode:copy id="f-8a4c3188ff6c-ScaffoldCreated">
/** The commit a scaffold made, carrying its recipe. */
export interface ScaffoldCreated {
  sha: string;
  short: string;
  /** The folder the scaffold landed in, relative to its repository. */
  output: string;
  files: string[];
  message: string;
  /** The `Hick-Output` tree hash. */
  output_tree: string;
  /** The repository the commit was made in — not necessarily the open one. */
  repository: string;
}
</storycode:copy>

### Assemble `apps/web/src/api/types.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/types.ts"><storycode:paste select="#f-8a4c3188ff6c-Project"/><storycode:paste select="#f-8a4c3188ff6c-DocSummary"/><storycode:paste select="#f-8a4c3188ff6c-Doc"/><storycode:paste select="#f-8a4c3188ff6c-TranscriptEvent"/><storycode:paste select="#f-8a4c3188ff6c-ExecStatus"/><storycode:paste select="#f-8a4c3188ff6c-Block"/><storycode:paste select="#f-8a4c3188ff6c-export_statement"/><storycode:paste select="#f-8a4c3188ff6c-ExecBlock"/><storycode:paste select="#f-8a4c3188ff6c-DiagramBlock"/><storycode:paste select="#f-8a4c3188ff6c-RenderResponse"/><storycode:paste select="#f-8a4c3188ff6c-export_statement-2"/><storycode:paste select="#f-8a4c3188ff6c-export_statement-3"/><storycode:paste select="#f-8a4c3188ff6c-import_statement"/><storycode:paste select="#f-8a4c3188ff6c-export_statement-4"/><storycode:paste select="#f-8a4c3188ff6c-AgentWsEvent"/><storycode:paste select="#f-8a4c3188ff6c-RunWsMessage"/><storycode:paste select="#f-8a4c3188ff6c-ProvenanceOrigin"/><storycode:paste select="#f-8a4c3188ff6c-Provenance"/><storycode:paste select="#f-8a4c3188ff6c-PlainFile"/><storycode:paste select="#f-8a4c3188ff6c-PlainFileSaved"/><storycode:paste select="#f-8a4c3188ff6c-ScratchpadSaved"/><storycode:paste select="#f-8a4c3188ff6c-SampleCreated"/><storycode:paste select="#f-8a4c3188ff6c-AdoptResponse"/><storycode:paste select="#f-8a4c3188ff6c-RefactorDiff"/><storycode:paste select="#f-8a4c3188ff6c-RefactorStatus"/><storycode:paste select="#f-8a4c3188ff6c-FileNode"/><storycode:paste select="#f-8a4c3188ff6c-FilesResponse"/><storycode:paste select="#f-8a4c3188ff6c-OutputFileMeta"/><storycode:paste select="#f-8a4c3188ff6c-DivergedOutput"/><storycode:paste select="#f-8a4c3188ff6c-DivergedOutputs"/><storycode:paste select="#f-8a4c3188ff6c-ContextInput"/><storycode:paste select="#f-8a4c3188ff6c-ContextWrite"/><storycode:paste select="#f-8a4c3188ff6c-ContextResponse"/><storycode:paste select="#f-8a4c3188ff6c-CitePlace"/><storycode:paste select="#f-8a4c3188ff6c-DeclaredCite"/><storycode:paste select="#f-8a4c3188ff6c-CitesResponse"/><storycode:paste select="#f-8a4c3188ff6c-OutputsResponse"/><storycode:paste select="#f-8a4c3188ff6c-OutputFile"/><storycode:paste select="#f-8a4c3188ff6c-OutputEdit"/><storycode:paste select="#f-8a4c3188ff6c-SourceEdit"/><storycode:paste select="#f-8a4c3188ff6c-OutputEditResponse"/><storycode:paste select="#f-8a4c3188ff6c-SyntheticRangeError"/><storycode:paste select="#f-8a4c3188ff6c-Health"/><storycode:paste select="#f-8a4c3188ff6c-ExecutorInfo"/><storycode:paste select="#f-8a4c3188ff6c-export_statement-5"/><storycode:paste select="#f-8a4c3188ff6c-SearchHit"/><storycode:paste select="#f-8a4c3188ff6c-SearchResponse"/><storycode:paste select="#f-8a4c3188ff6c-StructureDefinition"/><storycode:paste select="#f-8a4c3188ff6c-StructureReference"/><storycode:paste select="#f-8a4c3188ff6c-FileStructure"/><storycode:paste select="#f-8a4c3188ff6c-StructuralLink"/><storycode:paste select="#f-8a4c3188ff6c-StructureResponse"/><storycode:paste select="#f-8a4c3188ff6c-ProviderId"/><storycode:paste select="#f-8a4c3188ff6c-ProviderKey"/><storycode:paste select="#f-8a4c3188ff6c-SettingsKeysResponse"/><storycode:paste select="#f-8a4c3188ff6c-SettingsKeysPatch"/><storycode:paste select="#f-8a4c3188ff6c-UiSettings"/><storycode:paste select="#f-8a4c3188ff6c-SessionState"/><storycode:paste select="#f-8a4c3188ff6c-PromptSource"/><storycode:paste select="#f-8a4c3188ff6c-PromptChoice"/><storycode:paste select="#f-8a4c3188ff6c-TerminalPrompt"/><storycode:paste select="#f-8a4c3188ff6c-TerminalAnchor"/><storycode:paste select="#f-8a4c3188ff6c-TerminalSession"/><storycode:paste select="#f-8a4c3188ff6c-TerminalsResponse"/><storycode:paste select="#f-8a4c3188ff6c-OpenTerminal"/><storycode:paste select="#f-8a4c3188ff6c-WorkspaceDraft"/><storycode:paste select="#f-8a4c3188ff6c-WorkspaceUiState"/><storycode:paste select="#f-8a4c3188ff6c-FindMatch"/><storycode:paste select="#f-8a4c3188ff6c-FindFile"/><storycode:paste select="#f-8a4c3188ff6c-FindResponse"/><storycode:paste select="#f-8a4c3188ff6c-FindOptions"/><storycode:paste select="#f-8a4c3188ff6c-ReplaceResponse"/><storycode:paste select="#f-8a4c3188ff6c-BlameLine"/><storycode:paste select="#f-8a4c3188ff6c-FormulaResults"/><storycode:paste select="#f-8a4c3188ff6c-FormulaBinding"/><storycode:paste select="#f-8a4c3188ff6c-FormulaStep"/><storycode:paste select="#f-8a4c3188ff6c-FormulaTrace"/><storycode:paste select="#f-8a4c3188ff6c-ProjectSuggestion"/><storycode:paste select="#f-8a4c3188ff6c-GitFileChange"/><storycode:paste select="#f-8a4c3188ff6c-GitRecipe"/><storycode:paste select="#f-8a4c3188ff6c-GitReplay"/><storycode:paste select="#f-8a4c3188ff6c-GitRecipeRun"/><storycode:paste select="#f-8a4c3188ff6c-export_statement-6"/><storycode:paste select="#f-8a4c3188ff6c-ReplayCommit"/><storycode:paste select="#f-8a4c3188ff6c-ReplayResponse"/><storycode:paste select="#f-8a4c3188ff6c-export_statement-7"/><storycode:paste select="#f-8a4c3188ff6c-SavedAsset"/><storycode:paste select="#f-8a4c3188ff6c-SessionStep"/><storycode:paste select="#f-8a4c3188ff6c-SessionTurn"/><storycode:paste select="#f-8a4c3188ff6c-SessionView"/><storycode:paste select="#f-8a4c3188ff6c-export_statement-8"/><storycode:paste select="#f-8a4c3188ff6c-export_statement-9"/><storycode:paste select="#f-8a4c3188ff6c-ContinuitySettings"/><storycode:paste select="#f-8a4c3188ff6c-FleetMachine"/><storycode:paste select="#f-8a4c3188ff6c-FleetResponse"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldTemplate"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldToolchain"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldCatalog"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldChoice"/><storycode:paste select="#f-8a4c3188ff6c-TemplateOption"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldTemplateDetail"/><storycode:paste select="#f-8a4c3188ff6c-ChosenOption"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldSpec"/><storycode:paste select="#f-8a4c3188ff6c-OpenWhere"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldPreview"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldStarted"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldResult"/><storycode:paste select="#f-8a4c3188ff6c-ScaffoldCreated"/></storycode:file>

</storycode:doc>
