# Deploying to Railway

The repo ships a root `Dockerfile` (multi-stage: web dist + release server
binary, slim Debian runtime) and `railway.json` (Dockerfile builder, health
check on `/api/health`). One Railway service runs the server, which also
serves the built web app; Postgres is a Railway plugin.

## First-time setup

1. `railway login` and `railway init` (or link the GitHub repo in the
   dashboard; branch: `master` — production is a gated promote per the
   portfolio convention).
2. Add a **Postgres** database to the project. Railway injects
   `DATABASE_URL` into the service via a reference variable:
   `DATABASE_URL=${{Postgres.DATABASE_URL}}`.
3. Add a **volume** mounted at `/data` — `GIT_DATA_DIR=/data/git` (the
   default in the image) is where per-project git repos live. Without a
   volume, doc history dies on every deploy.
4. Set the service variables (canonical names in
   `docs/operators/ENVIRONMENTS.md`):

   ```
   APP_ENV=staging                # or production, once promoted
   JWT_SECRET=<openssl rand -hex 32>
   APP_BASE_URL=https://<your-domain>
   HICKORY_EXECUTOR=local         # or canopy (set the CANOPY_* vars below)
   # optional:
   STRIPE_SECRET_KEY=sk_test_...  # sandbox until the readiness gate in
   STRIPE_WEBHOOK_SECRET=whsec_...#   pricing-strategy.md is met
   POSTHOG_API_KEY=phc_...
   POSTHOG_HOST=https://us.i.posthog.com
   ANTHROPIC_API_KEY=sk-ant-...   # enables the agent endpoint
   ```

   Boot is strict in staging/production: a missing `JWT_SECRET` or
   `DATABASE_URL`, a live Stripe key in staging (or a test key in
   production), or `HICKORY_EXECUTOR=canopy` without the canopy crate all
   fail fast with a clear error.
5. `railway up` (or push to `master` with the repo linked). Migrations run
   automatically at boot. Verify `https://<domain>/api/health` returns
   `{"ok":true,"executor":"local","db":true}`.
6. Stripe webhook: add an endpoint in the Stripe dashboard pointing at
   `https://<domain>/api/billing/webhook` with events
   `checkout.session.completed`, `customer.subscription.updated`,
   `customer.subscription.deleted`, `invoice.payment_failed`; copy its
   signing secret into `STRIPE_WEBHOOK_SECRET`.

## Cloud Canopy note

The execution node's control plane binds `127.0.0.1:8088` on Nate's
hardware. The Railway app reaches it through a **portzero tunnel** (or,
later, canopy's nginx/ACME `domain` option): expose the control plane via
the tunnel, then set `CANOPY_URL` to the tunnel URL plus `CANOPY_TOKEN`,
`CANOPY_NODE`, and `CANOPY_IMAGE_MAP`. These variables are ignored while
`HICKORY_EXECUTOR=local`; with `HICKORY_EXECUTOR=canopy` they are validated
at boot (strict mode fails fast on invalid values). The canopy adapter
boundary is `crates/hickory-executor-canopy`, selected only in
`apps/server/src/executor.rs` (see `docs/specs/freeform/canopy-integration.md`).

## Image notes

- Plain multi-stage build (no cargo-chef): the runtime layer is
  `debian:bookworm-slim` + `git` + CA certs plus the single release binary
  and the web dist.
- `git` must stay in the runtime image (per-project repos) and the local
  executor runs doc commands with the host shell — treat the container as
  the execution sandbox boundary until canopy is wired.
