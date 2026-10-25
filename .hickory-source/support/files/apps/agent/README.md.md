<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `4dbb88170e0ceab433f1bfe8426e8e5bae0d779019ff8aaec18d606821d9b4cd`.

<book:entry path="apps/agent/README.md">
# Hickory Python agent

Python owns the agent loop. `src/hickory_agent_worker/core.py` defines the
LangGraph graph, prompts, history, requests, retries, actions and stopping policy.
It takes injectable model/tool clients and an executor. It imports no Hickory
process transport or UI code. Model snippets run separately in Monty.

Run without Rust or a provider account:

```sh
uv sync --project apps/agent --locked --no-editable
printf '[{"python":"tool(\"list_docs\", {})"},{"done":"Inspected the fixture"}]\n' > /tmp/hickory-responses.json
uv run --project apps/agent --locked --no-editable hickory-agent-dev \
  --responses /tmp/hickory-responses.json 'Inspect the development workspace'
uv run --project apps/agent --locked --no-editable python -m unittest discover -s apps/agent/tests -v
```

The default development workspace is a temporary directory deleted on exit.
This mode has read-only fixture tools and makes no Hickory confinement or
verification claim. See [the contributor guide](../../docs/developers/python-agent.md)
for real providers, client injection and the confined Hickory connection.
</book:entry>

</book:doc>
