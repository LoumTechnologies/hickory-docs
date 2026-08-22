import type {
  AdoptResponse,
  AgentTurnsResponse,
  Doc,
  DocSummary,
  ExecutorInfo,
  FilesResponse,
  OutputEdit,
  OutputEditResponse,
  OutputFile,
  OutputsResponse,
  PlainFile,
  PlainFileSaved,
  Project,
  RefactorStatus,
  RenderResponse,
  SavedAsset,
  Run,
  ScratchpadSaved,
  SearchResponse,
  SettingsKeysPatch,
  SettingsKeysResponse,
  StructureResponse,
  OpenTerminal,
  TerminalSession,
  TerminalsResponse,
  BlameLine,
  FindOptions,
  FormulaResults,
  FormulaTrace,
  GitLog,
  GitStatus,
  ProjectSuggestion,
  FindResponse,
  ReplaceResponse,
  UiSettings,
  WorkspaceDraft,
  WorkspaceUiState,
  ContextResponse,
  CitesResponse,
  SessionViewResponse,
} from "./types";

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
    let message = res.statusText;
    let errBody: unknown;
    try {
      const data = await res.json();
      errBody = data;
      if (typeof data?.error === "string") message = data.error;
      else if (typeof data?.message === "string") message = data.message;
    } catch {
      /* non-JSON error body */
    }
    throw new ApiError(res.status, message, errBody);
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
  adopt: (path: string, into?: string) =>
    request<AdoptResponse>("POST", "/api/adopt", {
      path,
      ...(into !== undefined ? { into } : {}),
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
  saveSettingsUi: (settings: UiSettings) =>
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
   * invocation, so expanding a row costs nothing. Read-only: changing a
   * repository is a thing people rightly do deliberately, and there is a
   * terminal on every row of the tree. */
  gitLog: (limit = 120, path?: string) =>
    request<GitLog>(
      "GET",
      `/api/git/log?limit=${limit}` +
        (path ? `&path=${encodeURIComponent(path)}` : ""),
    ),

  /** The branch, and whether anything is uncommitted. */
  gitStatus: () => request<GitStatus>("GET", "/api/git/status"),

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

  agentTurns: (docId: string) =>
    request<AgentTurnsResponse>("GET", `/api/docs/${docId}/agent/turns`),

  /** Every terminal session, and the one attention queue across them. The
   * order of `attention` is the server's — see hick_term::attention — so the
   * queue, the session tree, and ⌘J cannot disagree about what is next. */
  terminals: () => request<TerminalsResponse>("GET", "/api/terminals"),
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
