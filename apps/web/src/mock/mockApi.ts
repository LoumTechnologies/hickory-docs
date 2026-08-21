import { ApiError, installMockHandler } from "../api/client";
import { LocalRealtime } from "../api/realtime";
import type {
  AgentTurn,
  Block,
  Doc,
  ExecBlock,
  FileNode,
  Run,
  TranscriptEvent,
} from "../api/types";
import { MOCK_BLOCKS, MOCK_DOCS, MOCK_PROJECTS, PAPER_CHART_SVG } from "./mockData";
import {
  SyntheticRangeViolation,
  applySourceEdits,
  mapEditsToSource,
  weaveOutputs,
} from "../lib/weave";
import type { OutputEdit } from "../api/types";

// In-browser mock API (VITE_MOCK=1): implements the api.md contract, including
// fake streaming runs over the LocalRealtime "socket", so `npm run dev:mock`
// demos the whole UI with no backend.

export const mockRealtime = new LocalRealtime();

const state = {
  projects: [...MOCK_PROJECTS],
  docs: MOCK_DOCS.map((d) => ({ ...d })),
  blocks: Object.fromEntries(
    Object.entries(MOCK_BLOCKS).map(([id, blocks]) => [id, blocks.map((b) => ({ ...b }))]),
  ) as Record<string, Block[]>,
  runs: new Map<string, Run>(),
  agentTurns: [] as (AgentTurn & { doc_id: string })[],
  uiSettings: { window_title: null as string | null },
  // The window layout, in memory. A landing-page visitor must not have one
  // written anywhere on their machine.
  workspaceUi: null as unknown,
  // Past the seeded ids (d1…, p1…): starting at 1 minted a created document
  // as "d1", colliding with the seeded quickstart — the new tab silently
  // became a second window onto an existing document.
  nextId: 100,
};

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

/** Pinned refactor baselines, doc id → output path → content — the mock's
 * half of serve/refactor.rs, over the mock's own weave. */
const refactorBaselines = new Map<string, Map<string, string>>();
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
    { t: 400, kind: "out", data: "hick 0.4.2\n" },
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
    // The same block model the real server records: the run's own blocks,
    // which is what the document view reads its transcripts from.
    blocks: cells.map((c) => ({ ...c, status: c.status ?? "never-run", transcript: [] })),
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
      const rb = run.blocks.find((b) => b.kind === "exec" && b.id === cell.id);
      if (rb && rb.kind === "exec") {
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
    "$ hick run demo/hello.hick\nconverged: 3 nodes, 0 stale\n",
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

/** Group flat paths into the /api/files tree: dirs first, each level
 * alphabetical, `doc_id` only on documents — matching the server contract. */
function toFileNodes(entries: ReadonlyMap<string, string | undefined>, prefix = ""): FileNode[] {
  const dirs = new Map<string, Map<string, string | undefined>>();
  const files: FileNode[] = [];
  for (const [path, docId] of entries) {
    const slash = path.indexOf("/");
    if (slash === -1) {
      files.push({ name: path, path: `${prefix}${path}`, dir: false, ...(docId ? { doc_id: docId } : {}) });
    } else {
      const dir = path.slice(0, slash);
      if (!dirs.has(dir)) dirs.set(dir, new Map());
      dirs.get(dir)!.set(path.slice(slash + 1), docId);
    }
  }
  const byName = (a: { name: string }, b: { name: string }) => a.name.localeCompare(b.name);
  const dirNodes: FileNode[] = [...dirs]
    .map(([name, children]) => ({
      name,
      path: `${prefix}${name}`,
      dir: true,
      children: toFileNodes(children, `${prefix}${name}/`),
    }))
    .sort(byName);
  return [...dirNodes, ...files.sort(byName)];
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

    // What the window remembers. The mock keeps it in memory: the demo has
    // no data directory, and a landing-page visitor must not have a window
    // layout written anywhere on their machine.
    if (route === "GET /api/workspace/ui") return { state: state.workspaceUi ?? null };
    if (route === "PUT /api/workspace/ui") {
      state.workspaceUi = b.state;
      return { ok: true };
    }
    // No drafts, ever: nothing in the demo has a disk to be unsaved from.
    if (route === "GET /api/workspace/drafts") return { drafts: [] };
    if (path.startsWith("/api/workspace/drafts")) return { ok: true };

    // Exhaustive find over the mock's woven files. Replace is refused: the
    // demo has no disk to write to, and saying so is more honest than a
    // success that changed nothing.
    if (path.startsWith("/api/find?")) {
      const q = new URLSearchParams(path.slice(path.indexOf("?"))).get("q") ?? "";
      if (!q) return { files: [], truncated: false };
      const files: unknown[] = [];
      for (const doc of state.docs) {
        for (const file of weaveOutputs(doc.source, doc.path)) {
          const matches = file.content
            .split("\n")
            .map((text, i) => ({ text, line: i + 1 }))
            .filter((row) => row.text.toLowerCase().includes(q.toLowerCase()))
            .map((row) => ({
              line: row.line,
              text: row.text,
              at: [{ column: row.text.toLowerCase().indexOf(q.toLowerCase()), length: q.length }],
            }));
          if (matches.length) files.push({ path: file.path, matches, generated_by: doc.id });
        }
      }
      return { files, truncated: false };
    }
    if (route === "POST /api/find/replace") {
      return { changed: [], skipped: [], replacements: 0 };
    }

    if (route === "GET /api/files") {
      // The open folder: every document, plus everything they weave.
      const entries = new Map<string, string | undefined>();
      for (const doc of state.docs) entries.set(doc.path, doc.id);
      for (const doc of state.docs) {
        for (const file of weaveOutputs(doc.source, doc.path)) {
          if (!entries.has(file.path)) entries.set(file.path, undefined);
        }
      }
      return { root: "/home/mock/notebook", tree: toFileNodes(entries) };
    }

    // Plain files. The mock folder holds nothing but documents and what
    // they weave, so a plain-file read serves the woven copy; a save is
    // refused honestly — the demo has no disk to land it on.
    if ((m = path.match(/^\/api\/file\?path=(.+)$/)) && method === "GET") {
      const filePath = decodeURIComponent(m[1]);
      for (const doc of state.docs) {
        const file = weaveOutputs(doc.source, doc.path).find((f) => f.path === filePath);
        if (file) {
          const { path: p, language, content } = file;
          return { path: p, language, content, hash: `mock-${content.length}` };
        }
      }
      notFound(path);
    }
    if (path === "/api/file" && method === "PUT") {
      throw Object.assign(
        new Error("This demo runs without a disk, so plain-file saves stay in the buffer."),
        { status: 422 },
      );
    }
    if (path === "/api/adopt" && method === "POST") {
      throw Object.assign(
        new Error(
          "This demo runs without a disk, so there is nothing to adopt — in the app, this wraps the file in a document, byte-exactly.",
        ),
        { status: 422 },
      );
    }

    // The refactor baseline: real behavior over the mock's own weave, so
    // the demo badge tells the same truth the app's does.
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/refactor\/(begin|status|end)$/))) {
      const doc = state.docs.find((d) => d.id === m![1]) ?? notFound(path);
      const verb = m[2];
      if (verb === "begin") {
        const outputs = new Map(
          weaveOutputs(doc.source, doc.path).map((f) => [f.path, f.content]),
        );
        refactorBaselines.set(doc.id, outputs);
        return { active: true, started_at: new Date().toISOString(), clean: true, diffs: [] };
      }
      if (verb === "end") {
        refactorBaselines.delete(doc.id);
        return { active: false };
      }
      const baseline = refactorBaselines.get(doc.id);
      if (!baseline) return { active: false };
      const current = new Map(weaveOutputs(doc.source, doc.path).map((f) => [f.path, f.content]));
      const diffs: { path: string; kind: string }[] = [];
      for (const [p, was] of baseline) {
        if (!current.has(p)) diffs.push({ path: p, kind: "removed" });
        else if (current.get(p) !== was) diffs.push({ path: p, kind: "changed" });
      }
      for (const p of current.keys()) {
        if (!baseline.has(p)) diffs.push({ path: p, kind: "added" });
      }
      return { active: true, started_at: new Date().toISOString(), clean: diffs.length === 0, diffs };
    }

    if (path === "/api/settings/ui") {
      // The UI settings the real server persists as ui.json. In-memory here:
      // the mock has no disk, and the page only needs the round trip.
      if (method === "PUT") state.uiSettings = { window_title: (b.window_title as string | null) ?? null };
      return state.uiSettings;
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
    if ((m = path.match(/^\/api\/docs\/([^/]+)\/agent\/turns$/))) {
      const turns = state.agentTurns.filter((t) => t.doc_id === m![1]).map(({ doc_id: _d, ...t }) => t);
      const totals = { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 };
      for (const t of turns) {
        if (!t.usage) continue;
        totals.input += t.usage.input_tokens;
        totals.output += t.usage.output_tokens;
        totals.cache_read += t.usage.cache_read_input_tokens;
        totals.cache_write += t.usage.cache_creation_input_tokens;
        // Mock pricing: claude-sonnet-5 list rates, matching usage.rs.
        totals.usd +=
          (t.usage.input_tokens * 3 +
            t.usage.cache_creation_input_tokens * 1.25 * 3 +
            t.usage.cache_read_input_tokens * 0.1 * 3 +
            t.usage.output_tokens * 15) /
          1e6;
      }
      const last = turns[turns.length - 1];
      return {
        turns,
        provider: last?.provider ?? "anthropic",
        model: last?.model ?? "claude-sonnet-5",
        totals,
      };
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
        provider: typeof b.provider === "string" && b.provider !== "" ? b.provider : "anthropic",
        model: typeof b.model === "string" && b.model !== "" ? b.model : "claude-sonnet-5",
        usage: {
          input_tokens: 1200,
          cache_creation_input_tokens: 300,
          cache_read_input_tokens: 4800,
          output_tokens: 450,
        },
      });
      return { session_id: id };
    }
    if (route === "GET /api/health") return { ok: true, executor: "local", db: true };
    // Environment cards: mock runs everything on the "local" executor.
    if (route === "GET /api/executor") return { kind: "local", images: null };
    notFound(path);
  });
}
