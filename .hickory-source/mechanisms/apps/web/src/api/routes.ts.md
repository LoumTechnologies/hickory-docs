<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Declare API forwarding once

For maintainers reviewing endpoint intent. Forwarding rules live in the shared
[generator](../../../../scripts/generate-api.py.md); review them once when changing
policy, then check each declaration against the intended API request.

## Route vocabulary

A route is a TypeScript method signature with verb, path, response and body
metadata in its return type. These interfaces are erased at runtime. The code
model resolves imported types and supplies parameters and type arguments; the
generator never parses TypeScript signatures with regular expressions.

<storycode:copy id="api-codegen-route-vocabulary">
import type * as Fleet from "./fleetPairings";
import type * as Types from "./types";
import type * as Github from "./github";

interface Endpoint<P, R, B> { path: P; response: R; body: B; }
interface GET<P extends string, R, B extends string = "$fields"> extends Endpoint<P, R, B> {}
interface POST<P extends string, R, B extends string = "$fields"> extends Endpoint<P, R, B> {}
interface PUT<P extends string, R, B extends string = "$fields"> extends Endpoint<P, R, B> {}
interface DELETE<P extends string, R, B extends string = "$fields"> extends Endpoint<P, R, B> {}

export interface ApiRoutes {
</storycode:copy>

## Routes projects

Projects and files declare the same public signatures as the old wrappers.
`{id}` preserves literal identifier interpolation; `{path:url}` encodes a URL
component. Non-path parameters become JSON fields unless a body name is explicit.

<storycode:copy id="api-codegen-routes-projects">
  projects(): GET<"/api/projects", Types.Project[]>;
  createProject(name: string, visibility: "public" | "private"): POST<"/api/projects", Types.Project>;
  projectDocs(projectId: string): GET<"/api/projects/{projectId}/docs", Types.DocSummary[]>;
  createDoc(projectId: string, path: string, source: string): POST<"/api/projects/{projectId}/docs", Types.Doc>;
  doc(id: string): GET<"/api/docs/{id}", Types.Doc>;
  saveDoc(id: string, source: string): PUT<"/api/docs/{id}", Types.Doc>;
/** The open folder's file tree: directories first, alphabetical. */
  files(): GET<"/api/files", Types.FilesResponse>;
/** GitHub objects rooted in this checkout. Authentication remains in the
   * person's `gh` credential store; these routes never carry a token. */
  githubWorkspace(): GET<"/api/workspace/github", Github.GithubWorkspace>;
  githubPullRequest(number: number): GET<"/api/workspace/github/pr/{number}", Github.GithubPullRequest>;
  githubIssue(repository: string, number: number): GET<"/api/workspace/github/issue/{number}?repository={repository:url}", Github.GithubIssue>;
  associateGithubIssue(repository: string, number: number, folder: string): POST<"/api/workspace/github/issues", { repository: string; number: number; folder: string }>;
  editGithubObject(kind: "pr" | "issue", repository: string, number: number, field: "title" | "body", value: string): POST<"/api/workspace/github/edit", { ok: true }>;
  commentOnGithubObject(kind: "pr" | "issue", repository: string, number: number, body: string): POST<"/api/workspace/github/comment", { ok: true }>;
  githubCheckLog(run: number, job: number): GET<"/api/workspace/github/check-log?run={run}&job={job}", { text: string; truncated: boolean }>;
  markGithubNotificationRead(thread: string): POST<"/api/workspace/github/notification/{thread:url}", { ok: true }>;
/** Show a path in the platform's file manager: a file selected inside its
   * folder, a directory opened. `path` is root-relative; `""` is the folder
   * itself. */
  reveal(path: string): POST<"/api/reveal", { ok: true }>;
/** Open a path in whatever program this machine already opens that kind of
   * file with. Nothing is read or written here — the OS association decides. */
  openExternal(path: string): POST<"/api/open-external", { ok: true }>;
/** Any text file in the folder, whole, with the hash a save passes back. */
  openFile(path: string): POST<"/api/open-file", { path: string; doc_id?: string }>;
  file(path: string): GET<"/api/file?path={path:url}", Types.PlainFile>;
/** One dired verb: rename, move, copy, delete, a new file or folder. */
  fileOp(op: Types.FileOpRequest): POST<"/api/files/op", { op: string; from?: string; to?: string; path?: string }, "op">;
/** Fetch a tool the project needs — today a debug adapter — through the
   * same catalogue and the same confinement `hick dap install` uses. See
   * crates/hickory-cli/src/serve/install.rs. */
  installTool(kind: string, language: string): POST<"/api/install", { installed: string; path: string }>;
/** The platform's own folder chooser, opened at `start`. `{path: null}` is
   * a cancel, which is an answer and not an error. 503 when the program
   * hosting the engine has no dialogs — see crates/hickory-cli/src/serve/shell.rs. */
  pickFolder(start: string): POST<"/api/pick-folder", { path: string | null }>;
  saveFileDialog(name: string): POST<"/api/save-file-dialog", { path: string | null }>;
/** Close the native window after the page has settled dirty buffers. */
  closeWindow(): POST<"/api/window/close", { ok: true }>;
/** Clear a native application-quit request after its close flow was cancelled. */
  cancelQuit(): POST<"/api/window/cancel-quit", { ok: true }>;
/** How a started scaffold ended. The terminal tells the person; this is
   * how the app finds out, so the tree and the history pane can catch up. */
  scaffoldResult(session: string): GET<"/api/scaffold/result?session={session:url}", Types.ScaffoldResult>;
</storycode:copy>

## Routes documents

Document inspection, refactoring and formula requests share the ordinary
transport. Their response types remain references to the existing application
types; no second response schema is maintained here.

<storycode:copy id="api-codegen-routes-documents">
/** Pin the document's current woven outputs as a refactor baseline. */
  refactorBegin(docId: string): POST<"/api/docs/{docId}/refactor/begin", Types.RefactorStatus>;
/** The live equivalence verdict against the pinned baseline. */
  refactorStatus(docId: string): GET<"/api/docs/{docId}/refactor/status", Types.RefactorStatus>;
/** Drop the baseline. */
  refactorEnd(docId: string): POST<"/api/docs/{docId}/refactor/end", Types.RefactorStatus>;
  outputs(docId: string): GET<"/api/docs/{docId}/outputs", Types.OutputsResponse>;
/** Context provenance: every agent write in this document and what was in
   * front of the model when it happened. */
  context(docId: string): GET<"/api/docs/{docId}/context", Types.ContextResponse>;
/** A session file as the conversation it records (turns, steps, tree). */
  sessionView(path: string): GET<"/api/sessions/view?path={path:url}", Types.SessionViewResponse>;
/** Declared provenance: every `cites=` in the document, resolved. */
  cites(docId: string): GET<"/api/docs/{docId}/cites", Types.CitesResponse>;
  outputFile(docId: string, path: string): GET<"/api/docs/{docId}/outputs/file?path={path:url}", Types.OutputFile>;
  editOutput(docId: string, path: string, edits: Types.OutputEdit[]): POST<"/api/docs/{docId}/outputs/edit", Types.OutputEditResponse>;
/** The vocabulary the server draws, as data. */
  elements(): GET<"/api/elements", { elements: Types.ElementDescription[] }>;
  runStatus(runId: string): GET<"/api/runs/{runId}", Types.Run>;
  executor(): GET<"/api/executor", Types.ExecutorInfo>;
/** The agent's LLM API keys: configured-or-not plus a masked hint. The
   * full key never travels back — see SettingsKeysResponse. */
  settingsKeys(): GET<"/api/settings/keys", Types.SettingsKeysResponse>;
/** Set/clear only the named providers (string sets, null clears). */
  saveSettingsKeys(patch: Types.SettingsKeysPatch): PUT<"/api/settings/keys", Types.SettingsKeysResponse, "patch">;
/** UI settings the server persists (ui.json): the custom window title. */
  settingsUi(): GET<"/api/settings/ui", Types.UiSettings>;
  saveSettingsUi(settings: Partial<Types.UiSettings>): PUT<"/api/settings/ui", Types.UiSettings, "settings">;
/** Definitions and references across this session's generated files. */
  structure(): GET<"/api/structure", Types.StructureResponse>;
/** Every formula in a grid, computed.
   *
   * The host resolves references and works out the order before any backend
   * is asked anything, so this is one call whatever language the formulas
   * are in. The backend installs itself on first use — it is a script the
   * binary carries, not a download. */
  evaluateFormulas(language: string, rows: string[][]): POST<"/api/formula/evaluate", Types.FormulaResults>;
/** The same evaluation, cell by cell — what the table's debugger steps
   * through. The same code path on the host, so the steps can never disagree
   * with the values the grid is showing. */
  traceFormulas(language: string, rows: string[][]): POST<"/api/formula/trace", Types.FormulaTrace>;
/** Languages this machine can evaluate formulas in right now. */
  formulaLanguages(): GET<"/api/formula/languages", { languages: string[] }>;
</storycode:copy>

## Routes git

Git operations preserve their original request bodies. `$empty` deliberately
sends `{}` for push and pull; a route with no remaining fields otherwise sends
no body. This distinction must survive generation.

<storycode:copy id="api-codegen-routes-git">
/** The branch, and whether anything is uncommitted. */
  gitStatus(): GET<"/api/git/status", Types.GitStatus>;
/** One commit as a card: its diff, and which of its files were edited
   * since. Asked on expand by the history lens. */
  gitCommitDetail(sha: string): GET<"/api/git/commit?sha={sha:url}", Types.GitCommitDetail>;
// The history lens's verbs (lenses.md steps 4–6). Each is one git
// operation run as itself on the server; a refusal comes back in git's
// words, and a join that stops on a conflict comes back as 409.
/** Run a recipe commit's recipe again and join the result. */
  gitReplay(sha: string): POST<"/api/git/replay", Types.GitReplay>;
/** The tail: run a command in a clean worktree at HEAD and commit its
   * output under `output/` with the recipe. */
  gitRecipe(command: string, output: string): POST<"/api/git/recipe", Types.GitRecipeRun>;
  gitReword(sha: string, message: string): POST<"/api/git/reword", { head: string }>;
  gitDrop(sha: string): POST<"/api/git/drop", { head: string }>;
  gitMove(sha: string, direction: "earlier" | "later"): POST<"/api/git/move", { head: string }>;
// The git pane's verbs — each one git command, run as itself, with git's
// own words when it refuses. See crates/hickory-cli/src/serve/git_ops.rs.
/** Produced files whose disk bytes are not the document's, and why. */
  divergedOutputs(): GET<"/api/outputs/diverged", Types.DivergedOutputs>;
/** Overwrite a diverged file with what its document produces. */
  regenerateOutput(path: string, expected_current: string, expected_produced: string): POST<"/api/v1/outputs/regenerate", { ok: boolean; path: string; applied: boolean; carried_back: boolean }>;
/** Write the bytes a person chose — a merge's result — over a diverged file. */
  resolveOutput(path: string, content: string, expected_current: string, expected_produced: string): POST<"/api/v1/outputs/resolve", { ok: boolean; path: string; applied: boolean; carried_back: boolean }>;
  gitChanges(): GET<"/api/git/changes", Types.GitChanges>;
  gitStage(body: { paths?: string[]; all?: boolean }): POST<"/api/git/stage", Types.GitSaid, "body">;
  gitUnstage(body: { paths?: string[]; all?: boolean }): POST<"/api/git/unstage", Types.GitSaid, "body">;
  gitDiscard(paths: string[]): POST<"/api/git/discard", Types.GitSaid>;
  gitPush(): POST<"/api/git/push", Types.GitSaid, "$empty">;
  gitPull(): POST<"/api/git/pull", Types.GitSaid, "$empty">;
  gitBranches(): GET<"/api/git/branches", { branches: Types.GitBranch[] }>;
  gitStash(action: "push" | "pop"): POST<"/api/git/stash", Types.GitSaid>;
/** Which commits on this branch are still drafts. */
  gitFloor(): GET<"/api/git/floor", { repository: boolean; floor?: Types.PublicationFloor }>;
/** Whether `.md` merges go through hick in this clone. Asked at project
   * open, because a clone that never ran `hick init` has neither the driver
   * nor the pre-commit hook that would report it missing. */
  mergeDriver(): GET<"/api/git/merge-driver", { status: Types.MergeDriverStatus; ok: boolean }>;
/** Run `hick init` on the open folder — the banner's button. */
  initRepository(): POST<"/api/git/merge-driver", Types.InitOutcome>;
/** What a merged view can be opened over. */
  worktrees(): GET<"/api/worktrees", { repository: boolean; worktrees: Types.WorktreeInfo[] }>;
</storycode:copy>

## Routes fleet

Fleet pairing and continuity routes retain their existing wire paths.
Requests with keepalive or conditional protocol selection stay in the manual
client because the route vocabulary does not express those decisions.

<storycode:copy id="api-codegen-routes-fleet">
/** This machine, and the machines paired with it. Identity, not
   * connectivity: nothing here can be attached to yet. */
  fleet(): GET<"/api/fleet", Types.FleetResponse>;
/** A phrase to read out, before hosting it. */
  fleetPhrase(): GET<"/api/fleet/phrase", { phrase: string; seconds: number; note: string }>;
  startFleetPairing(window: string, request_id: string, mode: "host" | "dial", phrase: string): POST<"/api/v1/fleet/pairings", { started: boolean; job: Fleet.FleetPairingJob }>;
  fleetPairings(window: string): GET<"/api/v1/fleet/pairings?window={window:url}", { jobs: Fleet.FleetPairingJob[] }>;
  fleetPairing(window: string, id: string): GET<"/api/v1/fleet/pairings/{id:url}?window={window:url}", Fleet.FleetPairingJob>;
  forgetFleetPairing(window: string, id: string): DELETE<"/api/v1/fleet/pairings/{id:url}?window={window:url}", { forgotten: string }>;
/** Wait for the other machine to dial this phrase. Blocks for the window. */
  fleetHost(phrase: string): POST<"/api/fleet/host", { machine: Types.FleetMachine; their_fingerprint: string; our_fingerprint: string }>;
/** Dial a phrase the other machine printed. */
  fleetPair(phrase: string): POST<"/api/fleet/pair", {
      machine: Types.FleetMachine;
      their_fingerprint: string;
      our_fingerprint: string;
      note: string;
    }>;
/** Enrol the machine an invitation names. */
  fleetAccept(invitation: string): POST<"/api/fleet/accept", { machine: Types.FleetMachine }>;
/** Is continuity on for this project? Off by default. */
  continuity(): GET<"/api/settings/continuity", Types.ContinuitySettings>;
  setContinuity(enabled: boolean): PUT<"/api/settings/continuity", { enabled: boolean }>;
</storycode:copy>

## Routes sessions

History, agent and terminal operations pass named parameters through without
changing transport policy. Workspace drafts use the entire `draft` argument as
the body, rather than wrapping it in another object.

<storycode:copy id="api-codegen-routes-sessions">
/** The commits this document's time slider can stop at, newest first. */
  docHistory(docId: string): GET<"/api/docs/{docId}/history", { repository: boolean; commits: Types.ReplayCommit[] }>;
/** Who last touched each line. Answers `{lines: []}` for a folder that is
   * not a repository, an untracked file, or a machine with no git — the
   * column is an optional annotation, never a reason a file will not open. */
  blame(path: string): GET<"/api/blame?path={path:url}", { path: string; lines: Types.BlameLine[] }>;
/** Stop the agent turn running on this document. The run halts at its
   * next seam — mid-stream included, which is what stops the token spend —
   * and finishes with status "stopped" on the run channel. */
  agentStop(docId: string): POST<"/api/docs/{docId}/agent/stop", { stopping: string }>;
  agentTurns(docId: string): GET<"/api/docs/{docId}/agent/turns", Types.AgentTurnsResponse>;
/** Every terminal session, and the one attention queue across them. The
   * order of `attention` is the server's — see hick_term::attention — so the
   * queue, the session tree, and ⌘J cannot disagree about what is next. */
  terminals(): GET<"/api/terminals", Types.TerminalsResponse>;
/** Run one test (or a file's tests) in a terminal session named after it.
   * See crates/hickory-cli/src/serve/test_run.rs. */
  runTest(body: { path: string; name: string; language: string; expected_source_hash?: string }): POST<"/api/tests/run", Types.TerminalSession & { source_hash: string; source_checked_at_admission: boolean; source_frozen: false }, "body">;
  closeTerminal(id: string): DELETE<"/api/terminals/{id}", void>;
  terminalResize(id: string, rows: number, cols: number): POST<"/api/terminals/{id}/resize", void>;
  interruptTerminal(id: string): POST<"/api/terminals/{id}/interrupt", void>;
/** Answer the attention card: writes, and clears the prompt in the same
   * request, so the session leaves the queue immediately rather than on the
   * next poll. */
  answerTerminal(id: string, send: string): POST<"/api/terminals/{id}/answer", Types.TerminalSession>;
  setTurbo(enabled: boolean): PUT<"/api/terminals/turbo", { turbo: boolean }>;
/** Which terminals are writing into which documents, all at once — "never
   * anchor silently" means the state has to be readable at any moment, not
   * announced once when it changes. */
  terminalAnchors(): GET<"/api/terminals/anchors", { anchors: Record<string, Types.TerminalAnchor> }>;
  anchorTerminal(id: string, doc: string, container: string): POST<"/api/terminals/{id}/anchor", Types.TerminalAnchor>;
  unanchorTerminal(id: string): DELETE<"/api/terminals/{id}/anchor", void>;
/** Record again after a suspension. Always a new cell. */
  resumeAnchor(id: string): POST<"/api/terminals/{id}/anchor/resume", Types.TerminalAnchor>;
// --- What the window remembers between runs -----------------------------
//
// Stored under the user's own data directory, never in the project: git
// cannot reach it by construction. See api.md, "Workspace state and
// drafts".
/** The stored window layout, or null on a project opened for the first
   * time (and on a damaged file, which reads as "no layout" rather than as
   * a reason not to start). */
  workspaceUi(): GET<"/api/workspace/ui", { state: Types.WorkspaceUiState }>;
  responseDraft(doc: string): GET<"/api/workspace/responses?doc={doc:url}", { scope: string; state: unknown }>;
/** Every buffer that had unsaved changes when the app last closed, each
   * carrying the bytes it was taken from so a file that moved on can be
   * merged rather than fought over. */
  drafts(): GET<"/api/workspace/drafts", { drafts: Types.WorkspaceDraft[] }>;
  saveDraft(draft: Types.WorkspaceDraft): PUT<"/api/workspace/drafts", { ok: true }, "draft">;
  discardDraft(path: string): DELETE<"/api/workspace/drafts?path={path:url}", { ok: true }>;
}
</storycode:copy>

<storycode:file path="apps/web/src/api/routes.ts"><storycode:paste select="#api-codegen-route-vocabulary"/><storycode:paste select="#api-codegen-routes-projects"/><storycode:paste select="#api-codegen-routes-documents"/><storycode:paste select="#api-codegen-routes-git"/><storycode:paste select="#api-codegen-routes-fleet"/><storycode:paste select="#api-codegen-routes-sessions"/></storycode:file>

The isolated model reads this declaration and its imported types, rather than
materializing the entire frontend and its third-party declarations. This extends
the application compiler settings so name resolution uses the same rules.

<storycode:entry path="apps/web/src/api/route-model/tsconfig.json">
{"extends":"../../../tsconfig.json","include":["../routes.ts"]}
</storycode:entry>

</storycode:doc>
