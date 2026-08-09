# Deploying to Fly.io

The server, the web app, and the LSP bridge are one image. `fly.toml` and
`Dockerfile` in the repository root are the whole deployment.

## You do not deploy by hand

**Pushing to `master` deploys to production.** CI runs first; if it passes,
**Deploy Production** (`.github/workflows/deploy-production.yml`) deploys that
exact commit to `hickory-docs-production` and then health-checks
<https://hickorydocs.com/api/health>. A red CI run deploys nothing.

There is no staging environment and no promote gate. That is deliberate and
temporary — the reasoning and the conditions that should end it are in
`.instructions/continuous-delivery-shared.md`.

The commands below are for the **first** deploy, for setting secrets, and for
the times automation is not available. A manual `fly deploy` from a laptop
deploys your working tree, not a commit anyone can identify later — reach for
it only when you mean it.

### Rolling back

Revert the commit and push. The revert goes through CI and deploys like any
other change.

**A revert does not undo a database migration.** Migrations run at boot and are
forward-only, so a deploy that added a destructive migration needs a *forward*
fix — a new migration restoring what the last one removed. Rolling the app
image back under a migrated database gives you an old binary against a new
schema, which is usually worse than the bug you were fixing.

To get back on your feet faster than a revert allows:

```sh
fly releases --app hickory-docs-production          # find the last good version
fly deploy --image <image-ref-from-that-release> --app hickory-docs-production
```

Then still push the revert, or the next green build redeploys the bad code.

### When the health check fails

The deploy already happened — a failed health check does not roll anything
back. Start here:

```sh
fly logs --app hickory-docs-production
fly status --app hickory-docs-production
```

The most common cause is not a bad build: it is a missing or invalid secret,
which in production is a deliberate refusal to start (see
`docs/operators/ENVIRONMENTS.md`) rather than a silent degrade.

### Deploy credentials

The workflow authenticates with `FLY_API_TOKEN`, a deploy-scoped token held in
the **`production-deploy`** GitHub Environment alongside `APP_BASE_URL`. It
expires one year from issue. To replace it:

```sh
fly tokens create deploy --app hickory-docs-production --name github-actions-deploy --expiry 8760h \
  | gh secret set FLY_API_TOKEN --env production-deploy
```

`production-deploy` is ungated; the separate `production` environment keeps a
required reviewer and is used only by Terraform applies. Putting the deploy
token in `production` would make every app deploy wait for a click; removing
that reviewer instead would ungate DNS and analytics changes. Hence two
environments. See `docs/operators/analytics.md` for the full table.

## What it costs when nobody is using it

`auto_stop_machines = "stop"` with `min_machines_running = 0` means the machine
stops when idle and wakes on the next request, so **compute is $0 at idle**.
Two things still cost money and it is worth being precise:

- **The volume.** `/data` holds the per-project git repositories. Fly bills
  volumes whether or not the machine is running — a 1 GB volume is a small
  monthly charge, not zero.
- **Postgres.** Fly's managed Postgres is not free. Point `DATABASE_URL` at a
  provider whose free tier scales to zero (Neon and Supabase both do) and this
  stays at zero too.

Waking from stopped adds a cold start to the first request. A live editing
session holds a WebSocket, which keeps the machine awake for as long as someone
is actually working.

## First deploy

```sh
fly launch --no-deploy          # claims the app name; keep the committed fly.toml
fly volumes create hickory_data --size 1 --region iad

fly secrets set \
  DATABASE_URL='postgres://…' \
  JWT_SECRET="$(openssl rand -base64 48)" \
  ANTHROPIC_API_KEY='sk-ant-…'

fly deploy
```

`JWT_SECRET` must be at least 32 bytes and `DATABASE_URL` must be set — in
production the server refuses to start without either, rather than coming up in
a degraded state nobody notices.

Optional: `STRIPE_SECRET_KEY` / `STRIPE_WEBHOOK_SECRET` (billing endpoints
answer 503 when unset), `POSTHOG_API_KEY`, and `HICKORY_LLM_PROVIDER` +
that provider's key.

## Email and who may sign up

```sh
fly secrets set \
  SENDGRID_API_KEY='SG.…' \
  MAIL_FROM='noreply@yourdomain.com' \
  MAIL_FROM_NAME='Hickory Docs' \
  SIGNUP_ALLOWLIST='you@yourdomain.com,@yourcompany.com'
```

`MAIL_FROM` must be an address SendGrid has authenticated as a sender identity
for your account, or every send is rejected. Setting the key without it is a
startup error rather than a silent failure to deliver.

With email configured, an account must confirm its address before it can run
documents or start an agent — reading and writing stay open. Without it, that
gate is a no-op, because a deployment cannot require proof it has no way to
request.

`SIGNUP_ALLOWLIST` accepts exact addresses and whole domains (`@example.com`).
Unset means anyone may sign up. **On a public deployment, set one.** Until the
web app has a verification screen, the allowlist is the practical gate: the URL
is public, and an account that gets past signup can execute code.

## Execution: read this before you rely on it

`fly.toml` sets `HICKORY_EXECUTOR=docker`, which is the executor that makes a
document's `image=` real — `python:3.12` means that image, not whatever the
host happens to have. The runtime image ships the Docker **client**.

It does **not** ship a Docker daemon, and a Fly Machine does not provide one.
Nested containers need privileges Fly Machines are not guaranteed to have, and
a runtime that silently degrades to the host toolchain would quietly break the
reproducibility claim this product is built on. So the daemon has to come from
somewhere you choose:

- **`DOCKER_HOST=tcp://…`** pointing at a Docker host you run. Works today.
  It is an always-on machine, so it is not $0 at idle.
- **A Fly Machines executor** — create a Machine per run through the Fly API,
  exec into it, destroy it when the run ends. That is the $0-at-idle answer
  and it does not exist yet; see the executor discussion in the architecture
  spec. The `Executor` trait it would implement is the same eleven methods
  `hickory-executor-docker` implements.

Until one of those is in place, set `HICKORY_EXECUTOR=local` for a deployment
that runs documents against the image's own toolchain. That works, and it is
honest about what it is: `image=` is recorded and ignored, so a document
verified there is verified only for that container's toolchain.

## Health

- `GET /api/health` — liveness.
- `GET /api/executor` — which executor is configured, so you can confirm the
  deployment is running what you think it is.
