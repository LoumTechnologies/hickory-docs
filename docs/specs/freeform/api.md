# Server API contract (v0)

Base: `/api`. Auth: `Authorization: Bearer <JWT>` (argon2 password auth).
All bodies JSON. This contract is the coupling point between `apps/server`
and `apps/web` — change it here first.

## Auth & account
- `POST /api/auth/signup` `{email, password}` → `{token, user}`
- `POST /api/auth/login` `{email, password}` → `{token, user}`
- `GET  /api/me` → `{id, email, plan}`

## Projects & documents
- `GET  /api/projects` → `[{id, name, visibility, created_at}]`
- `POST /api/projects` `{name, visibility: "public"|"private"}` → project
- `GET  /api/projects/:id/docs` → `[{id, path, updated_at}]`
- `POST /api/projects/:id/docs` `{path, source}` → doc
- `GET  /api/docs/:id` → `{id, path, source, updated_at}`
- `PUT  /api/docs/:id` `{source}` → doc  (server persists to the project git repo)

## Rendering (block model)
- `GET /api/docs/:id/render` → `{blocks: Block[]}` where

```ts
type Block =
  | { kind: "prose";  html: string; span: [number, number] }
  | { kind: "exec";   id: string; container: string; image?: string;
      command: string; span: [number, number];
      transcript?: TranscriptEvent[]; expect?: { match: "exact"|"regex-lines"; body: string };
      status?: "ok"|"failed"|"stale"|"never-run" }
  | { kind: "file";   path: string; language: string; body: string; span: [number, number] }
  | { kind: "session-user" | "session-assistant" | "session-observation";
      body: string; span: [number, number] };

type TranscriptEvent =
  | { t: number; kind: "cmd";  data: string }
  | { t: number; kind: "out" | "err"; data: string }
  | { t: number; kind: "exit"; code: number };
```

`span` is byte offsets into the doc source (provenance — clicking a block
selects its source).

## Execution
- `POST /api/docs/:id/run` `{cells?: string[]}` → `{run_id}` (202)
- `GET  /api/runs/:id` → `{id, status: "queued"|"running"|"ok"|"failed", started_at,
   blocks: {exec_id, status, transcript}[]}`
- `POST /api/docs/:id/check` → `{run_id}` — verification run (expect blocks + drift)

## Realtime — `WS /api/ws?doc=doc:<id>&token=<JWT>`
Connection params: `doc` (the Yjs doc name, `doc:<id>`) and `token` (JWT —
browsers cannot set WS headers). One socket, message-framed by a 1-byte
channel prefix:
- `0x00` + Yjs sync/awareness bytes (y-websocket protocol) — collaborative
  editing state shared by web/iOS/Android.
- `0x01` + JSON run event: `{run_id, exec_id, event: TranscriptEvent}` and
  `{run_id, status}` terminal messages.

`TranscriptEvent.t` is **milliseconds since run start**.

Agent sessions stream on the run channel with `run_id === session_id` and
`exec_id: "agent"`.

## Billing
- `GET  /api/billing/plans` → the active plan set from `plans.json` (respects
  PostHog flag for plan-set selection). Shape (pinned; the web client's
  `src/api/types.ts` mirrors it):
  `{plans: [{key, name, description, trial_days?, highlight?, prices: [{key,
  interval, amount_cents, currency, per_seat?}], features: string[]}],
  enterprise?}`
- `POST /api/billing/checkout` `{price_key}` → `{checkout_url}` (Stripe)
- `POST /api/billing/webhook` — Stripe webhooks (signature-verified, idempotent)

## Agent
- `POST /api/docs/:id/agent` `{prompt}` → `{session_id}` — starts an agent
  session; session events stream on the WS run channel; the session itself is
  persisted as a `hick:session` document in the project repo.

## Ops
- `GET /api/health` → `{ok: true, executor: "local"|"canopy", db: bool}`
- `GET /api/executor` → `{kind: "local"|"canopy", images: {<image ref>: <store path>}|null}`
  — where cells run, for the Document view's environment cards. `images` is
  the configured image-ref → Nix-store-path map on canopy (empty object when
  none configured); `null` on local, where the `image` attribute is recorded
  provenance, not an enforced sandbox.

## Generated outputs & lineage (v0.2)

The other half of the editor: a doc's generated output files (e.g. one code
file woven from many `hick:copy`/`hick:paste` slots) are viewable AND
editable, with every character's lineage traced back to its source span.

- `GET /api/docs/:id/outputs` → `{files: [{path, language}]}` — output files
  the doc produced on its last successful run.
- `GET /api/docs/:id/outputs/file?path=<rel>` →
  `{path, language, content, provenance: Provenance[]}` where

```ts
type Provenance = {
  start: number; end: number;            // byte range in `content`
  origin:
    | { kind: "literal" | "paste" | "exec" | "variable" | "substitution";
        doc_path: string; span: [number, number] }   // byte span in source doc
    | { kind: "synthetic" };             // separators etc. — not editable
};
```

- `POST /api/docs/:id/outputs/edit` `{path, edits: [{start, end, text}]}` →
  `{source_edits: [{doc_path, span: [number, number], text}], applied: true}`.
  The server maps output-range edits through provenance to source-document
  edits, applies them (git commit per edit batch), and the next run
  reproduces the edited output. Edits overlapping `synthetic` ranges → 422
  with body `{error: string, range: {start: number, end: number}}` (the
  offending output byte range). All ranges in this section are UTF-8 byte
  offsets.

## Editor model (v0.2)

The web UI has TWO views (replacing Notebook/Source):
1. **Document** — one Typora-style WYSIWYG editor over the raw `.hick`
   source: syntax stays visible (tags, markdown marks) while styled like the
   rendered result; exec cells render run buttons/status/transcripts inline
   as widgets. Collab (Yjs) runs on the raw source exactly as before.
2. **Output** — the generated files, syntax-aware, with lineage: selecting
   output text highlights its origin; edits POST to /outputs/edit.

## Editor intelligence — LSP bridge (v0.3)

The server runs `hick-lsp` (the meta-LSP: virtual files per `hick:file`,
child language servers, positions mapped to `.hick` coordinates) against the
project checkout, bridged to the browser on the existing doc WebSocket:

- Channel byte `0x02` + JSON-RPC 2.0 payload (UTF-8): a plain LSP stream,
  one message per frame, no Content-Length headers. The server owns one
  hick-lsp session per (project, connection); `initialize` is handled
  server-side — the client starts at `didOpen` using URI
  `hick:///<doc-path>` and LSP positions computed over the SAME source text
  the editor holds.
- Supported requests v1: `textDocument/hover`, `textDocument/definition`,
  `textDocument/references`, `textDocument/completion`,
  `textDocument/publishDiagnostics` (server→client). Child-language-server
  availability is best-effort: a missing rust-analyzer/pyright degrades to
  hick-structural answers, never errors the channel.
- Definition/reference results whose target lies inside a GENERATED output
  file are translated through run provenance back to source-document
  coordinates when possible; untranslatable targets are returned with URI
  `hick-output:///<output-path>` so the client can open the Output view at
  that range.
- The Output view navigates too: `POST /api/docs/:id/outputs/nav`
  `{path, offset, kind: "definition"|"references"}` →
  `{targets: [{uri: "hick:///…"|"hick-output:///…", range: {start, end}} …]}`
  (byte offsets; server maps output positions into the virtual-file space,
  asks the child LSP, and maps results back through provenance).
