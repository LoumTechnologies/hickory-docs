---
name: implement-dev-environment
description: >
  Wire up a repo's local development environment so `just dev` is the one
  true way to get a working environment: docker-compose (never a bare
  `docker run`) isolated per git worktree with no hardcoded ports/names,
  auto-generated REST API clients from the server's OpenAPI spec (never
  hand-written or hand-merged), graceful offline degradation for any
  combination of present/absent third-party accounts, idempotent seed data
  with real logins, and pre-commit hooks that can never pass something CI
  then fails. Use when asked to set up or fix a dev environment, add
  docker-compose, generate an API client, add seed data, wire PortZero into
  local dev, or make pre-commit/CI checks selective by changed files.
  Complements $plan-deploy-shared (staging/production infra this skill's dev
  environment must stay parity-adjacent to, per config-and-environments) and
  $just (the task runner this skill's whole command surface sits on top of).
---

# Implement Dev Environment

## Goal

One command, `just dev`, brings up a fully working local environment —
without touching the cloud, without a fixed port or container name that
collides across git worktrees on the same machine, and without requiring
every third-party account the app integrates with. The same environment
degrades gracefully (never crashes) when a credential or the internet is
missing, and breaks CI the same way it breaks for a developer, so drift
never goes unnoticed.

This skill does **not** invent new tooling. It orchestrates: `just` (task
runner), docker-compose (the only way this skill starts a container),
PortZero (port/route assignment), the target framework's own OpenAPI
support (spec generation), and openapi-typescript + openapi-fetch (client
generation) — see `references/api-client-codegen.md` for why that pairing
over a generator that needs a JVM or a running server.

## The `just` command surface

Exactly six recipes, no more (see `$dev-environment` for the authoritative
list and behavior contract):

| Recipe | Concern |
|---|---|
| `dev` | Idempotent full bring-up |
| `dev-stop` | Stop without deleting data |
| `dev-clean` | Delete this worktree's containers/volumes/local images only |
| `dev-seed` | Idempotent seed data, with real logins |
| `codegen` | Regenerate API client(s) from the OpenAPI spec, no infra needed |
| `check-codegen` | `codegen` + fail with the fix command if anything changed |

Don't add a seventh recipe to solve a one-off need — extend an existing one
or write a project-local script it calls.

This list is the *dev-environment* surface, not a cap on the whole
justfile — a repo's existing `test`/`build`/`lint` recipes are unaffected,
and the changed-file check entrypoint the hook and CI share lives outside it
(see `references/pre-commit-and-selective-ci.md`).

**Every recipe must be a single-line hand-off to a script.** The environment
has to run on Windows as well as macOS and Linux, and under `just`'s
`windows-shell` neither shebang recipes nor `cd foo && bar` work. Put all
logic in the script; let `just` only name it:

```just
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

dev:
    @node scripts/dev-cli.mjs dev
```

## Workflow

1. **Confirm the primary backend language** (per `$just`) if not already
   recorded in `AGENTS.md`/`CLAUDE.md` — every script these recipes call
   into is written in it.
   - **One bootstrap file is exempt, out of necessity.** If the language
     needs installed tooling to run (TypeScript needs `ts-node`/`tsc`), the
     scripts can't run before `npm install` — and installing is `just dev`'s
     own first step. Resolve it with a single dependency-free entrypoint in
     the runtime's native form (e.g. `scripts/dev-cli.mjs`) whose only job is
     to install if needed, register the transpiler, and hand off. Everything
     past that file stays in the project language. Don't rewrite the whole
     surface in the lower-level language to avoid this, and don't require a
     manual install step before `just dev` works.
2. **Scaffold `docker-compose.yml` only if the stack actually needs a
   container** (database, queue, etc.). A pure-language dev server needs no
   compose file at all. When compose is used:
   - No fixed `container_name`, `ports:` host-side literal, or network name.
     Let compose assign ephemeral host ports; route to them via PortZero.
   - The compose **project name** comes from the untracked per-worktree
     config file (step 3), passed as `-p`/`COMPOSE_PROJECT_NAME`, so two
     worktrees never share a network/volume namespace.
   - A `devcontainer.json` is optional — add one only if it earns its keep,
     never as a default alongside compose. See `references/docker-compose-worktrees.md`.
3. **Generate the untracked config file** (e.g. `.dev-env.local`, gitignored)
   at the repo root on first `just dev` if it doesn't exist yet: derive a
   stable-but-unique project name/identifier from the worktree's absolute
   path (e.g. a short hash), write it once, and have every recipe read it.
   Never hand a human a step that requires editing this file — it's
   machine-generated and worktree-local. Full schema in
   `references/docker-compose-worktrees.md`.
4. **Wire PortZero** for any service that listens on HTTP: launch it with
   `PZ_TUNNEL=<name>:80` set while it listens on an ephemeral port, and have
   `just dev` print the routable URL — never a hardcoded `localhost:XXXX` in
   docs or code. Treat PortZero as optional (fall back to OS-assigned ports
   when the daemon isn't running), and verify the server actually honored
   port 0 rather than silently substituting its own default. Mechanics and
   both gotchas in `references/docker-compose-worktrees.md`.
5. **Write the four lifecycle recipes** (`dev`, `dev-stop`, `dev-clean`,
   `dev-seed`) as thin `just` wrappers around scripts in the primary
   backend language. Every one must be safe to kill and re-run — check
   "is this already done" before doing it, don't assume a clean start.
   `dev` supervises long-running servers, so its teardown matters as much as
   its startup: kill process *groups*, wait for them to actually be gone,
   tear down on the failure path, and drop signal handlers so a one-shot run
   can exit. See `references/process-lifecycle.md` — leaked servers holding
   a PortZero name are the most expensive failure mode in this whole skill.
6. **Set up API client codegen** (`codegen`, `check-codegen`):
   - Add a static OpenAPI-spec generation entrypoint that needs **no live
     database or running server** (see `references/api-client-codegen.md`
     for the concrete no-DB-required trick for frameworks whose app
     bootstrap normally requires one, e.g. Nest + TypeORM).
   - Generate the client with openapi-typescript + openapi-fetch from that
     spec.
   - Mark the generated path(s) in `.gitattributes`
     (`<path>/** linguist-generated=true -diff`).
   - Wire the CI drift check (`check-codegen`) as a required PR check into
     `main`, with a failure message that states the exact fix. Wire the
     same check into the pre-commit hook so it's caught before push, per
     `$pre-commit-ci-parity`.
   - Document the "never hand-resolve, always regenerate" conflict
     procedure in `docs/developers/developer-environment.md` in plain,
     no-dev-environment-required language.
7. **Write `dev-seed`** so every seeded account is created through the
   app's real signup/registration path (or an equivalent that produces a
   working, loggable-in-with credential) — never a direct DB insert that
   leaves an account nobody can authenticate as. List the seeded
   usernames/emails/passwords in `docs/developers/developer-environment.md`.
   Because it runs in its own process, it can't know an OS-assigned port:
   have `dev` write the resolved base URL to the untracked state directory
   for it to read. Treat "account already exists" as success (that's what
   makes it idempotent), and retry *connection* failures — but never an HTTP
   response — so a settling tunnel can't make the check flaky.
8. **Handle each third-party integration per `$third-party-integration-mocking`**:
   for each one, decide and document (in
   `references/third-party-mocking.md`'s per-project companion notes, or
   inline near the integration's config) what happens with the credential
   present, absent, and with no internet connection. Only add a mock after
   explicitly deciding graceful degradation isn't enough.
9. **Wire pre-commit hooks and selective CI paths** per
   `$pre-commit-ci-parity` — see `references/pre-commit-and-selective-ci.md`
   for the shared change-detection mechanism both must call into so they
   can't drift apart. Install the hook from `just dev` itself
   (`git config core.hooksPath …`) so nobody has a separate step to forget.
   Give the dev environment a smoke mode and make it one of the selectable
   checks, so a broken `just dev` fails CI the same way it fails a
   developer (`references/process-lifecycle.md`).
10. **Maintain the machine-dependency list** (Docker, Node, `just`,
    PortZero, language toolchains) and write/refresh
    `docs/developers/developer-environment.md` as a short, numbered,
    copy-pasteable path from clone to running app.
11. **Run the acceptance checklist below.** Reasoning that the design is
    correct is not evidence that it works; every item is cheap and each one
    corresponds to a rule above that is easy to satisfy on paper and miss in
    practice.

## Acceptance checklist

Actually execute these — don't reason about them. Report honestly which ones
you ran and which you only verified by construction.

- [ ] `just dev` on a clean checkout, with no `.env` files and no
      `node_modules`, reaches a serving stack.
- [ ] Kill `just dev` partway through (mid-install, mid-migration) and
      re-run — it completes without manual cleanup.
- [ ] Ctrl-C leaves **zero** processes behind (`pgrep -af` the server
      binaries; don't count your own `grep`).
- [ ] Two worktrees of the same repo run `just dev` simultaneously without
      colliding.
- [ ] `just codegen` succeeds with no database, no running server, and no
      network.
- [ ] `just check-codegen` passes on a clean tree *and* on a tree where the
      generated output is staged but unmodified.
- [ ] `dev-clean` removes this worktree's volumes and leaves pulled base
      images (`docker images`) and other worktrees untouched.
- [ ] `dev-seed` run twice in a row succeeds both times, and a seeded
      account can actually log in.
- [ ] The environment comes up with no third-party credentials configured.
- [ ] A docs-only change selects no checks; a backend change selects the
      backend suite plus the codegen drift check.

## Guardrails

- Never start a container with a bare `docker run` — docker-compose only,
  even for a single container.
- Never hardcode a port, hostname, or container name anywhere a second
  worktree on the same machine would collide with it.
- Never hand-edit a generated API client file, and never hand-resolve a
  merge conflict inside one — regenerate.
- Never require a specific third-party account to be present for the dev
  environment to come up — missing credentials degrade gracefully.
- Never add a `just` recipe beyond the six listed above without a real,
  recurring need.
- Never let `dev-clean` touch another worktree's containers/volumes, or
  delete a pulled (not locally built) base image.
- Never let CI or a pre-commit hook run more than what the changed files
  actually require — but never let `main` run less than everything.
- Never leave a dev server running after `dev` exits, by any path — Ctrl-C,
  a failed bring-up, or a completed smoke run. A stale process holding a
  PortZero name makes the *next* run fail in ways that look like application
  bugs.
- Never assume a framework honors `listen(0)`, tolerates a failed database
  connection, or propagates signals to its children. Each of these is
  commonly false; check the one in front of you.

## References

Read these when the task touches their area:

- `references/docker-compose-worktrees.md` — untracked config file schema,
  compose project naming, PortZero wiring (`PZ_TUNNEL`), the optional-daemon
  fallback, servers that ignore port 0, `dev-clean` semantics.
- `references/process-lifecycle.md` — starting and stopping long-running dev
  servers without leaking them: process groups, waiting for real exit,
  signal handlers that block exit, address discovery between services, and
  the CI smoke-mode pattern.
- `references/api-client-codegen.md` — openapi-typescript + openapi-fetch
  pipeline, the no-DB-required OpenAPI spec generation approach (read this
  before writing the generator; the obvious approach does not work), the
  `.gitattributes` + CI-drift-check mechanics, the conflict-resolution
  procedure.
- `references/third-party-mocking.md` — per-integration
  present/absent/offline decision matrix (SendGrid, Stripe, Twilio,
  PostHog, S3-as-local-folder) and when a mock is actually worth adding.
- `references/pre-commit-and-selective-ci.md` — the shared changed-file
  scoping mechanism for hooks and CI, and the full-suite-on-`main` rule.
