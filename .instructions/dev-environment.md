---
skills:
  - implement-dev-environment
---

# Local Dev Environment

- **The `just` command surface for the dev environment is exactly:** `dev`,
  `dev-stop`, `dev-clean`, `dev-seed`, `codegen`, `check-codegen`. Do not add
  more without a real, recurring need — the point of a fixed list is that it
  never becomes overwhelming to scan.
  - `dev` — idempotent full bring-up (any needed containers + migrations +
    app dev servers). Safe to re-run after a partial or killed run.
  - `dev-stop` — stops the environment without deleting data.
  - `dev-clean` — stops, then deletes **only this worktree's** containers,
    volumes, and locally-built images. Never touches another worktree's
    environment and never deletes pulled base images (postgres, redis,
    etc.) — those aren't ours to redownload.
  - `dev-seed` — idempotent seed-data population.
- **Every dev-environment `just` recipe must be incrementally idempotent.**
  Killing `just dev` halfway through (e.g. mid-migration) and re-running it
  must pick up where it left off, not fail or duplicate work.
- **When `just dev` exits, it leaves no process running** — whether it was
  interrupted, failed partway through, or finished a one-shot run. A
  surviving dev server keeps holding this worktree's ports and hostnames, so
  the next run misbehaves in ways that look like application bugs rather
  than leftovers.
- **Any Docker container the dev environment needs is started via
  docker-compose — never a bare `docker run`.** This holds even for a single
  container.
- **A devcontainer is always optional**, even when docker-compose is in use.
  Don't require one; add `devcontainer.json` only when it earns its keep.
- **Never hardcode a port, hostname, container name, or other machine-wide
  identifier** in `docker-compose.yml` or anywhere else. Use PortZero for
  port/route assignment, and put any other
  per-worktree value (e.g. a compose project name) in a single **untracked,
  gitignored config file at the repo root** — this is what lets `just dev`
  run in multiple git worktrees on the same machine at once without
  collisions.
- **The dev environment must run on macOS, Windows, and Linux.** Don't rely
  on a shell feature or path convention specific to one platform.
- **Use [direnv](https://direnv.net/) to auto-load the untracked per-worktree
  config where it makes sense** (e.g. exporting `COMPOSE_PROJECT_NAME` or a
  PortZero-derived URL into the shell on `cd`). Any `.envrc` this repo adds
  must start with `source_up_if_exists` so a developer's own parent-directory
  `.envrc` (their own unrelated direnv setup) still loads — never assume this
  repo's `.envrc` is the only one in the chain.
- **Maintain a current list of machine-level developer dependencies**
  (Docker, Node, `just`, PortZero, etc.) — see `$implement-dev-environment`
  for where this lives.
- **Document the short path to a running environment in
  `docs/developers/developer-environment.md`.** Keep it procedural (a short
  numbered list of commands), not a design document.
- **Breaking the dev environment should break CI.** If `just dev` can't
  bring up a working environment, a CI job should fail for the same reason —
  don't let dev-environment rot go undetected.
