#!/usr/bin/env bash
# Dev environment: Postgres (docker compose) + hickory-server + web (Vite).
# Usage: scripts/dev.sh [stop]   (via `just dev` / `just dev-stop`)
set -euo pipefail
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

stop() {
  for name in server web; do
    if [ -f "$RUN_DIR/$name.pid" ]; then
      pid=$(cat "$RUN_DIR/$name.pid")
      if kill -0 "$pid" 2>/dev/null; then
        echo "Stopping $name (pid $pid)"
        kill "$pid" 2>/dev/null || true
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

echo "Starting Postgres (docker compose)..."
docker compose up -d --wait db

echo "Starting hickory-server on port ${PORT:-8080}..."
cargo run -p hickory-server >"$RUN_DIR/server.log" 2>&1 &
echo $! >"$RUN_DIR/server.pid"

echo "Starting web dev server on port ${WEB_PORT:-5173}..."
(cd apps/web && HICKORY_SERVER_PORT="${PORT:-8080}" WEB_PORT="${WEB_PORT:-5173}" npm run dev >"../../$RUN_DIR/web.log" 2>&1) &
echo $! >"$RUN_DIR/web.pid"

echo
echo "Dev environment up:"
echo "  API:  http://localhost:${PORT:-8080}/api/health"
echo "  Web:  http://localhost:${WEB_PORT:-5173}  (proxies /api to the server)"
echo "  Logs: $RUN_DIR/server.log, $RUN_DIR/web.log"
echo "Stop with: just dev-stop"
