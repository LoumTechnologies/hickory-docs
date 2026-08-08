#!/usr/bin/env bash
# Dev environment: Postgres (docker compose) + hickory-server + web (Vite),
# each reachable at a stable Port Zero domain instead of a hand-picked port.
# Usage: scripts/dev.sh [stop]   (via `just dev` / `just dev-stop`)
#
# Why Port Zero: nothing below picks a port. The server binds 0 (the kernel
# hands out a free one) and Vite falls back to its own increment-until-free
# default (it has no port-0 support). The portzero daemon watches for the
# PZ_TUNNEL env var, finds the port each process actually landed on, and
# publishes it at a fixed name — so two checkouts, or a leftover process,
# can never collide, and nothing here or in .env knows a real port number.
#
# Postgres is the deliberate exception: it keeps publishing a fixed host port
# because CI has no Postgres service and apps/server/tests/integration.rs
# bootstraps one with `docker compose up -d --wait db`, reaching it at the
# hardcoded localhost:5433 fallback. It gets a tunnel domain as well, so dev
# can use the name while tests and CI keep using the port.
set -euo pipefail
# Job control, so each background job below becomes its own process-group
# leader (pgid == pid). Without it they inherit THIS script's group and the
# `kill -- -$pid` in stop() would signal the script itself.
set -m
cd "$(dirname "$0")/.."

RUN_DIR=.dev
mkdir -p "$RUN_DIR"

if [ ! -f .env ]; then
  echo "No .env found — creating one from .env.example"
  cp .env.example .env
fi
set -a
# shellcheck disable=SC1091
source .env
set +a

# One knob decides every domain below. A git worktree that wants its own
# parallel stack sets PZ_NAMESPACE to something else in its .env and gets a
# complete second environment with zero port coordination.
NS="${PZ_NAMESPACE:-hickory}"
WEB_DOMAIN="${NS}.portzero.local"
API_DOMAIN="api.${NS}.portzero.local"
DB_DOMAIN="db.${NS}.portzero.local"

stop() {
  for name in server web; do
    if [ -f "$RUN_DIR/$name.pid" ]; then
      pid=$(cat "$RUN_DIR/$name.pid")
      if kill -0 "$pid" 2>/dev/null; then
        echo "Stopping $name (pid $pid)"
        # Kill the whole process group: `npm run dev` execs a shell that execs
        # node, so signalling only the recorded pid orphans the real listener
        # and the next `just dev` fails with "Address already in use".
        kill -- "-$pid" 2>/dev/null || kill "$pid" 2>/dev/null || true
      fi
      rm -f "$RUN_DIR/$name.pid"
    fi
  done
  docker compose stop db
  echo "Dev environment stopped."
}

if [ "${1:-}" = "stop" ]; then
  stop
  exit 0
fi

if ! command -v portzero >/dev/null 2>&1; then
  echo "portzero not found on PATH — install it from https://portzero.net" >&2
  echo "(it is what maps these services to stable *.portzero.local names)" >&2
  exit 1
fi
if ! portzero status >/dev/null 2>&1; then
  echo "Starting the portzero daemon..."
  portzero start --no-browser
fi

# A previous run that was killed without `just dev-stop` (or killed by pid,
# orphaning the real child) leaves a listener behind. Say so plainly instead
# of letting the new server die with a bare "Address already in use" buried
# in a log file nobody reads.
for name in server web; do
  if [ -f "$RUN_DIR/$name.pid" ] && kill -0 "$(cat "$RUN_DIR/$name.pid")" 2>/dev/null; then
    echo "A previous $name (pid $(cat "$RUN_DIR/$name.pid")) is still running." >&2
    echo "Run 'just dev-stop' first." >&2
    exit 1
  fi
done

echo "Starting Postgres (docker compose)..."
PZ_NAMESPACE="$NS" docker compose up -d --wait db

# The daemon only discovers a container on its next scan, so the tunnel name
# appears a beat AFTER compose reports healthy. The server connects to
# DATABASE_URL at boot and dies on an unresolvable host, so gate on the name
# actually resolving rather than on the container being up.
echo "Waiting for ${DB_DOMAIN} ..."
if ! portzero wait "$DB_DOMAIN" --timeout 60; then
  echo "Postgres tunnel ${DB_DOMAIN} never appeared — is the portzero daemon healthy?" >&2
  echo "Try: portzero doctor" >&2
  exit 1
fi

# PORT=0 → the kernel picks a free port; PZ_TUNNEL tells the daemon which name
# to publish it under. PZ_HEALTH_PATH makes `portzero wait --healthy` a real
# readiness gate rather than a "the socket is open" guess.
echo "Starting hickory-server at http://${API_DOMAIN} ..."
PORT=0 \
PZ_TUNNEL="${API_DOMAIN}:80" \
PZ_HEALTH_PATH=/api/health \
  cargo run -p hickory-server >"$RUN_DIR/server.log" 2>&1 &
echo $! >"$RUN_DIR/server.pid"

echo "Starting web dev server at http://${WEB_DOMAIN} ..."
# Redirect the SUBSHELL, not just npm inside it: otherwise the subshell keeps
# this script's stdout open for as long as the dev server runs, and anything
# piping `just dev` (a tee, a log capture) blocks forever waiting for EOF.
(cd apps/web && \
  HICKORY_API_ORIGIN="http://${API_DOMAIN}" \
  PZ_TUNNEL="${WEB_DOMAIN}:80" \
  npm run dev) >"$RUN_DIR/web.log" 2>&1 &
echo $! >"$RUN_DIR/web.pid"

# `cargo run` may need to compile first, so allow a generous window before
# calling it a failure.
echo
echo "Waiting for tunnels..."
if ! portzero wait "$API_DOMAIN" --healthy --timeout 300; then
  echo "hickory-server never came up — see $RUN_DIR/server.log" >&2
  exit 1
fi
if ! portzero wait "$WEB_DOMAIN" --timeout 120; then
  echo "web dev server never came up — see $RUN_DIR/web.log" >&2
  exit 1
fi

echo
echo "Dev environment up:"
echo "  Web:  http://${WEB_DOMAIN}          (proxies /api to the server)"
echo "  API:  http://${API_DOMAIN}/api/health"
echo "  DB:   ${DB_DOMAIN}:5432"
echo "  Logs: $RUN_DIR/server.log, $RUN_DIR/web.log"
echo "Stop with: just dev-stop"
