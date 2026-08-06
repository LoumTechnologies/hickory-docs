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
