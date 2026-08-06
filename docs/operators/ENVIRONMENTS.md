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
| `PORT` | no | `8080` | HTTP listen port |
| `DATABASE_URL` | strict: yes | `postgres://hickory:hickory@localhost:5433/hickory` | Postgres; sqlx migrations run at boot |
| `JWT_SECRET` | strict: yes (≥32 bytes) | insecure dev constant | HS256 signing secret |
| `GIT_DATA_DIR` | no | `./data/git` | One plain git repo per project lives here; mount a persistent volume in deploys |
| `HICKORY_EXECUTOR` | no | `local` | `local` \| `canopy`. `canopy` refuses to boot in strict mode until `hickory-executor-canopy` lands (dev falls back to local with a warning) |
| `APP_BASE_URL` | no | `http://localhost:<PORT>` | Public base URL used for Stripe redirect URLs |
| `WEB_DIST_DIR` | no | `./apps/web/dist` if present | Static web app served with SPA fallback |

## Billing (optional — absent ⇒ billing endpoints answer 503 "billing not configured")

| Variable | Notes |
|---|---|
| `STRIPE_SECRET_KEY` | Must be `sk_live_…` in production and a test/sandbox key in staging (validated at boot) |
| `STRIPE_WEBHOOK_SECRET` | Required in strict mode whenever Stripe is configured |
| `PLAN_SET` | Explicit plan-set override; otherwise the PostHog flag `hickory-plan-set`, else `default` |

## Analytics (optional — absent ⇒ capture is a no-op)

| Variable | Notes |
|---|---|
| `POSTHOG_API_KEY` | Project API key; enables server-side capture (`signup`, `doc_run`, `doc_check`, billing events) and plan-set flag lookup |
| `POSTHOG_HOST` | Default `https://us.i.posthog.com` |

## Cloud Canopy (optional; only read once `hickory-executor-canopy` lands)

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
including WS, to the server). Ports come from `.env` (`PORT`, `WEB_PORT`,
`POSTGRES_PORT`) — nothing is hardcoded. `just dev-stop` shuts it down.
