#!/usr/bin/env bash
# The same locked Python and real confined-worker checks run locally and in CI.
set -euo pipefail
cd "$(dirname "$0")/.."
# Rebuild the worker wheel so source edits cannot be masked by a cached install.
uv sync --project apps/agent --locked --no-editable --reinstall-package hickory-agent-worker
uv run --project apps/agent --locked --no-editable python -m unittest discover -s apps/agent/tests -v
HICKORY_AGENT_TEST_PYTHON="$PWD/apps/agent/.venv/bin/python" \
  cargo test -p hickory-cli --test python_agent -- --nocapture
cargo test -p hickory-executor-sandbox agent_process
