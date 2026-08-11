# Environment configuration

One canonical variable name set for every environment — dev, staging, and
production differ only in *values* (no `STAGING_*` twins). The server reads
these at boot (`apps/server/src/config.rs`) and validates them:

- **Dev** (`APP_ENV` unset or `dev`): graceful degradation. Missing optional
  integrations (Stripe, PostHog, canopy) become feature no-ops with a boot
  log line; missing core vars fall back to documented dev defaults, loudly.
- **Strict** (`APP_ENV=staging|production`): invalid or unsafe configuration
  fails the boot. Optional integrations may still be absent (their endpoints
  answer 503 / no-op), but *misconfigured* values never boot.

## Core

| Variable | Required | Default (dev) | Notes |
|---|---|---|---|
| `APP_ENV` | no | `dev` | `dev` \| `staging` \| `production` |
| `PORT` | no | `8080` | HTTP listen port. `just dev` sets `0` (kernel picks; Port Zero publishes it by name) |
| `DATABASE_URL` | strict: yes | `postgres://hickory:hickory@localhost:5433/hickory` | Postgres; sqlx migrations run at boot |
| `JWT_SECRET` | strict: yes (≥32 bytes) | insecure dev constant | HS256 signing secret |
| `GIT_DATA_DIR` | no | `./data/git` | One plain git repo per project lives here; mount a persistent volume in deploys |
| `HICKORY_EXECUTOR` | no | `local` | `local` \| `canopy`. With `canopy`, the canopy env below is validated at boot: strict mode fails fast on invalid values; dev falls back to local with a warning |
| `APP_BASE_URL` | no | `http://localhost:<PORT>` | Public base URL used for Stripe redirect URLs. Dev: `http://hickory.portzero.local` |
| `WEB_DIST_DIR` | no | `./apps/web/dist` if present | Static web app served with SPA fallback |

## Billing (optional — absent ⇒ billing endpoints answer 503 "billing not configured")

| Variable | Notes |
|---|---|
| `STRIPE_SECRET_KEY` | Must be `sk_live_…` in production and a test/sandbox key in staging (validated at boot) |
| `STRIPE_WEBHOOK_SECRET` | Required in strict mode whenever Stripe is configured |
| `PLAN_SET` | Explicit plan-set override; otherwise the PostHog flag `hickory-plan-set`, else `default` |

## Bring your own key (optional — absent ⇒ accounts cannot store their own keys)

| Variable | Required | Notes |
|---|---|---|
| `KEY_ENCRYPTION_KEY` | no, but see below | Base64 of 32 random bytes; encrypts every account's stored provider API key (`user_llm_keys`). Mint one with `just gen-key`, or `openssl rand -base64 32` when scripting — the value ends in `=` padding, so `cut -d= -f2` on the `KEY=value` line truncates it. **One per environment, and keep it**: replacing it makes every stored key unreadable and each account has to paste its key again. A malformed value fails the boot in every environment rather than silently disabling the feature |

Absent, `/api/me/llm-keys` answers 503 and accounts on a `byo_key` plan (Open,
Pro in `plans.json`) have **no agent at all** — the entitlement is "the account
brings a key", and there is nowhere to put one. On a deployment that sells
those plans this variable is effectively required; it is not marked strict only
because an existing deployment must not fail to boot the moment this feature
ships.

## Agent (optional — absent ⇒ accounts on a `metered_allowance` plan answer 503)

The deployment's own key, spent by plans whose entitlement includes an agent
allowance (Team, Business). Accounts on `byo_key` plans never reach it — see
`docs/guarantees/agent/byok-plan-boundary.md`.

| Variable | Notes |
|---|---|
| `HICKORY_LLM_PROVIDER` | `anthropic` (default) \| `openai` \| `deepseek` \| `grok`. Selects which key variable below is read |
| `HICKORY_LLM_MODEL` | Model override; default is the provider's own |
| `OPENAI_API_KEY` / `DEEPSEEK_API_KEY` / `XAI_API_KEY` | The key for the selected non-Anthropic provider |
| `ANTHROPIC_API_KEY` | Enables the server-side agent (hickory-agent ReAct loop; sessions stream on the WS run channel and persist as `hick:session` docs in the project git repo) |
| `ANTHROPIC_BASE_URL` | Messages endpoint for the agent. Defaults to `https://api.anthropic.com/v1/messages`; `count_tokens` follows the same host. Set it for an enterprise gateway or proxy — or for a local endpoint that records requests, which is how the conversation tests run without a real key |

## The relay (`hickory-relay-production` — a separate Fly app)

Set on the relay app, not the workspace app. See
`docs/operators/deploy-relay.md`.

| Variable | Required | Notes |
|---|---|---|
| `RELAY_APEX` | yes | The domain sessions hang off (`relay.hickorydocs.com`). Must match the wildcard DNS record and certificate; the relay cannot guess its own public name |
| `RELAY_TOKEN_SECRET` | yes (≥32 bytes) | Signs every token the relay issues. **Stable across restarts** — a new value signs everyone out. `openssl rand -base64 48` |
| `RELAY_DATABASE_URL` | yes in practice | SQLite for the accounts table, e.g. `sqlite:///data/relay.db?mode=rwc` on the mounted volume. Absent, existing tokens still work but nobody new can sign in |
| `RELAY_SCHEME` | no (`https`) | `http` only for a relay running locally without TLS |
| `GH_OAUTH_CLIENT_ID` | no | Enables GitHub sign-in. Absent, the relay offers email/password only and the CLI hides the GitHub option entirely. Device flow needs no client secret |
| `GITHUB_API_BASE` | no | Override for testing against a stand-in |

## Analytics (optional — absent ⇒ capture is a no-op)

| Variable | Notes |
|---|---|
| `POSTHOG_API_KEY` | Project **write** key (`phc_…`); enables server-side capture (`signup`, `doc_run`, `doc_check`, billing events), plan-set flag lookup, **and** the landing page's events, which the browser posts to `POST /api/analytics/capture` for the server to forward. The project is Terraform-owned (`terraform/posthog`); get the key onto Fly with `just posthog-sync-key`, never by copying it out of the dashboard. Leave it unset in local dev — dev traffic in the production project cannot be separated out afterwards. |
| `POSTHOG_HOST` | Default `https://us.i.posthog.com` |

Do not confuse `POSTHOG_API_KEY` with `POSTHOG_PERSONAL_API_KEY`. The first is
a project write key that can only send events; the second is a personal key
that can create and destroy projects, lives only in GitHub Environments, and
is never given to the running server. See `docs/operators/analytics.md`.

There is deliberately **no `VITE_POSTHOG_KEY`**. `import.meta.env.VITE_*` is
inlined when the web bundle is built — the `web` stage of `Dockerfile` — so a
build-time key would bake one environment's project id into the very image the
promote path then ships to the other environment. Routing browser events
through the server keeps one canonical variable name, one promotable image,
and no analytics credential in the bundle. See
`docs/specs/freeform/landing-discovery.md`.

## Cloud Canopy (read when `HICKORY_EXECUTOR=canopy`)

| Variable | Notes |
|---|---|
| `CANOPY_URL` | Control-plane GraphQL endpoint (portzero tunnel or canopy nginx/ACME domain) |
| `CANOPY_TOKEN` | Base64 capability token |
| `CANOPY_NODE` | Node name (e.g. `colo-1`) |
| `CANOPY_IMAGE_MAP` | JSON: OCI image reference → Nix store sandbox image path |

See `docs/specs/freeform/canopy-integration.md` for the full canopy contract.

## Dev loop

`just dev` copies `.env.example` to `.env` when missing, boots Postgres via
docker compose, then the server and the Vite dev server (proxying `/api`,
including WS, to the server). `just dev-stop` shuts it down.

Dev does not pick port numbers. The server binds port `0` and the kernel hands
out a free one; Vite has no port-0 support, so it falls back to its own
increment-until-free default. Either way nothing has to agree on a number: the
[Port Zero](https://portzero.net) daemon discovers each process by its
`PZ_TUNNEL` environment variable and publishes whatever port it landed on at a
stable name:

| URL | Service |
|---|---|
| `http://hickory.portzero.local` | Vite dev server (the one you open) |
| `http://api.hickory.portzero.local` | `hickory-server` |
| `db.hickory.portzero.local:5432` | Postgres |

`PZ_NAMESPACE` in `.env` renames all three at once, so a second checkout or a
git worktree can run a complete parallel stack with no port coordination —
set it and point `DATABASE_URL` at the matching `db.<namespace>` host.
`just dev` requires the `portzero` daemon and starts it if it is not running.

Postgres is the deliberate exception: it still publishes a fixed host port
(`POSTGRES_PORT`, default `5433`) *in addition to* its tunnel name. CI has no
Postgres service, and `apps/server/tests/integration.rs` bootstraps one with
`docker compose up -d --wait db`, reaching it at the hardcoded
`localhost:5433` fallback. Dev uses the name; tests and CI use the port, and
neither needs the daemon.
