import type {
  AdoptResponse,
  AgentTurnsResponse,
  BlameLine,
  BlockActionOutcome,
  CitesResponse,
  ContextResponse,
  ContinuitySettings,
  DivergedOutputs,
  Doc,
  DocSummary,
  ElementDescription,
  ExecutorInfo,
  FileOpRequest,
  FilesResponse,
  FindOptions,
  FindResponse,
  FleetMachine,
  FleetResponse,
  FormulaResults,
  FormulaTrace,
  GitBranch,
  GitChanges,
  GitCommitDetail,
  GitCommitResult,
  GitDiff,
  GitLog,
  GitRecipeRun,
  GitReplay,
  GitSaid,
  GitStatus,
  InitOutcome,
  MergeDriverStatus,
  MergedViewResponse,
  OpenTerminal,
  OutputEdit,
  OutputEditResponse,
  OutputFile,
  OutputsResponse,
  PlainFile,
  PlainFileSaved,
  Project,
  ProjectSuggestion,
  PublicationFloor,
  RefactorStatus,
  RenderResponse,
  ReplaceResponse,
  ReplayCommit,
  ReplayResponse,
  Run,
  SampleCreated,
  SavedAsset,
  ScaffoldCatalog,
  ScaffoldPreview,
  ScaffoldResult,
  ScaffoldSpec,
  ScaffoldStarted,
  ScaffoldTemplateDetail,
  ScratchpadSaved,
  SearchResponse,
  SessionViewResponse,
  SettingsKeysPatch,
  SettingsKeysResponse,
  StructureResponse,
  TerminalAnchor,
  TerminalSession,
  TerminalsResponse,
  UiSettings,
  WorkspaceDraft,
  WorkspaceUiState,
  WorktreeInfo,
} from "./types";
import type { GithubIssue, GithubPullRequest, GithubWorkspace } from "./github";

export const MOCK = import.meta.env.VITE_MOCK === "1";

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    /** Parsed JSON error body, when the server sent one (e.g. the 422
     * synthetic-range payload from POST /outputs/edit). */
    public body?: unknown,
  ) {
    super(message);
  }
}

/** The longest error body worth putting in front of a person as a sentence.
 * Past this it is a page, not a message, and the console has the whole of it. */
const ERROR_TEXT_LIMIT = 600;

/**
 * A failed response, read for everything it actually says.
 *
 * The server's own refusals are `{"error": "…"}` and were always read. What
 * was not, and what cost a real debugging session, is everything that answers
 * a request **before** a handler runs: axum's own extractor rejections are
 * `text/plain`, so a 422 saying `missing field \`image\`` — the whole
 * diagnosis, sitting right there in the body — was thrown away and shown as
 * the HTTP status text, "Unprocessable Entity". A message that names no
 * field, no value and no route is not an error message.
 *
 * So: the JSON shape first, then the body's own text, and only then the
 * status line — which is the honest answer for a body that is genuinely
 * empty. `user-facing-errors`: an error says which check failed.
 */
async function apiError(res: Response): Promise<ApiError> {
  const raw = await res.text().catch(() => "");
  let message = "";
  let errBody: unknown;
  try {
    const data = JSON.parse(raw);
    errBody = data;
    if (typeof data?.error === "string") message = data.error;
    else if (typeof data?.message === "string") message = data.message;
  } catch {
    /* Not JSON. The text is the message. */
  }
  if (!message) {
    const text = raw.trim();
    message =
      text.length > ERROR_TEXT_LIMIT
        ? `${text.slice(0, ERROR_TEXT_LIMIT)}…`
        : text;
  }
  return new ApiError(res.status, message || res.statusText, errBody);
}

type MockHandler = (
  method: string,
  path: string,
  body: unknown,
) => Promise<unknown>;

let mockHandler: MockHandler | null = null;

/** Installed by src/mock/mockApi.ts when VITE_MOCK=1. */
export function installMockHandler(h: MockHandler) {
  mockHandler = h;
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  if (mockHandler) {
    return (await mockHandler(method, path, body)) as T;
  }
  const headers: Record<string, string> = {};
  if (body !== undefined) headers["Content-Type"] = "application/json";
  const res = await fetch(path, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!res.ok) {
    throw await apiError(res);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

// Rendering a document is the most expensive thing the API does, and the app
// asks for it from several independent places (initial load, save, run
// events, lineage edits). Concurrent requests for the SAME document are
// collapsed onto one in-flight promise: a render started 10 ms ago cannot
// have a different answer from one started now, and the callers all want the
// same value. The entry is cleared as soon as it settles, so the next call
// always fetches fresh state.
const inFlightRenders = new Map<string, Promise<RenderResponse>>();

function dedupedRender(id: string): Promise<RenderResponse> {
  const existing = inFlightRenders.get(id);
  if (existing) return existing;
  const p = request<RenderResponse>("GET", `/api/docs/${id}/render`).finally(
    () => {
      if (inFlightRenders.get(id) === p) inFlightRenders.delete(id);
    },
  );
  inFlightRenders.set(id, p);
  return p;
}

export const api = {
  projects: () => request<Project[]>("GET", "/api/projects"),
  createProject: (name: string, visibility: "public" | "private") =>
    request<Project>("POST", "/api/projects", { name, visibility }),
  projectDocs: (projectId: string) =>
    request<DocSummary[]>("GET", `/api/projects/${projectId}/docs`),
  createDoc: (projectId: string, path: string, source: string) =>
    request<Doc>("POST", `/api/projects/${projectId}/docs`, { path, source }),

  doc: (id: string) => request<Doc>("GET", `/api/docs/${id}`),
  saveDoc: (id: string, source: string) =>
    request<Doc>("PUT", `/api/docs/${id}`, { source }),
  render: (id: string) => dedupedRender(id),

  /** The open folder's file tree: directories first, alphabetical. */
  files: () => request<FilesResponse>("GET", "/api/files"),

  /** GitHub objects rooted in this checkout. Authentication remains in the
   * person's `gh` credential store; these routes never carry a token. */
  githubWorkspace: () => request<GithubWorkspace>("GET", "/api/workspace/github"),
  githubPullRequest: (number: number) =>
    request<GithubPullRequest>("GET", `/api/workspace/github/pr/${number}`),
  githubIssue: (repository: string, number: number) =>
    request<GithubIssue>(
      "GET",
      `/api/workspace/github/issue/${number}?repository=${encodeURIComponent(repository)}`,
    ),
  associateGithubIssue: (repository: string, number: number, folder: string) =>
    request<{ repository: string; number: number; folder: string }>(
      "POST",
      "/api/workspace/github/issues",
      { repository, number, folder },
    ),
  editGithubObject: (
    kind: "pr" | "issue",
    repository: string,
    number: number,
    field: "title" | "body",
    value: string,
  ) => request<{ ok: true }>("POST", "/api/workspace/github/edit", {
    kind, repository, number, field, value,
  }),
  commentOnGithubObject: (
    kind: "pr" | "issue",
    repository: string,
    number: number,
    body: string,
  ) => request<{ ok: true }>("POST", "/api/workspace/github/comment", {
    kind, repository, number, body,
  }),
  githubCheckLog: (run: number, job: number) =>
    request<{ text: string; truncated: boolean }>(
      "GET",
      `/api/workspace/github/check-log?run=${run}&job=${job}`,
    ),
  markGithubNotificationRead: (thread: string) =>
    request<{ ok: true }>("POST", `/api/workspace/github/notification/${encodeURIComponent(thread)}`),

  /** Show a path in the platform's file manager: a file selected inside its
   * folder, a directory opened. `path` is root-relative; `""` is the folder
   * itself. */
  reveal: (path: string) =>
    request<{ ok: true }>("POST", "/api/reveal", { path }),
  /** Open a path in whatever program this machine already opens that kind of
   * file with. Nothing is read or written here — the OS association decides. */
  openExternal: (path: string) =>
    request<{ ok: true }>("POST", "/api/open-external", { path }),

  /** Any text file in the folder, whole, with the hash a save passes back. */
  file: (path: string) =>
    request<PlainFile>("GET", `/api/file?path=${encodeURIComponent(path)}`),
  /** Adopt a plain file into a literate document, byte-exactly: the server
   * verifies the new document weaves the file's exact bytes before writing
   * anything. `into` appends to an existing document instead of creating
   * `<stem>.hick` beside the file. */
  /** One dired verb: rename, move, copy, delete, a new file or folder. */
  fileOp: (op: FileOpRequest) =>
    request<{ op: string; from?: string; to?: string; path?: string }>(
      "POST",
      "/api/files/op",
      op,
    ),
  adopt: (path: string, into?: string) =>
    request<AdoptResponse>("POST", "/api/adopt", {
      path,
      ...(into !== undefined ? { into } : {}),
    }),

  /** Fetch a tool the project needs — today a debug adapter — through the
   * same catalogue and the same confinement `hick dap install` uses. See
   * crates/hickory-cli/src/serve/install.rs. */
  installTool: (kind: string, language: string) =>
    request<{ installed: string; path: string }>("POST", "/api/install", {
      kind,
      language,
    }),

  /** The platform's own folder chooser, opened at `start`. `{path: null}` is
   * a cancel, which is an answer and not an error. 503 when the program
   * hosting the engine has no dialogs — see crates/hickory-cli/src/serve/shell.rs. */
  pickFolder: (start: string) =>
    request<{ path: string | null }>("POST", "/api/pick-folder", { start }),
  saveFileDialog: (name: string) =>
    request<{ path: string | null }>("POST", "/api/save-file-dialog", { name }),
  /** Close the native window after the page has settled dirty buffers. */
  closeWindow: () => request<{ ok: true }>("POST", "/api/window/close"),

  /** What this machine can scaffold: every `dotnet new` template its SDK
   * has. 422 with `{missing: "dotnet"}` when there is no SDK at all — the
   * dialog keys off that field, never off the sentence.
   * See crates/hickory-cli/src/serve/scaffold.rs. */
  scaffoldTemplates: (toolchain?: string) =>
    request<ScaffoldCatalog>(
      "GET",
      "/api/scaffold/templates" +
        (toolchain ? `?toolchain=${encodeURIComponent(toolchain)}` : ""),
    ),

  /** One template's options, as fields. A second `dotnet` process, so it is
   * asked for only once a template is chosen. */
  scaffoldOptions: (template: string, language?: string | null, toolchain?: string) =>
    request<ScaffoldTemplateDetail>(
      "GET",
      `/api/scaffold/options?template=${encodeURIComponent(template)}` +
        (language ? `&language=${encodeURIComponent(language)}` : "") +
        (toolchain ? `&toolchain=${encodeURIComponent(toolchain)}` : ""),
    ),

  /** The exact bytes New Project would write, without writing them. The same
   * function that writes them, called over the wire rather than reimplemented
   * here — a preview free to disagree with the file is worse than none. */
  scaffoldPreview: (spec: ScaffoldSpec) =>
    request<ScaffoldPreview>("POST", "/api/scaffold/preview", { ...spec }),
  /** Run the scaffolder in a terminal, and commit what it wrote the moment
   * it exits zero. Answers the session, not the commit: the work is
   * watchable, not finished — `scaffoldResult` says how it ended. The commit
   * carries the recipe in its trailers; see
   * docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md and
   * docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md. */
  scaffoldCreate: (spec: ScaffoldSpec) =>
    request<ScaffoldStarted>("POST", "/api/scaffold", { ...spec }),
  /** How a started scaffold ended. The terminal tells the person; this is
   * how the app finds out, so the tree and the history pane can catch up. */
  scaffoldResult: (session: string) =>
    request<ScaffoldResult>(
      "GET",
      `/api/scaffold/result?session=${encodeURIComponent(session)}`,
    ),

  /** Put a `<hick:sample>` under the cell that generated this file: a window
   * onto a few of its lines, shown in the weave and never stored in the
   * document. `path` is root-relative; the server finds the owning cell.
   * See crates/hickory-cli/src/serve/sample.rs. */
  createSample: (path: string, from: number, to: number, caption?: string) =>
    request<SampleCreated>("POST", "/api/samples", {
      path,
      from,
      to,
      ...(caption ? { caption } : {}),
    }),

  /** Write an image dropped or pasted into a note to disk, into an `assets/`
   * directory beside the document, and answer where it landed. `relative` is
   * what goes in the markdown; `path` is root-relative, for the tree and for
   * `assetUrl`. See crates/hickory-cli/src/serve/asset.rs. */
  saveAsset: (name: string, contentBase64: string, docPath: string | null) =>
    request<SavedAsset>("POST", "/api/asset", {
      name,
      content_base64: contentBase64,
      ...(docPath ? { doc_path: docPath } : {}),
    }),

  /** Save text typed in the app as a note in the open folder.
   *
   * A different act from ingesting a file, and it produces a different note:
   * this is prose a named human typed here, not bytes another tool produced,
   * so it is never wrapped as a transcript. See
   * `docs/specs/freeform/ingest.md`. */
  saveScratchpad: (text: string) =>
    request<ScratchpadSaved>("POST", "/api/scratchpad", { text }),

  /** Pin the document's current woven outputs as a refactor baseline. */
  refactorBegin: (docId: string) =>
    request<RefactorStatus>("POST", `/api/docs/${docId}/refactor/begin`),
  /** The live equivalence verdict against the pinned baseline. */
  refactorStatus: (docId: string) =>
    request<RefactorStatus>("GET", `/api/docs/${docId}/refactor/status`),
  /** Drop the baseline. */
  refactorEnd: (docId: string) =>
    request<RefactorStatus>("POST", `/api/docs/${docId}/refactor/end`),

  /** Save a plain file whole. `baseHash` is the hash the content was loaded
   * under — a mismatch means the disk moved and the server answers 409.
   * `force` is the deliberate overwrite after that 409. */
  saveFile: (path: string, content: string, baseHash?: string, force = false) =>
    request<PlainFileSaved>("PUT", "/api/file", {
      path,
      content,
      ...(baseHash !== undefined ? { base_hash: baseHash } : {}),
      ...(force ? { force } : {}),
    }),

  outputs: (docId: string) =>
    request<OutputsResponse>("GET", `/api/docs/${docId}/outputs`),
  /** Context provenance: every agent write in this document and what was in
   * front of the model when it happened. */
  context: (docId: string) =>
    request<ContextResponse>("GET", `/api/docs/${docId}/context`),
  /** A session file as the conversation it records (turns, steps, tree). */
  sessionView: (path: string) =>
    request<SessionViewResponse>("GET", `/api/sessions/view?path=${encodeURIComponent(path)}`),
  /** Declared provenance: every `cites=` in the document, resolved. */
  cites: (docId: string) =>
    request<CitesResponse>("GET", `/api/docs/${docId}/cites`),
  outputFile: (docId: string, path: string) =>
    request<OutputFile>(
      "GET",
      `/api/docs/${docId}/outputs/file?path=${encodeURIComponent(path)}`,
    ),
  editOutput: (docId: string, path: string, edits: OutputEdit[]) =>
    request<OutputEditResponse>("POST", `/api/docs/${docId}/outputs/edit`, {
      path,
      edits,
    }),

  /** The vocabulary the server draws, as data. */
  elements: () => request<{ elements: ElementDescription[] }>("GET", "/api/elements"),
  /** One action on the element whose tag starts at byte `at`. The element
   * says what it wants and the server carries it out; a run comes back as
   * a run id on the same channel `run` uses. */
  blockAction: (docId: string, at: number, action: string, body?: unknown) =>
    request<BlockActionOutcome>(
      "POST",
      `/api/docs/${docId}/blocks/${at}/${encodeURIComponent(action)}`,
      body ?? {},
    ),
  run: (docId: string, cells?: string[]) =>
    request<{ run_id: string }>(
      "POST",
      `/api/docs/${docId}/run`,
      cells ? { cells } : {},
    ),
  runStatus: (runId: string) => request<Run>("GET", `/api/runs/${runId}`),
  check: (docId: string) =>
    request<{ run_id: string }>("POST", `/api/docs/${docId}/check`),

  executor: () => request<ExecutorInfo>("GET", "/api/executor"),

  /** The agent's LLM API keys: configured-or-not plus a masked hint. The
   * full key never travels back — see SettingsKeysResponse. */
  settingsKeys: () =>
    request<SettingsKeysResponse>("GET", "/api/settings/keys"),
  /** Set/clear only the named providers (string sets, null clears). */
  saveSettingsKeys: (patch: SettingsKeysPatch) =>
    request<SettingsKeysResponse>("PUT", "/api/settings/keys", patch),

  /** UI settings the server persists (ui.json): the custom window title. */
  settingsUi: () => request<UiSettings>("GET", "/api/settings/ui"),
  saveSettingsUi: (settings: Partial<UiSettings>) =>
    request<UiSettings>("PUT", "/api/settings/ui", settings),

  /** Definitions and references across this session's generated files. */
  structure: () => request<StructureResponse>("GET", "/api/structure"),
  /** What this project calls things: identifiers from its own text, ranked
   * by how often they are used and — when the local model is installed — by
   * how close their surroundings are to what is being typed. A different
   * question from the language server's, shown beside it rather than instead
   * of it. */
  complete: (prefix: string, context: string, k = 8) =>
    request<{ suggestions: ProjectSuggestion[] }>(
      "GET",
      `/api/complete?prefix=${encodeURIComponent(prefix)}` +
        `&context=${encodeURIComponent(context.slice(-600))}&k=${k}`,
    ),

  /** Every formula in a grid, computed.
   *
   * The host resolves references and works out the order before any backend
   * is asked anything, so this is one call whatever language the formulas
   * are in. The backend installs itself on first use — it is a script the
   * binary carries, not a download. */
  evaluateFormulas: (language: string, rows: string[][]) =>
    request<FormulaResults>("POST", "/api/formula/evaluate", {
      language,
      rows,
    }),

  /** The same evaluation, cell by cell — what the table's debugger steps
   * through. The same code path on the host, so the steps can never disagree
   * with the values the grid is showing. */
  traceFormulas: (language: string, rows: string[][]) =>
    request<FormulaTrace>("POST", "/api/formula/trace", { language, rows }),

  /** Languages this machine can evaluate formulas in right now. */
  formulaLanguages: () =>
    request<{ languages: string[] }>("GET", "/api/formula/languages"),

  /** The commit graph, with every commit's files and line counts — one git
   * invocation, so expanding a row costs nothing. The verbs that change the
   * repository are below, each one git command run as itself. */
  gitLog: (limit = 120, path?: string) =>
    request<GitLog>(
      "GET",
      `/api/git/log?limit=${limit}` +
        (path ? `&path=${encodeURIComponent(path)}` : ""),
    ),

  /** The branch, and whether anything is uncommitted. */
  gitStatus: () => request<GitStatus>("GET", "/api/git/status"),
  /** One commit as a card: its diff, and which of its files were edited
   * since. Asked on expand by the history lens. */
  gitCommitDetail: (sha: string) =>
    request<GitCommitDetail>("GET", `/api/git/commit?sha=${encodeURIComponent(sha)}`),
  // The history lens's verbs (lenses.md steps 4–6). Each is one git
  // operation run as itself on the server; a refusal comes back in git's
  // words, and a join that stops on a conflict comes back as 409.
  /** Run a recipe commit's recipe again and join the result. */
  gitReplay: (sha: string) => request<GitReplay>("POST", "/api/git/replay", { sha }),
  /** The tail: run a command in a clean worktree at HEAD and commit its
   * output under `output/` with the recipe. */
  gitRecipe: (command: string, output: string) =>
    request<GitRecipeRun>("POST", "/api/git/recipe", { command, output }),
  gitReword: (sha: string, message: string) =>
    request<{ head: string }>("POST", "/api/git/reword", { sha, message }),
  gitDrop: (sha: string) => request<{ head: string }>("POST", "/api/git/drop", { sha }),
  gitMove: (sha: string, direction: "earlier" | "later") =>
    request<{ head: string }>("POST", "/api/git/move", { sha, direction }),

  // The git pane's verbs — each one git command, run as itself, with git's
  // own words when it refuses. See crates/hickory-cli/src/serve/git_ops.rs.
  /** Produced files whose disk bytes are not the document's, and why. */
  divergedOutputs: () => request<DivergedOutputs>("GET", "/api/outputs/diverged"),
  /** Overwrite a diverged file with what its document produces. */
  regenerateOutput: (path: string) =>
    request<{ ok: boolean; path: string }>("POST", "/api/outputs/regenerate", { path }),
  /** Write the bytes a person chose — a merge's result — over a diverged file. */
  resolveOutput: (path: string, content: string) =>
    request<{ ok: boolean; path: string }>("POST", "/api/outputs/resolve", { path, content }),
  gitChanges: () => request<GitChanges>("GET", "/api/git/changes"),
  gitDiff: (path: string, staged = false) =>
    request<GitDiff>(
      "GET",
      `/api/git/diff?path=${encodeURIComponent(path)}&staged=${staged ? "true" : "false"}`,
    ),
  gitStage: (body: { paths?: string[]; all?: boolean }) =>
    request<GitSaid>("POST", "/api/git/stage", body),
  gitUnstage: (body: { paths?: string[]; all?: boolean }) =>
    request<GitSaid>("POST", "/api/git/unstage", body),
  gitDiscard: (paths: string[]) => request<GitSaid>("POST", "/api/git/discard", { paths }),
  gitCommit: (message: string, amend = false) =>
    request<GitCommitResult>("POST", "/api/git/commit", { message, amend }),
  gitPush: () => request<GitSaid>("POST", "/api/git/push", {}),
  gitPull: () => request<GitSaid>("POST", "/api/git/pull", {}),
  gitBranches: () => request<{ branches: GitBranch[] }>("GET", "/api/git/branches"),
  gitCheckout: (branch: string, create = false) =>
    request<GitSaid>("POST", "/api/git/checkout", { branch, create }),
  gitStash: (action: "push" | "pop") => request<GitSaid>("POST", "/api/git/stash", { action }),

  /** Which commits on this branch are still drafts. */
  gitFloor: () =>
    request<{ repository: boolean; floor?: PublicationFloor }>(
      "GET",
      "/api/git/floor",
    ),

  /** Whether `.hick` merges go through hick in this clone. Asked at project
   * open, because a clone that never ran `hick init` has neither the driver
   * nor the pre-commit hook that would report it missing. */
  mergeDriver: () =>
    request<{ status: MergeDriverStatus; ok: boolean }>(
      "GET",
      "/api/git/merge-driver",
    ),
  /** Run `hick init` on the open folder — the banner's button. */
  initRepository: () => request<InitOutcome>("POST", "/api/git/merge-driver"),

  /** What a merged view can be opened over. */
  worktrees: () =>
    request<{ repository: boolean; worktrees: WorktreeInfo[] }>(
      "GET",
      "/api/worktrees",
    ),

  /** One file as it exists in several worktrees at once. Read-only: the
   * alignment is where the risk lives, so it is proved before anything writes
   * through it. */
  merged: (path: string, targets?: string[]) =>
    request<MergedViewResponse>(
      "GET",
      `/api/merged?path=${encodeURIComponent(path)}` +
        (targets?.length ? `&targets=${encodeURIComponent(targets.join(","))}` : ""),
    ),

  /** This machine, and the machines paired with it. Identity, not
   * connectivity: nothing here can be attached to yet. */
  fleet: () => request<FleetResponse>("GET", "/api/fleet"),

  /** A phrase to read out, before hosting it. */
  fleetPhrase: () =>
    request<{ phrase: string; seconds: number; note: string }>(
      "GET",
      "/api/fleet/phrase",
    ),

  /** Wait for the other machine to dial this phrase. Blocks for the window. */
  fleetHost: (phrase: string) =>
    request<{ machine: FleetMachine; their_fingerprint: string; our_fingerprint: string }>(
      "POST",
      "/api/fleet/host",
      { phrase },
    ),

  /** Dial a phrase the other machine printed. */
  fleetPair: (phrase: string) =>
    request<{
      machine: FleetMachine;
      their_fingerprint: string;
      our_fingerprint: string;
      note: string;
    }>("POST", "/api/fleet/pair", { phrase }),

  /** The line another machine accepts to enrol this one. */
  fleetInvite: (phone = false) =>
    request<{ invitation: string; fingerprint: string; note: string }>(
      "GET",
      `/api/fleet/invite${phone ? "?phone=true" : ""}`,
    ),

  /** Enrol the machine an invitation names. */
  fleetAccept: (invitation: string) =>
    request<{ machine: FleetMachine }>("POST", "/api/fleet/accept", { invitation }),

  /** Give or take one verb for one machine. */
  fleetGrant: (machine: string, grant: string, on: boolean) =>
    request<{ machine: FleetMachine }>("PUT", "/api/fleet/grant", {
      machine,
      grant,
      on,
    }),

  /** Revocation, which is deleting a key. */
  fleetRemove: (machine: string) =>
    request<{ removed: boolean; note: string }>("POST", "/api/fleet/remove", {
      machine,
    }),

  /** Is continuity on for this project? Off by default. */
  continuity: () =>
    request<ContinuitySettings>("GET", "/api/settings/continuity"),

  setContinuity: (enabled: boolean) =>
    request<{ enabled: boolean }>("PUT", "/api/settings/continuity", { enabled }),

  /** The commits this document's time slider can stop at, newest first. */
  docHistory: (docId: string) =>
    request<{ repository: boolean; commits: ReplayCommit[] }>(
      "GET",
      `/api/docs/${docId}/history`,
    ),

  /** The lineage this document had AT a commit — recomputed by weaving that
   * commit's document, never executing anything. */
  docReplay: (docId: string, commit: string, path?: string) =>
    request<ReplayResponse>(
      "GET",
      `/api/docs/${docId}/replay?commit=${encodeURIComponent(commit)}` +
        (path ? `&path=${encodeURIComponent(path)}` : ""),
    ),

  /** Who last touched each line. Answers `{lines: []}` for a folder that is
   * not a repository, an untracked file, or a machine with no git — the
   * column is an optional annotation, never a reason a file will not open. */
  blame: (path: string) =>
    request<{ path: string; lines: BlameLine[] }>(
      "GET",
      `/api/blame?path=${encodeURIComponent(path)}`,
    ),

  /** EXHAUSTIVE find across the folder, in path order — the one replace is
   * built on. Not `search`, which is ranked top-k: a ranked answer is a
   * sample, and replacing across a sample changes some of the occurrences. */
  find: (q: string, options: FindOptions = {}) =>
    request<FindResponse>(
      "GET",
      `/api/find?q=${encodeURIComponent(q)}` +
        (options.regex ? "&regex=true" : "") +
        (options.case ? "&case=true" : "") +
        (options.whole_word ? "&whole_word=true" : ""),
    ),

  /** Rewrite every match. `paths` narrows it to the files that were ticked;
   * a generated file is refused and reported in `skipped`, because writing
   * to one either loses the edit at the next weave or fights the up-loop. */
  replaceAll: (
    q: string,
    replacement: string,
    options: FindOptions = {},
    paths?: string[],
  ) =>
    request<ReplaceResponse>("POST", "/api/find/replace", {
      q,
      replacement,
      ...options,
      ...(paths ? { paths } : {}),
    }),

  /** Ranked project-wide search over documents and generated files. */
  search: (q: string, k = 20) =>
    request<SearchResponse>(
      "GET",
      `/api/search?q=${encodeURIComponent(q)}&k=${k}`,
    ),

  /** Start a turn. `parentId` continues from that turn — naming an older one
   * forks a branch (rewind) rather than overwriting what followed it.
   * `provider`/`model` set the document's model choice for this and later
   * turns; an empty string clears back to the default, and leaving a field
   * out keeps the current choice. */
  agent: (
    docId: string,
    prompt: string,
    parentId?: string | null,
    provider?: string,
    model?: string,
  ) =>
    request<{ session_id: string }>("POST", `/api/docs/${docId}/agent`, {
      prompt,
      parent_id: parentId ?? null,
      ...(provider !== undefined ? { provider } : {}),
      ...(model !== undefined ? { model } : {}),
    }),

  /** Stop the agent turn running on this document. The run halts at its
   * next seam — mid-stream included, which is what stops the token spend —
   * and finishes with status "stopped" on the run channel. */
  agentStop: (docId: string) =>
    request<{ stopping: string }>("POST", `/api/docs/${docId}/agent/stop`),

  agentTurns: (docId: string) =>
    request<AgentTurnsResponse>("GET", `/api/docs/${docId}/agent/turns`),

  /** Every terminal session, and the one attention queue across them. The
   * order of `attention` is the server's — see hick_term::attention — so the
   * queue, the session tree, and ⌘J cannot disagree about what is next. */
  terminals: () => request<TerminalsResponse>("GET", "/api/terminals"),
  /** Run one test (or a file's tests) in a terminal session named after it.
   * See crates/hickory-cli/src/serve/test_run.rs. */
  runTest: (body: { path: string; name: string; language: string }) =>
    request<TerminalSession>("POST", "/api/tests/run", body),
  openTerminal: (spec: OpenTerminal = {}) =>
    request<TerminalSession>("POST", "/api/terminals", spec),
  closeTerminal: (id: string) =>
    request<void>("DELETE", `/api/terminals/${id}`),
  /** Type into a session. The pane's own keystrokes go over the socket
   * instead; this is for everything else (menus, the card's input line). */
  terminalInput: (id: string, data: string) =>
    request<void>("POST", `/api/terminals/${id}/input`, { data }),
  terminalResize: (id: string, rows: number, cols: number) =>
    request<void>("POST", `/api/terminals/${id}/resize`, { rows, cols }),
  interruptTerminal: (id: string) =>
    request<void>("POST", `/api/terminals/${id}/interrupt`),
  /** Answer the attention card: writes, and clears the prompt in the same
   * request, so the session leaves the queue immediately rather than on the
   * next poll. */
  answerTerminal: (id: string, send: string) =>
    request<TerminalSession>("POST", `/api/terminals/${id}/answer`, { send }),
  setTurbo: (enabled: boolean) =>
    request<{ turbo: boolean }>("PUT", "/api/terminals/turbo", { enabled }),
  /** Which terminals are writing into which documents, all at once — "never
   * anchor silently" means the state has to be readable at any moment, not
   * announced once when it changes. */
  terminalAnchors: () => request<{ anchors: Record<string, TerminalAnchor> }>(
    "GET",
    "/api/terminals/anchors",
  ),
  anchorTerminal: (id: string, doc: string, container: string) =>
    request<TerminalAnchor>("POST", `/api/terminals/${id}/anchor`, { doc, container }),
  unanchorTerminal: (id: string) =>
    request<void>("DELETE", `/api/terminals/${id}/anchor`),
  /** Record again after a suspension. Always a new cell. */
  resumeAnchor: (id: string) =>
    request<TerminalAnchor>("POST", `/api/terminals/${id}/anchor/resume`),

  // --- What the window remembers between runs -----------------------------
  //
  // Stored under the user's own data directory, never in the project: git
  // cannot reach it by construction. See api.md, "Workspace state and
  // drafts".

  /** The stored window layout, or null on a project opened for the first
   * time (and on a damaged file, which reads as "no layout" rather than as
   * a reason not to start). */
  workspaceUi: () =>
    request<{ state: WorkspaceUiState }>("GET", "/api/workspace/ui"),
  saveWorkspaceUi: (state: WorkspaceUiState) =>
    request<{ ok: true }>("PUT", "/api/workspace/ui", { state }),

  /** Every buffer that had unsaved changes when the app last closed, each
   * carrying the bytes it was taken from so a file that moved on can be
   * merged rather than fought over. */
  drafts: () =>
    request<{ drafts: WorkspaceDraft[] }>("GET", "/api/workspace/drafts"),
  saveDraft: (draft: WorkspaceDraft) =>
    request<{ ok: true }>("PUT", "/api/workspace/drafts", draft),
  discardDraft: (path: string) =>
    request<{ ok: true }>(
      "DELETE",
      `/api/workspace/drafts?path=${encodeURIComponent(path)}`,
    ),
};
