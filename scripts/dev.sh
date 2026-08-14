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

# Per-worktree, derived, never written down: two checkouts of this repo can
# each run `just dev` without agreeing on anything. 41000–48999 avoids the
# ephemeral range and Vite's own 5173 default, so a stray `npm run dev`
# elsewhere cannot answer for this one.
worktree_port() {
  local slug
  slug=$(pwd | cksum | cut -d' ' -f1)
  echo $((41000 + slug % 8000))
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
  # Anything still holding this worktree's port is ours by construction.
  local port
  port=$(worktree_port)
  if command -v fuser >/dev/null 2>&1; then
    fuser -k "$port/tcp" 2>/dev/null || true
  fi
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

if [ ! -d apps/web/node_modules ]; then
  echo "Installing frontend dependencies…"
  npm --prefix apps/web ci
fi

# Seeding is idempotent and cheap, so `just dev` does it rather than making
# an empty first run look broken.
if [ ! -d "$PROJECT_DIR" ] || [ -z "$(ls -A "$PROJECT_DIR" 2>/dev/null)" ]; then
  ./scripts/dev-seed.sh
fi

PORT=$(worktree_port)
export HICKORY_PROJECT_DIR="$PWD/$PROJECT_DIR"

echo "Dev environment"
echo "  project : $HICKORY_PROJECT_DIR"
echo "  vite    : http://localhost:$PORT (derived from this worktree)"
echo "  app     : the window that opens; it runs the engine in-process"
echo "Close the window to stop everything."
echo

# `--config` rather than editing tauri.conf.json: the committed config is a
# release artifact, and a dev port written into it would be a hardcoded port
# in a tracked file — the thing .instructions/dev-environment.md forbids.
exec cargo tauri dev \
  --config "{\"build\": {\"devUrl\": \"http://localhost:$PORT\", \"beforeDevCommand\": \"npm --prefix ../web run dev -- --port $PORT --strictPort\"}}"
