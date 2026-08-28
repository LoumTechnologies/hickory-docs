#!/usr/bin/env bash
# Dev environment: the desktop app, running the real engine, on a seeded
# scratch project.
#
# Usage: scripts/dev.sh [stop|clean]   (via `just dev` / `dev-stop` / `dev-clean`)
#
# This replaced a script that started Postgres and a `hickory-server` — both
# left over from a hosted product that no longer exists (see
# docs/specs/freeform/local-only.md). There is no database, no API server, and
# nothing to log into: the desktop app runs `hickory_cli::serve` in its own
# process, so `just dev` is that app, in dev mode, against documents on disk.
#
# Everything it needs is brought up to date on every run — frontend
# dependencies, the `hick` binary, and the seeded scratch project — because a
# dev environment that shows you something out of date, and cures it with a
# command you were supposed to know about, is worse than one that is slow.
#
# What it starts:
#   * Vite, serving apps/web with hot reload, on a port derived from this
#     worktree's path (below).
#   * The Tauri shell, pointed at that Vite, with the engine in-process.
# `cargo tauri dev` owns both: Vite is its `beforeDevCommand`, and it stops
# when the app window closes — which is the rule that `just dev` leaves
# nothing running when it exits.
set -euo pipefail
set -m
cd "$(dirname "$0")/.."

RUN_DIR=.dev
PROJECT_DIR="$RUN_DIR/project"

# Dogfooding: `HICKORY_PROJECT_DIR=~/notes just dev` opens the app on a real
# folder instead of the seeded scratch. Without this the dev environment can
# only ever be pointed at fixtures, which is the one project whose problems
# are already known. The seeder still owns .dev/project and still runs, so
# `just dev` with nothing set is unchanged.
DOGFOOD_DIR="${HICKORY_PROJECT_DIR:-}"

# Per-worktree, derived, never written down: two checkouts of this repo can
# each run `just dev` without agreeing on anything. 41000–48999 avoids the
# ephemeral range and Vite's own 5173 default, so a stray `npm run dev`
# elsewhere cannot answer for this one.
worktree_port() {
  local slug
  slug=$(pwd | cksum | cut -d' ' -f1)
  echo $((41000 + slug % 8000))
}

# The engine's port sits directly above the UI's. Derived the same way and
# from the same slug, so the pair moves together between worktrees and neither
# is written down anywhere.
api_port() {
  echo $(( $(worktree_port) + 1 ))
}

stop() {
  # The window closing is the normal exit; this is for a run that was killed
  # in a way that left the vite child behind.
  if [ -f "$RUN_DIR/vite.pid" ]; then
    local pid
    pid=$(cat "$RUN_DIR/vite.pid")
    if kill -0 "$pid" 2>/dev/null; then
      echo "Stopping the dev server (pid $pid)…"
      kill -- "-$pid" 2>/dev/null || kill "$pid" 2>/dev/null || true
    fi
    rm -f "$RUN_DIR/vite.pid"
  fi
  # Anything still holding either of this worktree's ports is ours by
  # construction. The engine's matters as much as Vite's now: a named port
  # cannot be stepped around the way an ephemeral one could.
  if command -v fuser >/dev/null 2>&1; then
    fuser -k "$(worktree_port)/tcp" 2>/dev/null || true
    fuser -k "$(api_port)/tcp" 2>/dev/null || true
  fi
  # Those kills are SIGKILL, so the engine never runs the drop that releases
  # its directory lock — and the next `just dev` refuses to start on a lock
  # held by a process that no longer exists. Clearing it here is what keeps
  # the rule in .instructions/dev-environment.md true: a killed run must be
  # re-runnable, not a puzzle. Only ever THIS worktree's scratch lock.
  rm -f "$PROJECT_DIR/.hick-cache/up.lock"
  echo "Stopped."
}

clean() {
  stop
  # Only this worktree's scratch. Never another checkout's, and never the
  # cargo/npm caches — those are not ours to make anyone download again.
  rm -rf "$RUN_DIR"
  echo "Removed $RUN_DIR (this worktree's dev scratch only)."
}

case "${1:-}" in
  stop)
    stop
    exit 0
    ;;
  clean)
    clean
    exit 0
    ;;
  "") ;;
  *)
    echo "usage: scripts/dev.sh [stop|clean]" >&2
    exit 2
    ;;
esac

# --- prerequisites, each named with the one command that fixes it ------------

missing=0
if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is not on your PATH." >&2
  echo "  Install Rust: https://rustup.rs" >&2
  missing=1
fi
if ! command -v npm >/dev/null 2>&1; then
  echo "error: npm is not on your PATH (the UI is a Vite app)." >&2
  echo "  Install Node 20 or newer: https://nodejs.org" >&2
  missing=1
fi
if ! command -v cargo-tauri >/dev/null 2>&1; then
  echo "error: cargo-tauri is not installed — it is what runs the desktop app." >&2
  echo "  cargo install tauri-cli --version '^2' --locked" >&2
  missing=1
fi
if [ "$(uname -s)" = "Linux" ] && command -v pkg-config >/dev/null 2>&1; then
  if ! pkg-config --exists webkit2gtk-4.1; then
    echo "error: the system webview is missing (Tauri renders through it)." >&2
    echo "  Debian/Ubuntu: sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev \\" >&2
    echo "                                  librsvg2-dev patchelf" >&2
    missing=1
  fi
fi
[ "$missing" -eq 0 ] || exit 1

# One-time, idempotent: nobody has a separate "install the hook" step.
if [ "$(git config --get core.hooksPath || true)" != ".githooks" ]; then
  git config core.hooksPath .githooks
fi

mkdir -p "$RUN_DIR"

# Reinstall when the lockfile is newer than what is installed, not only when
# nothing is installed at all. A pull that changes package-lock.json otherwise
# leaves the old dependency tree in place, and the UI that opens is not the UI
# in this checkout — with `npm ci` as a step nobody told you to run.
if [ ! -d apps/web/node_modules ] || [ apps/web/package-lock.json -nt apps/web/node_modules ]; then
  echo "Installing frontend dependencies…"
  npm --prefix apps/web ci
  touch apps/web/node_modules
fi

# Build the engine's CLI up front and hand it to the seeder, so the scratch
# project is woven by this checkout's code. `cargo tauri dev` compiles the same
# workspace into the same target directory a moment later, so this shares that
# work rather than duplicating it.
echo "Building hick (no-op if it is already current)…"
cargo build --quiet --bin hick
export HICK="$PWD/target/debug/hick"

# Every run, not only the first. The seed is idempotent and knows the
# difference between a file you edited and a file that is merely old, so
# re-running it is what keeps the scratch project equal to the fixtures in this
# checkout instead of frozen at whenever the folder happened to be created.
./scripts/dev-seed.sh

PORT=$(worktree_port)
API_PORT=$(api_port)
export HICKORY_PROJECT_DIR="${DOGFOOD_DIR:-$PWD/$PROJECT_DIR}"
# The engine binds this instead of an ephemeral port, and the window loads
# Vite instead of the engine. Both are read by the desktop shell at startup
# and both are absent in a downloaded copy — see dev.rs.
export HICKORY_SERVE_PORT="$API_PORT"
export HICKORY_UI_ORIGIN="http://localhost:$PORT"
# What Vite proxies `/api` to. Read by apps/web/vite.config.ts.
export HICKORY_API_ORIGIN="http://127.0.0.1:$API_PORT"

echo "Dev environment"
if [ -n "$DOGFOOD_DIR" ]; then
  echo "  project : $HICKORY_PROJECT_DIR (yours, from HICKORY_PROJECT_DIR)"
else
  echo "  project : $HICKORY_PROJECT_DIR (seeded scratch)"
fi
echo "  ui      : http://localhost:$PORT — Vite, and what the window loads."
echo "            Frontend changes hot-reload; open it in a browser too."
echo "  engine  : http://127.0.0.1:$API_PORT — in the app's own process."
echo "            Vite proxies /api here, WebSocket included."
echo "Close the window to stop everything."
echo

# `--config` rather than editing tauri.conf.json: the committed config is a
# release artifact, and a dev port written into it would be a hardcoded port
# in a tracked file — the thing .instructions/dev-environment.md forbids.
#
# The npm prefix is ABSOLUTE and the CLI runs from the crate directory, both
# on purpose: `cargo tauri` picks the before-dev command's working directory
# by hunting for a package.json, and the hunt broke the day an example
# gained one (examples/service — scaffolded-service's committed output): the
# command ran there, `../web` pointed at examples/web, and `just dev` died
# on a path that never existed. An absolute prefix is immune to wherever the
# hunt lands, and the crate directory is the same cwd
# scripts/dist-desktop.sh already uses for the same reason.
ROOT="$PWD"
cd apps/desktop/src-tauri
exec cargo tauri dev \
  --config "{\"build\": {\"devUrl\": \"http://localhost:$PORT\", \"beforeDevCommand\": \"npm --prefix $ROOT/apps/web run dev -- --port $PORT --strictPort\"}}"
