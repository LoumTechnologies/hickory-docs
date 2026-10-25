<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# scripts/check-python-agent.sh

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Review investigations link observed agent work without resolving human review](../../contract.md)

Captured SHA-256: `34c8f51a2719ece53055677ca01c96f39d44f34236b0514ef0868d94dcb5ebdc`.

<book:entry path="scripts/check-python-agent.sh" executable="true">
#!/usr/bin/env bash
# The same locked Python and real confined-worker checks run locally and in CI.
set -euo pipefail
cd "$(dirname "$0")/.."
# Rebuild the worker wheel so source edits cannot be masked by a cached install.
uv sync --project apps/agent --locked --no-editable --reinstall-package hickory-agent-worker
uv run --project apps/agent --locked --no-editable python -m unittest discover -s apps/agent/tests -v
HICKORY_AGENT_TEST_PYTHON="$PWD/apps/agent/.venv/bin/python" \
  cargo test -p hickory-cli --test python_agent --test serve_python_agent -- --nocapture
cargo test -p hickory-executor-sandbox agent_process

(cd apps/web && npx playwright install chromium)
scripts/test-python-agent-ui.sh
</book:entry>

</book:doc>
