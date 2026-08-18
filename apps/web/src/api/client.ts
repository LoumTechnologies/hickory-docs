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
  Run,
  SearchResponse,
  SettingsKeysPatch,
  SettingsKeysResponse,
  StructureResponse,
  OpenTerminal,
  TerminalSession,
  TerminalsResponse,
  UiSettings,
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

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
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
  const p = request<RenderResponse>("GET", `/api/docs/${id}/render`).finally(() => {
    if (inFlightRenders.get(id) === p) inFlightRenders.delete(id);
  });
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
    request<{ run_id: string }>("POST", `/api/docs/${docId}/run`, cells ? { cells } : {}),
  runStatus: (runId: string) => request<Run>("GET", `/api/runs/${runId}`),
  check: (docId: string) =>
    request<{ run_id: string }>("POST", `/api/docs/${docId}/check`),

  executor: () => request<ExecutorInfo>("GET", "/api/executor"),

  /** The agent's LLM API keys: configured-or-not plus a masked hint. The
   * full key never travels back — see SettingsKeysResponse. */
  settingsKeys: () => request<SettingsKeysResponse>("GET", "/api/settings/keys"),
  /** Set/clear only the named providers (string sets, null clears). */
  saveSettingsKeys: (patch: SettingsKeysPatch) =>
    request<SettingsKeysResponse>("PUT", "/api/settings/keys", patch),

  /** UI settings the server persists (ui.json): the custom window title. */
  settingsUi: () => request<UiSettings>("GET", "/api/settings/ui"),
  saveSettingsUi: (settings: UiSettings) =>
    request<UiSettings>("PUT", "/api/settings/ui", settings),

  /** Definitions and references across this session's generated files. */
  structure: () => request<StructureResponse>("GET", "/api/structure"),

  /** Ranked project-wide search over documents and generated files. */
  search: (q: string, k = 20) =>
    request<SearchResponse>("GET", `/api/search?q=${encodeURIComponent(q)}&k=${k}`),

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
  closeTerminal: (id: string) => request<void>("DELETE", `/api/terminals/${id}`),
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
};
