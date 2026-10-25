<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .claude/skills/implement-dev-environment/agents/openai.yaml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `ab3f86e0cced207bbc4a6925702735186b6c121e17eccc7148dd921e64e6a61d`.

<book:entry path=".claude/skills/implement-dev-environment/agents/openai.yaml">
interface:
  display_name: "Implement Dev Environment"
  short_description: "Wire up a worktree-safe, offline-friendly local dev environment"
  default_prompt: "Use $implement-dev-environment to set up this repo's local dev environment: just dev/dev-stop/dev-clean/dev-seed via docker-compose (never a bare docker run), no hardcoded ports/names so multiple git worktrees can run at once (PortZero + an untracked per-worktree config file), an auto-generated REST API client from the OpenAPI spec (openapi-typescript + openapi-fetch, never hand-edited or hand-merged), graceful offline degradation for any combination of present/absent third-party accounts, idempotent seed data with real logins, and pre-commit hooks that can never pass something CI then fails."
</book:entry>

</book:doc>
