# Deploying to Fly.io

The server, the web app, and the LSP bridge are one image. `fly.toml` and
`Dockerfile` in the repository root are the whole deployment.

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
