import { ApiError, installMockHandler, setToken } from "../api/client";
import { LocalRealtime } from "../api/realtime";
import type {
  AgentTurn,
  Block,
  Doc,
  ExecBlock,
  Run,
  TranscriptEvent,
  User,
} from "../api/types";
import {
  MOCK_BLOCKS,
  MOCK_DOCS,
  MOCK_ENTERPRISE,
  MOCK_PLANS,
  MOCK_PROJECTS,
  PAPER_CHART_SVG,
} from "./mockData";
import {
  SyntheticRangeViolation,
  applySourceEdits,
  mapEditsToSource,
  weaveOutputs,
} from "../lib/weave";
import type { LlmKey, OutputEdit } from "../api/types";

// In-browser mock API (VITE_MOCK=1): implements the api.md contract, including
// fake streaming runs over the LocalRealtime "socket", so `npm run dev:mock`
// demos the whole UI with no backend.

export const mockRealtime = new LocalRealtime();

const state = {
  user: null as User | null,
  projects: [...MOCK_PROJECTS],
  docs: MOCK_DOCS.map((d) => ({ ...d })),
  blocks: Object.fromEntries(
    Object.entries(MOCK_BLOCKS).map(([id, blocks]) => [id, blocks.map((b) => ({ ...b }))]),
  ) as Record<string, Block[]>,
  llmKeys: [] as LlmKey[],
  runs: new Map<string, Run>(),
  agentTurns: [] as (AgentTurn & { doc_id: string })[],
  nextId: 1,
};

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));
const id = (prefix: string) => `${prefix}${state.nextId++}`;

function execBlocks(docId: string): ExecBlock[] {
  return (state.blocks[docId] ?? []).filter((b): b is ExecBlock => b.kind === "exec");
}

/** Fake transcripts a cell produces when (re-)run in mock mode. */
function fakeTranscript(cell: ExecBlock): TranscriptEvent[] {
  if (cell.id === "paper-figure") {
    return [
      { t: 0, kind: "cmd", data: cell.command },
      { t: 250, kind: "out", data: "loading data/runs.json (300 samples)\n" },
      { t: 900, kind: "out", data: PAPER_CHART_SVG },
      { t: 950, kind: "exit", code: 0 },
    ];
  }
  if (cell.id === "cli-run") {
    return [
      { t: 0, kind: "cmd", data: cell.command },
      { t: 300, kind: "out", data: "parsing demo/hello.hick\n" },
      { t: 800, kind: "out", data: "exec shell: echo hello\n" },
      { t: 1300, kind: "out", data: "converged: 3 nodes, 0 stale\n" },
      { t: 1360, kind: "exit", code: 0 },
    ];
  }
  if (cell.id === "cli-init") {
    return [
      { t: 0, kind: "cmd", data: cell.command },
      { t: 350, kind: "out", data: "initialised demo/\n" },
      { t: 700, kind: "out", data: "installed pre-commit hook (sentinel-delimited)\n" },
      { t: 1050, kind: "out", data: "cache  git  hooks\n" },
      { t: 1100, kind: "exit", code: 0 },
    ];
  }
  if (cell.id === "weave-run") {
    return [
      { t: 0, kind: "cmd", data: cell.command },
      { t: 400, kind: "out", data: "{'1': 4.1, '4': 4.9, '16': 7.2, '64': 15.8, '256': 47.0}\n" },
      { t: 460, kind: "exit", code: 0 },
    ];
  }
  return [
    { t: 0, kind: "cmd", data: cell.command },
    { t: 400, kind: "out", data: "hickory 0.4.2\n" },
    { t: 450, kind: "exit", code: 0 },
  ];
}

/**
 * Streams a fake run: emits WS run events in real time, mutates the stored
 * blocks so a later /render reflects the run, and completes the run record.
 * `verify=true` makes the check on doc d2 fail (output drift) so both banner
 * states are demoable.
 */
function startRun(docId: string, cellIds: string[] | undefined, verify: boolean): string {
  const runId = id(verify ? "check-" : "run-");
  const cells = execBlocks(docId).filter((c) => !cellIds || cellIds.includes(c.id));
  const run: Run = {
    id: runId,
    status: "running",
    started_at: new Date().toISOString(),
    blocks: cells.map((c) => ({ exec_id: c.id, status: c.status ?? "never-run", transcript: [] })),
  };
  state.runs.set(runId, run);

  const drift = verify && docId === "d2";

  void (async () => {
    await delay(200);
    let anyFailed = false;
    for (const cell of cells) {
      const events = fakeTranscript(cell);
      const collected: TranscriptEvent[] = [];
      let prev = 0;
      for (const event of events) {
        await delay(event.t - prev);
        prev = event.t;
        const emitted: TranscriptEvent =
          drift && cell.id === "paper-figure" && event.kind === "exit"
            ? { t: event.t, kind: "exit", code: 1 }
            : event;
        collected.push(emitted);
        mockRealtime.emit({ run_id: runId, exec_id: cell.id, event: emitted });
        if (drift && cell.id === "paper-figure" && event.kind === "out" && event.data.startsWith("<svg")) {
          const err: TranscriptEvent = {
            t: event.t + 20,
            kind: "err",
            data: "check: figure drift vs committed woven output (data/runs.json changed)\n",
          };
          collected.push(err);
          mockRealtime.emit({ run_id: runId, exec_id: cell.id, event: err });
        }
      }
      const failed = collected.some((e) => e.kind === "exit" && e.code !== 0);
      anyFailed ||= failed;
      const status = failed ? "failed" : "ok";
      const rb = run.blocks.find((b) => b.exec_id === cell.id);
      if (rb) {
        rb.status = status;
        rb.transcript = collected;
      }
      // Persist into the doc's render model.
      const stored = execBlocks(docId).find((c) => c.id === cell.id);
      if (stored) {
        stored.transcript = collected;
        stored.status = status;
      }
    }
    run.status = anyFailed ? "failed" : "ok";
    mockRealtime.emit({ run_id: runId, status: run.status });
  })();

  return runId;
}

function streamAgentSession(docId: string, prompt: string): string {
  const sessionId = id("session-");
  const chunks = [
    `session opened for ${state.docs.find((d) => d.id === docId)?.path ?? docId}\n`,
    `> ${prompt}\n\n`,
    "Reading document and pipeline state…\n",
    "Plan: add an <hick:expect> block to the unverified cell, then re-run it.\n",
    "Editing source (span 612..796)…\n",
    "Running cell to capture a fresh transcript…\n",
    "$ hickory run demo/hello.hick\nconverged: 3 nodes, 0 stale\n",
    "Verification passes. Session committed as sessions/2026-08-05-a.hick\n",
  ];
  void (async () => {
    await delay(300);
    let t = 0;
    for (const data of chunks) {
      t += 350 + Math.floor(Math.random() * 500);
      await delay(t > 0 ? 450 : 0);
      mockRealtime.emit({
        run_id: sessionId,
        exec_id: "agent",
        event: { t, kind: "out", data },
      });
    }
    mockRealtime.emit({ run_id: sessionId, status: "ok" });
  })();
  return sessionId;
}

function notFound(path: string): never {
  throw Object.assign(new Error(`mock: not found ${path}`), { status: 404 });
}

export function installMockApi() {
  installMockHandler(async (method, path, body) => {
    await delay(120); // simulated network latency
    const b = body as Record<string, unknown>;
    const route = `${method} ${path}`;
    let m: RegExpMatchArray | null;

    if (route === "POST /api/auth/signup" || route === "POST /api/auth/login") {
      const email = String(b.email ?? "dev@example.com");
      state.user = { id: "u1", email, plan: "pro" };
      setToken("mock-token");
      return { token: "mock-token", user: state.user };
    }
    if (route === "GET /api/me") {
      return state.user ?? { id: "u1", email: "dev@example.com", plan: "pro" };
    }
    // BYOK, in the demo: keys behave as they do for real (write-only, one
    // active) so the mock never teaches a UI habit the server would refuse.
    if (path === "/api/me/llm-keys" || path.startsWith("/api/me/llm-keys/")) {
      const listing = () => ({
        keys: state.llmKeys,
        storage_available: true,
        plan_agent: "byo_key",
      });
      if (method === "GET") return listing();
      if ((m = path.match(/^\/api\/me\/llm-keys\/([^/]+)$/))) {
        const provider = m[1] === "grok" ? "xai" : m[1];
        if (method === "DELETE") {
          state.llmKeys = state.llmKeys.filter((k) => k.provider !== provider);
          if (state.llmKeys.length === 1) state.llmKeys[0].active = true;
          return listing();
        }
        const key = String(b.api_key ?? "");
        const stored = {
          provider,
          last4: key.slice(-4),
          model: (b.model as string) ?? null,
          active: state.llmKeys.length === 0,
          created_at: new Date().toISOString(),
          last_used_at: null,
        };
        state.llmKeys = [
          ...state.llmKeys.filter((k) => k.provider !== provider),
          stored,
        ];
        return stored;
      }
      // PUT /api/me/llm-keys — choose the active key.
      const chosen = String(b.provider) === "grok" ? "xai" : String(b.provider);
      state.llmKeys = state.llmKeys.map((k) => ({
        ...k,
        active: k.provider === chosen,
      }));
      return listing();
    }
    if (route === "GET /api/projects") return state.projects;
    if (route === "POST /api/projects") {
      const project = {
        id: id("p"),
        name: String(b.name),
        visibility: (b.visibility as "public" | "private") ?? "private",
        created_at: new Date().toISOString(),
      };
      state.projects.push(project);
      return project;
    }
    if ((m = path.match(/^\/api\/projects\/([^/]+)\/docs$/))) {
      const projectId = m[1];
      if (method === "GET") {
        return state.docs
          .filter((d) => d.project_id === projectId)
          .map(({ id, path, updated_at }) => ({ id, path, updated_at }));
      }
      const doc: Doc & { project_id: string } = {
        id: id("d"),
        project_id: projectId,
        path: String(b.path),
        source: String(b.source ?? ""),
        updated_at: new Date().toISOString(),
      };
      state.docs.push(doc);
      state.blocks[doc.id] = [];
      return doc;
    }
    if ((m = path.match(/^\/api\/docs\/([^/]+)$/))) {
      const doc = state.docs.find((d) => d.id === m![1]) ?? notFound(path);
      if (method === "PUT") {
        doc.source = String(b.source);
        doc.updated_at = new Date().toISOString();
        // Source changed: previously-ok cells become stale.
        for (const cell of execBlocks(doc.id)) {
          if (cell.status === "ok") cell.status = "stale";
        }
      }
      const { project_id: _p, ...rest } = doc;
      return rest;
    }
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/render$/))) {
      state.docs.find((d) => d.id === m![1]) ?? notFound(path);
      return { blocks: state.blocks[m![1]] ?? [] };
    }
    // Generated outputs & lineage (v0.2). Outputs are woven fresh from the
    // doc's current source, so provenance ranges are always real.
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/outputs$/))) {
      const doc = state.docs.find((d) => d.id === m![1]) ?? notFound(path);
      const files = weaveOutputs(doc.source, doc.path);
      return { files: files.map(({ path, language }) => ({ path, language })) };
    }
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/outputs\/file\?path=(.+)$/))) {
      const doc = state.docs.find((d) => d.id === m![1]) ?? notFound(path);
      const filePath = decodeURIComponent(m![2]);
      const file = weaveOutputs(doc.source, doc.path).find((f) => f.path === filePath);
      return file ?? notFound(path);
    }
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/outputs\/edit$/))) {
      const doc = state.docs.find((d) => d.id === m![1]) ?? notFound(path);
      const filePath = String(b.path);
      const edits = (b.edits ?? []) as OutputEdit[];
      const file = weaveOutputs(doc.source, doc.path).find((f) => f.path === filePath);
      if (!file) notFound(path);
      try {
        const sourceEdits = mapEditsToSource(file, edits);
        doc.source = applySourceEdits(doc.source, sourceEdits);
        doc.updated_at = new Date().toISOString();
        // Source changed: previously-ok cells go stale, as with PUT /docs/:id.
        for (const cell of execBlocks(doc.id)) {
          if (cell.status === "ok") cell.status = "stale";
        }
        return { source_edits: sourceEdits, applied: true };
      } catch (e) {
        if (e instanceof SyntheticRangeViolation) {
          throw new ApiError(422, e.message, { error: e.message, range: e.range });
        }
        throw e;
      }
    }
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/run$/))) {
      return { run_id: startRun(m![1], b.cells as string[] | undefined, false) };
    }
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/check$/))) {
      return { run_id: startRun(m![1], undefined, true) };
    }
    if ((m = path.match(/^\/api\/runs\/([^/]+)$/))) {
      return state.runs.get(m![1]) ?? notFound(path);
    }
    if (route === "GET /api/billing/plans")
      return { plans: MOCK_PLANS, enterprise: MOCK_ENTERPRISE };
    if (route === "POST /api/billing/checkout") {
      return { checkout_url: `https://checkout.stripe.com/mock/${String(b.price_key)}` };
    }
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/agent\/turns$/))) {
      return { turns: state.agentTurns.filter((t) => t.doc_id === m![1]).map(({ doc_id: _d, ...t }) => t) };
    }
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/agent$/))) {
      const docId = m![1];
      const id = streamAgentSession(docId, String(b.prompt));
      state.agentTurns.push({
        doc_id: docId,
        id,
        parent_id: (b.parent_id as string | null) ?? null,
        prompt: String(b.prompt),
        answer: "Mock agent: nothing was actually executed.",
        status: "ok",
        error: null,
        created_at: new Date().toISOString(),
      });
      return { session_id: id };
    }
    if (route === "GET /api/health") return { ok: true, executor: "local", db: true };
    // Environment cards: mock runs everything on the "local" executor.
    if (route === "GET /api/executor") return { kind: "local", images: null };
    notFound(path);
  });
}
