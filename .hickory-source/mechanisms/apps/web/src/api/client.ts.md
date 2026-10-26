<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/client.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1e909407d4cd"></a>
## `apps/web/src/api/client.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A running agent can be stopped](../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md), [A New Project Is A Recipe Commit, Made As One Act](../../../../../guarantees/authoring/a-new-project-is-a-recipe-commit/contract.md), [The Editor Does Not Wait For Workspace Preparation](../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md), [Typed Text Is Prose, Not A Transcript](../../../../../guarantees/authoring/typed-text-is-prose-not-a-transcript/contract.md), [A Missing Merge Driver Is A Button, Not A Command To Go And Type](../../../../../guarantees/collaboration/a-missing-merge-driver-is-a-button/contract.md), [The Git Pane Does The Daily Loop](../../../../../guarantees/collaboration/the-git-pane-does-the-daily-loop/contract.md), [A Command The App Runs On Your Behalf Is Watched In A Terminal](../../../../../guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal/contract.md), [Stepping A Table Replays The Order The Host Chose](../../../../../guarantees/execution/stepping-a-table-replays-the-order-the-host-chose/contract.md), [An Element Is Drawn By Its View](../../../../../guarantees/language/an-element-is-drawn-by-its-view/contract.md), [Find And Replace Is Exhaustive, And Refuses What It Would Undo](../../../../../guarantees/search/find-and-replace-is-exhaustive/contract.md).

<a id="f-1e909407d4cd-MOCK"></a>
### `MOCK`

<storycode:copy id="f-1e909407d4cd-MOCK">
import type { AgentEditorContext } from "./agentTypes";
import type { FleetPairingJob } from "./fleetPairings";
import { generatedApi } from "./generatedApi";
import type { CommitReadingData } from "./git";
import type {
  AdoptResponse,
  BlockActionOutcome,
  FindOptions,
  FindResponse,
  FleetMachine,
  GitCommitResult,
  GitDiff,
  GitLog,
  GitSaid,
  MergedViewResponse,
  OpenTerminal,
  PlainFileSaved,
  ProjectSuggestion,
  RenderResponse,
  ReplaceResponse,
  ReplayResponse,
  SampleCreated,
  SavedAsset,
  ScaffoldCatalog,
  ScaffoldPreview,
  ScaffoldSpec,
  ScaffoldStarted,
  ScaffoldTemplateDetail,
  ScratchpadSaved,
  SearchResponse,
  TerminalSession,
  WorkspaceUiState
} from "./types";
import { waitForWorkspace } from "./workspaceStartup";

export const MOCK = import.meta.env.VITE_MOCK === "1";

</storycode:copy>
<a id="f-1e909407d4cd-ApiError"></a>
### `ApiError`

<storycode:copy id="f-1e909407d4cd-ApiError">
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

</storycode:copy>
<a id="f-1e909407d4cd-ERROR_TEXT_LIMIT"></a>
### `ERROR_TEXT_LIMIT`

The longest error body worth putting in front of a person as a sentence.
Past this it is a page, not a message, and the console has the whole of it.

<storycode:copy id="f-1e909407d4cd-ERROR_TEXT_LIMIT">
/** The longest error body worth putting in front of a person as a sentence.
 * Past this it is a page, not a message, and the console has the whole of it. */
const ERROR_TEXT_LIMIT = 600;

</storycode:copy>
<a id="f-1e909407d4cd-apiError"></a>
### `apiError`

A failed response, read for everything it actually says.
The server's own refusals are `{"error": "…"}` and were always read. What
was not, and what cost a real debugging session, is everything that answers
a request **before** a handler runs: axum's own extractor rejections are
`text/plain`, so a 422 saying `missing field \`image\`` — the whole
diagnosis, sitting right there in the body — was thrown away and shown as
the HTTP status text, "Unprocessable Entity". A message that names no
field, no value and no route is not an error message.
So: the JSON shape first, then the body's own text, and only then the
status line — which is the honest answer for a body that is genuinely
empty. `user-facing-errors`: an error says which check failed.

<storycode:copy id="f-1e909407d4cd-apiError">
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

</storycode:copy>
<a id="f-1e909407d4cd-MockHandler"></a>
### `MockHandler`

<storycode:copy id="f-1e909407d4cd-MockHandler">
type MockHandler = (
  method: string,
  path: string,
  body: unknown,
) => Promise<unknown>;

</storycode:copy>
<a id="f-1e909407d4cd-mockHandler"></a>
### `mockHandler`

<storycode:copy id="f-1e909407d4cd-mockHandler">
let mockHandler: MockHandler | null = null;

</storycode:copy>
<a id="f-1e909407d4cd-installMockHandler"></a>
### `installMockHandler`

Installed by src/mock/mockApi.ts when VITE_MOCK=1.

<storycode:copy id="f-1e909407d4cd-installMockHandler">
/** Installed by src/mock/mockApi.ts when VITE_MOCK=1. */
export function installMockHandler(h: MockHandler) {
  mockHandler = h;
}

</storycode:copy>
<a id="f-1e909407d4cd-request"></a>
### `request`

<storycode:copy id="f-1e909407d4cd-request">
export async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  keepalive = false,
): Promise<T> {
  if (mockHandler) {
    return (await mockHandler(method, path, body)) as T;
  }
  const headers: Record<string, string> = {};
  if (body !== undefined) headers["Content-Type"] = "application/json";
  // Preserve the exact requested bytes if preparation postpones dispatch;
  // editing a form while waiting must not change an already submitted act.
  const encoded = body === undefined ? undefined : JSON.stringify(body);
  for (;;) {
    const res = await fetch(path, {
      ...(keepalive ? { keepalive: true } : {}),
      method,
      headers,
      body: encoded,
    });
    if (!res.ok) {
      if (res.status === 503 && res.headers.get("x-hickory-startup") === "pending") {
        await res.body?.cancel();
        await waitForWorkspace();
        continue;
      }
      throw await apiError(res);
    }
    if (res.status === 204) return undefined as T;
    return (await res.json()) as T;
  }
}

</storycode:copy>
<a id="f-1e909407d4cd-inFlightRenders"></a>
### `inFlightRenders`

Rendering a document is the most expensive thing the API does, and the app
asks for it from several independent places (initial load, save, run
events, lineage edits). Concurrent requests for the SAME document are
collapsed onto one in-flight promise: a render started 10 ms ago cannot
have a different answer from one started now, and the callers all want the
same value. The entry is cleared as soon as it settles, so the next call
always fetches fresh state.

<storycode:copy id="f-1e909407d4cd-inFlightRenders">
// Rendering a document is the most expensive thing the API does, and the app
// asks for it from several independent places (initial load, save, run
// events, lineage edits). Concurrent requests for the SAME document are
// collapsed onto one in-flight promise: a render started 10 ms ago cannot
// have a different answer from one started now, and the callers all want the
// same value. The entry is cleared as soon as it settles, so the next call
// always fetches fresh state.
const inFlightRenders = new Map<string, Promise<RenderResponse>>();

</storycode:copy>
<a id="f-1e909407d4cd-dedupedRender"></a>
### `dedupedRender`

<storycode:copy id="f-1e909407d4cd-dedupedRender">
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

</storycode:copy>
<a id="f-1e909407d4cd-api"></a>
### `api`

The compact [route declarations](routes.ts.md) own the 99 mechanical wrappers.
The [code-model generator](../../../../scripts/generate-api.py.md) emits their
parameter forwarding; this object retains the adapters with defaults, conditional
payloads, render deduplication, and special transport behavior. `request` still
owns startup gating and errors for both kinds of adapter.

<storycode:copy id="f-1e909407d4cd-api">
export const api = {
  ...generatedApi(request),
  render: (id: string) => dedupedRender(id),
  /** Adopt a plain file into a literate document, byte-exactly: the server
   * verifies the new document weaves the file's exact bytes before writing
   * anything. `into` appends to an existing document instead of creating
   * `<stem>.md` beside the file. */
  adopt: (path: string, into?: string) =>
    request<AdoptResponse>("POST", "/api/adopt", {
      path,
      ...(into !== undefined ? { into } : {}),
    }),

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

  /** Put a `<hick:sample>` under the cell that generated this file: a window
   * onto a few of its lines, shown in the weave and never stored in the
   * document. `path` is root-relative; the server finds the owning cell.
   * See crates/hickory-cli/src/serve/sample.rs. */
  createSample: (path: string, from: number, to: number, caption?: string, expectedOutputHash?: string) =>
    request<SampleCreated>("POST", "/api/samples", {
      path,
      from,
      to,
      ...(caption ? { caption } : {}),
      ...(expectedOutputHash ? { expected_output_hash: expectedOutputHash } : {}),
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
  saveScratchpad: (text: string, agentAssisted = false) =>
    request<ScratchpadSaved>("POST", agentAssisted ? "/api/v1/scratchpad" : "/api/scratchpad", { text, ...(agentAssisted ? { agent_assisted: true } : {}) }),

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
  /** One action on the element whose tag starts at byte `at`. The element
   * says what it wants and the server carries it out; a run comes back as
   * a run id on the same channel `run` uses. */
  blockAction: (docId: string, at: number, action: string, body?: unknown) =>
    request<BlockActionOutcome>(
      "POST",
      `/api/docs/${docId}/blocks/${at}/${encodeURIComponent(action)}`,
      body ?? {},
    ),
  run: (docId: string, cells?: string[], expectedSourceHash?: string) =>
    request<{ run_id: string }>(
      "POST",
      `/api/docs/${docId}/run`,
      { ...(cells ? { cells } : {}), ...(expectedSourceHash ? { expected_source_hash: expectedSourceHash } : {}) },
    ),
  check: (docId: string, expectedSourceHash?: string) =>
    request<{ run_id: string }>("POST", `/api/docs/${docId}/check`, expectedSourceHash ? { expected_source_hash: expectedSourceHash } : undefined),
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

  /** The commit graph, with every commit's files and line counts — one git
   * invocation, so expanding a row costs nothing. The verbs that change the
   * repository are below, each one git command run as itself. */
  gitLog: (limit = 120, path?: string) =>
    request<GitLog>(
      "GET",
      `/api/git/log?limit=${limit}` +
        (path ? `&path=${encodeURIComponent(path)}` : ""),
    ),
  // The history lens's verbs (lenses.md steps 4–6). Each is one git
  // operation run as itself on the server; a refusal comes back in git's
  // words, and a join that stops on a conflict comes back as 409.

  // The git pane's verbs — each one git command, run as itself, with git's
  // own words when it refuses. See crates/hickory-cli/src/serve/git_ops.rs.
  gitReading: (sha: string) => request<CommitReadingData>("GET", `/api/git/reading?${new URLSearchParams({ sha })}`),
  gitDiff: (path: string, staged = false) =>
    request<GitDiff>(
      "GET",
      `/api/git/diff?path=${encodeURIComponent(path)}&staged=${staged ? "true" : "false"}`,
    ),
  gitCommit: (message: string, amend = false) =>
    request<GitCommitResult>("POST", "/api/git/commit", { message, amend }),
  gitCheckout: (branch: string, create = false) =>
    request<GitSaid>("POST", "/api/git/checkout", { branch, create }),

  /** Captured worktree lens; exact selection is independent of write authority. */
  merged: (path: string, targets?: string[]) =>
    request<MergedViewResponse>(
      "GET",
      `/api/v1/merged?path=${encodeURIComponent(path)}` +
        (targets !== undefined ? `&targets=${encodeURIComponent(JSON.stringify(targets))}` : ""),
    ).catch(error => {
      if (error instanceof ApiError && error.status === 404) throw new ApiError(404, "Worktree selection protocol v1 is unavailable. Update the app and engine together; no legacy selection was requested.", error.body);
      throw error;
    }),
  cancelFleetPairing: (window: string, id: string) => request<{ job: FleetPairingJob }>("POST", `/api/v1/fleet/pairings/${encodeURIComponent(id)}/cancel?window=${encodeURIComponent(window)}`, {}, true),

  /** The line another machine accepts to enrol this one. */
  fleetInvite: (phone = false) =>
    request<{ invitation: string; fingerprint: string; note: string }>(
      "GET",
      `/api/fleet/invite${phone ? "?phone=true" : ""}`,
    ),

  /** Give or take one verb for one machine. */
  fleetGrant: (machine: string, grant: string, on: boolean, expectedKey?: string) =>
    request<{ machine: FleetMachine }>("PUT", expectedKey !== undefined ? "/api/v1/fleet/grant" : "/api/fleet/grant", {
      machine, grant, on, ...(expectedKey !== undefined ? { expected_key: expectedKey } : {}),
    }),

  /** Revocation, which is deleting a key. */
  fleetRemove: (machine: string, expectedKey?: string) =>
    request<{ removed: boolean; note: string }>("POST", expectedKey !== undefined ? "/api/v1/fleet/remove" : "/api/fleet/remove", {
      machine, ...(expectedKey !== undefined ? { expected_key: expectedKey } : {}),
    }),

  /** The lineage this document had AT a commit — recomputed by weaving that
   * commit's document, never executing anything. */
  docReplay: (docId: string, commit: string, path?: string) =>
    request<ReplayResponse>(
      "GET",
      `/api/docs/${docId}/replay?commit=${encodeURIComponent(commit)}` +
        (path ? `&path=${encodeURIComponent(path)}` : ""),
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
    expectedSources?: Record<string, string>,
  ) =>
    request<ReplaceResponse>("POST", "/api/find/replace", {
      q,
      replacement,
      ...options,
      ...(paths ? { paths } : {}),
      ...(expectedSources ? { expected_sources: expectedSources } : {}),
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
    backend?: string,
    context?: AgentEditorContext,
  ) =>
    request<{ session_id: string }>("POST", `/api/docs/${docId}/agent`, {
      backend,
      context,
      prompt,
      parent_id: parentId ?? null,
      ...(provider !== undefined ? { provider } : {}),
      ...(model !== undefined ? { model } : {}),
    }),
  openTerminal: (spec: OpenTerminal = {}) =>
    request<TerminalSession>("POST", "/api/terminals", spec),
  /** Type into a session. The pane's own keystrokes go over the socket
   * instead; this is for everything else (menus, the card's input line). */
  terminalInput: (id: string, data: string, agent = false) =>
    request<void>("POST", `/api/terminals/${id}/input`, { data, ...(agent ? { agent: true } : {}) }),

  // --- What the window remembers between runs -----------------------------
  //
  // Stored under the user's own data directory, never in the project: git
  // cannot reach it by construction. See api.md, "Workspace state and
  // drafts".
  saveResponseDraft: (doc: string, scope: string, state: unknown, keepalive = false) =>
    request<{ ok: true }>("PUT", `/api/workspace/responses?doc=${encodeURIComponent(doc)}`, { scope, state }, keepalive),
  saveWorkspaceUi: (state: WorkspaceUiState, keepalive = false) =>
    request<{ ok: true }>("PUT", "/api/workspace/ui", { state }, keepalive)
};
</storycode:copy>

### Assemble `apps/web/src/api/client.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/client.ts"><storycode:paste select="#f-1e909407d4cd-MOCK"/><storycode:paste select="#f-1e909407d4cd-ApiError"/><storycode:paste select="#f-1e909407d4cd-ERROR_TEXT_LIMIT"/><storycode:paste select="#f-1e909407d4cd-apiError"/><storycode:paste select="#f-1e909407d4cd-MockHandler"/><storycode:paste select="#f-1e909407d4cd-mockHandler"/><storycode:paste select="#f-1e909407d4cd-installMockHandler"/><storycode:paste select="#f-1e909407d4cd-request"/><storycode:paste select="#f-1e909407d4cd-inFlightRenders"/><storycode:paste select="#f-1e909407d4cd-dedupedRender"/><storycode:paste select="#f-1e909407d4cd-api"/></storycode:file>

## Forwarding witness

A real client call must preserve URL encoding, literal interpolation and the
wire distinction between no body, an empty object and the caller’s draft object.
This focused assertion exercises the generated adapters through the same mocked
transport used by the existing client tests.

<storycode:copy id="api-codegen-forwarding-witness">
describe("generated API forwarding", () => {
  afterEach(() => installMockHandler(null as never));
  it("preserves encoded paths, literal identifiers and body distinctions", async () => {
    const handler = vi.fn(async (_method: string, _path: string, _body?: unknown) => ({}));
    installMockHandler(handler);
    await api.file("a b/é.md");
    await api.doc("a/b");
    await api.saveDoc("doc", "text");
    await api.gitPush();
    await api.closeWindow();
    const draft = { path: "doc.md", contents: "text", base: "", saved_at: 0 };
    await api.saveDraft(draft);
    expect(handler.mock.calls).toEqual([
      ["GET", "/api/file?path=a%20b%2F%C3%A9.md", undefined],
      ["GET", "/api/docs/a/b", undefined],
      ["PUT", "/api/docs/doc", { source: "text" }],
      ["POST", "/api/git/push", {}],
      ["POST", "/api/window/close", undefined],
      ["PUT", "/api/workspace/drafts", draft],
    ]);
    expect(handler.mock.calls[5][2]).toBe(draft);
  });
});
</storycode:copy>
</storycode:doc>
