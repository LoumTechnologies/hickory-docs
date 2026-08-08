#!/usr/bin/env bash
# Idempotent seed data: creates a fixed set of accounts through the real
# POST /api/auth/signup endpoint (never a direct DB insert), so every seeded
# login actually works. Safe to run twice — a 409 (account already exists)
# counts as success.
# Usage: scripts/dev-seed.sh   (via `just dev-seed`)
set -euo pipefail
cd "$(dirname "$0")/.."

if [ ! -f .env ]; then
  echo "No .env found — run 'just dev' first." >&2
  exit 1
fi
set -a
# shellcheck disable=SC1091
source .env
set +a

# Same formula scripts/dev.sh uses — PortZero names are deterministic and
# resolved by the daemon at request time, so there is no port to discover.
NS="${PZ_NAMESPACE:-hickory}"
API_DOMAIN="api.${NS}.portzero.local"
BASE_URL="http://${API_DOMAIN}"

# email:password pairs. Keep this list small and documented in
# docs/developers/developer-environment.md.
SEED_ACCOUNTS=(
  "dev@hickory.local:dev-password-123"
  "owner@hickory.local:owner-password-123"
)

signup() {
  local email="$1" password="$2"
  local attempt status body

  for attempt in 1 2 3 4 5; do
    if body=$(curl -sS -w '\n%{http_code}' \
      -H 'Content-Type: application/json' \
      -d "{\"email\":\"${email}\",\"password\":\"${password}\"}" \
      "${BASE_URL}/api/auth/signup" 2>/tmp/dev-seed-curl-err); then
      status="${body##*$'\n'}"
      case "$status" in
        200 | 201)
          echo "seeded ${email}"
          return 0
          ;;
        409)
          echo "${email} already exists (ok)"
          return 0
          ;;
        *)
          echo "signup for ${email} failed with HTTP ${status}: ${body%$'\n'*}" >&2
          return 1
          ;;
      esac
    fi
    # curl itself failed (connection refused/timeout/DNS) — the tunnel may
    # still be settling. Retry only this case, never a real HTTP response.
    echo "connecting to ${BASE_URL} (attempt ${attempt}/5)..." >&2
    sleep 2
  done
  echo "could not reach ${BASE_URL} — is 'just dev' running?" >&2
  cat /tmp/dev-seed-curl-err >&2 2>/dev/null || true
  return 1
}

for pair in "${SEED_ACCOUNTS[@]}"; do
  signup "${pair%%:*}" "${pair#*:}"
done
