import type {
  AgentTurn,
  AuthResponse,
  Doc,
  DocSummary,
  ExecutorInfo,
  LlmKey,
  LlmKeysResponse,
  OutputEdit,
  OutputEditResponse,
  OutputFile,
  OutputsResponse,
  PlansResponse,
  Project,
  RenderResponse,
  Run,
  User,
  StructureResponse,
} from "./types";

export const MOCK = import.meta.env.VITE_MOCK === "1";

const TOKEN_KEY = "hickory.token";

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string | null) {
  if (token === null) localStorage.removeItem(TOKEN_KEY);
  else localStorage.setItem(TOKEN_KEY, token);
}

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
  const token = getToken();
  if (token) headers.Authorization = `Bearer ${token}`;
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
  signup: (email: string, password: string) =>
    request<AuthResponse>("POST", "/api/auth/signup", { email, password }),
  login: (email: string, password: string) =>
    request<AuthResponse>("POST", "/api/auth/login", { email, password }),
  me: () => request<User>("GET", "/api/me"),

  sendVerification: () =>
    request<{ status: string }>("POST", "/api/auth/verify/send", {}),
  confirmVerification: (token: string) =>
    request<{ status: string }>("POST", "/api/auth/verify/confirm", { token }),
  requestReset: (email: string) =>
    request<{ status: string }>("POST", "/api/auth/reset/request", { email }),
  confirmReset: (token: string, password: string) =>
    request<{ status: string }>("POST", "/api/auth/reset/confirm", {
      token,
      password,
    }),

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

  plans: () => request<PlansResponse>("GET", "/api/billing/plans"),
  checkout: (priceKey: string) =>
    request<{ checkout_url: string }>("POST", "/api/billing/checkout", {
      price_key: priceKey,
    }),

  executor: () => request<ExecutorInfo>("GET", "/api/executor"),

  /** Definitions and references across this session's generated files. */
  structure: () => request<StructureResponse>("GET", "/api/structure"),

  /** Start a turn. `parentId` continues from that turn — naming an older one
   * forks a branch (rewind) rather than overwriting what followed it. */
  agent: (docId: string, prompt: string, parentId?: string | null) =>
    request<{ session_id: string }>("POST", `/api/docs/${docId}/agent`, {
      prompt,
      parent_id: parentId ?? null,
    }),

  agentTurns: (docId: string) =>
    request<{ turns: AgentTurn[] }>("GET", `/api/docs/${docId}/agent/turns`),

  llmKeys: () => request<LlmKeysResponse>("GET", "/api/me/llm-keys"),
  /** Store or replace this account's key for one provider. The server
   * validates it against the vendor before saving, so a rejection here is a
   * bad key and not a deferred surprise mid-run. */
  saveLlmKey: (
    provider: string,
    apiKey: string,
    opts?: { model?: string; preferred?: boolean },
  ) =>
    request<LlmKey>("PUT", `/api/me/llm-keys/${provider}`, {
      api_key: apiKey,
      model: opts?.model || null,
      preferred: opts?.preferred ?? false,
    }),
  deleteLlmKey: (provider: string) =>
    request<LlmKeysResponse>("DELETE", `/api/me/llm-keys/${provider}`),
  selectLlmKey: (provider: string) =>
    request<LlmKeysResponse>("PUT", "/api/me/llm-keys", { provider }),
};
