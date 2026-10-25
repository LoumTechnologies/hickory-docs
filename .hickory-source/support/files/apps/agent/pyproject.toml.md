<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/pyproject.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `2054c49b5efe0871c24444b2b6298f4464559f7707bf5dfb23be7e5ab6845d97`.

<book:entry path="apps/agent/pyproject.toml">
[project]
name = "hickory-agent-worker"
version = "0.1.0"
description = "Independent Python agent with a confined Hickory adapter"
requires-python = ">=3.11"
dependencies = ["langgraph>=1.0,<2", "pydantic-monty==1.1.0", "jsonschema>=4.23,<5"]

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.build.targets.wheel]
packages = ["src/hickory_agent_worker"]

[project.scripts]
hickory-agent-dev = "hickory_agent_worker.cli:main"
</book:entry>

</book:doc>
