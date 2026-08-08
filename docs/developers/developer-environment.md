# Developer environment

Short and procedural. See `docs/specs/freeform/architecture.md` for the
design; this page only gets you to a running stack.

## Machine dependencies

- Docker + docker compose
- Rust (pinned by `rust-toolchain.toml` — `rustup` picks it up automatically)
- Node.js 22+
- [`just`](https://github.com/casey/just)
- [PortZero](https://portzero.net) daemon (`portzero`) — optional, but
  `just dev` uses it to publish stable `*.portzero.local` names instead of
  hand-picked ports

## From clone to running app

1. `just dev` — idempotent. First run copies `.env.example` to `.env`,
   generates an untracked `.dev-env.local` (per-worktree compose isolation),
   installs the pre-commit hook, brings up Postgres, and starts the server +
   web dev server. Safe to Ctrl-C and re-run at any point; leaves no
   processes running when it exits.
   ```
   Web:  http://hickory.portzero.local
   API:  http://api.hickory.portzero.local/api/health
   DB:   db.hickory.portzero.local:5432
   ```
2. `just dev-seed` — creates seeded accounts through the real signup
   endpoint (never a DB insert). Safe to run more than once.
   | email | password |
   |---|---|
   | `dev@hickory.local` | `dev-password-123` |
   | `owner@hickory.local` | `owner-password-123` |
3. `just dev-stop` — stop without deleting data.
4. `just dev-clean` — stop, then delete **this worktree's** containers and
   volumes. Never touches another worktree's stack, and never deletes a
   pulled base image (`postgres:16-alpine` stays cached).

## API client codegen

`apps/server`'s OpenAPI spec is generated purely from `#[utoipa::path]` /
`ToSchema` macros on the route handlers — it needs no database, no `.env`,
and no network. `apps/web`'s typed client types are generated from that spec
with `openapi-typescript`.

- `just codegen` — regenerate `apps/server/openapi.json` and
  `apps/web/src/api/generated/schema.d.ts`.
- `just check-codegen` — `codegen` + fail if the committed output is stale.
  This is what CI's `check-codegen` job and the pre-commit hook both run —
  there is exactly one implementation of "is the client stale."

**Never hand-edit anything under `apps/server/openapi.json` or
`apps/web/src/api/generated/`.** If a merge shows conflict markers in either:

1. Don't read them.
2. `git checkout --ours -- apps/server/openapi.json apps/web/src/api/generated` (either side — the content is about to be regenerated).
3. `just codegen`
4. `git add apps/server/openapi.json apps/web/src/api/generated`
5. Continue the merge/rebase.

**Known temporary exception**: `apps/web/src/api/client.ts` and `types.ts`
are still hand-written, not generated. Most `apps/server` handlers return
`Json<serde_json::Value>` built from ad-hoc `json!()` rather than typed
response structs, so the generated schema's response types are currently
generic JSON objects — switching the app over today would strip real types
(`User`, `Project`, ...) for no gain. Typing responses route-by-route and
migrating callers onto the generated client is tracked as separate follow-up
work, not silently deferred forever.

## Third-party integrations

Every integration degrades gracefully when its credential is unset — no
combination of present/absent accounts blocks the dev environment from
coming up. See `apps/server/src/config.rs:190-264` and `src/mail.rs`.

| Integration | Unset | Set |
|---|---|---|
| SendGrid (`SENDGRID_API_KEY`) | Email verification + password reset disabled; mail is a logged no-op | `MAIL_FROM` also required, or startup fails |
| Stripe (`STRIPE_SECRET_KEY`) | Billing endpoints answer 503 | Staging must use a `sk_test_`/sandbox key, production a live key; `STRIPE_WEBHOOK_SECRET` required once Stripe is configured in staging/production |
| PostHog (`POSTHOG_API_KEY`) | Analytics capture is a no-op | — |
| Agent LLM (`HICKORY_LLM_PROVIDER` + provider key) | Agent endpoint answers 503, naming the exact missing env var | One of `ANTHROPIC_API_KEY`/`OPENAI_API_KEY`/`DEEPSEEK_API_KEY`/`XAI_API_KEY` |
| Canopy executor (`CANOPY_URL`/`CANOPY_TOKEN`) | Only relevant when `HICKORY_EXECUTOR=canopy` | — |

No internet connection: every integration above is already opt-in via env
var, so a fully offline machine behaves the same as "everything unset."

## Pre-commit hook and selective CI

`just dev` installs `.githooks/pre-commit` (`git config core.hooksPath
.githooks`). It runs `scripts/affected-checks.sh` against the merge-base
with `master` and executes only what that returns — the exact same script
`.github/workflows/ci.yml`'s `changes` job calls, so a hook pass can never be
followed by a CI failure the hook should have caught.

Mapping (`scripts/affected-checks.sh`):

| Changed paths | Checks run |
|---|---|
| `apps/server/**`, `crates/**`, `Cargo.toml`, `Cargo.lock` | `rust`, `check-codegen` |
| `apps/web/**` | `web`, `check-codegen` |
| `docs/**`, `examples/**` | `rust` — this repo's `rust` job also verifies docs/examples drift (`hickory-cli check docs/`, `check examples/`), so a docs-only change genuinely needs it |
| `docker-compose.yml`, `scripts/dev*.sh`, `justfile` | `rust`, `web` — deliberately the full suites; there is no separate fast dev-environment smoke test yet (follow-up work) |
| anything else (e.g. `README.md`, `LICENSE`) | nothing |

A push to `master` always runs every CI job unconditionally — the mapping
only scopes pull request checks. If something fails on `master` that a PR's
selective checks missed, that means a row above is wrong (too narrow, or
missing a check), not something to shrug at.

## Known limitations

- **Postgres keeps a fixed host port (`POSTGRES_PORT`, default `5433`)**,
  deliberately — CI's integration tests bootstrap their own Postgres via
  `docker compose` and fall back to `localhost:5433` when `DATABASE_URL` is
  unset (`apps/server/tests/integration.rs`). Two worktrees running `just
  dev` at the same time will still contend for that one port even though
  their containers/networks/volumes are otherwise isolated
  (`COMPOSE_PROJECT_NAME` from `.dev-env.local`). `just dev-stop` or
  `just dev-clean` in one worktree frees it for the other.
