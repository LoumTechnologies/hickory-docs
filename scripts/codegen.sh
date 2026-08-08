#!/usr/bin/env bash
# Regenerate the OpenAPI spec + typed web client. Needs no running
# infrastructure: the spec comes from `print-openapi`, which is pure
# utoipa macro output (no DB, no env vars) — see
# apps/server/src/bin/print_openapi.rs.
# Usage: scripts/codegen.sh   (via `just codegen`)
set -euo pipefail
cd "$(dirname "$0")/.."

echo "Generating OpenAPI spec..."
cargo run -q -p hickory-server --bin print-openapi > apps/server/openapi.json

echo "Generating typed web client from the spec..."
(cd apps/web && npx --no-install openapi-typescript ../../apps/server/openapi.json -o src/api/generated/schema.d.ts)

echo "Codegen done: apps/server/openapi.json, apps/web/src/api/generated/schema.d.ts"
