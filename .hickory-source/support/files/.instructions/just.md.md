<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .instructions/just.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `282c1b5a96c65a4a88d06ec84836fd61f44ed14e042e5058f692b99a38fd6572`.

<book:entry path=".instructions/just.md">
# Just as the task runner

- Always use [`just`](https://github.com/casey/just) as the task runner for standalone scripts and recurring repo operations (dev, test, build, deploy helpers, seed, launch-state, etc.).
- Always place the `justfile` at the **repo root** so recipes can be run from any directory in the repo (`just` walks parents for a justfile).
- Always write standalone scripts invoked by recipes in the **primary programming language** of the repository. If multiple languages are in use, use the primary **backend** language.
- If it is not clear which language is primary, ask the user and record the choice in the project-level agent instructions (`AGENTS.md` / `CLAUDE.md`) so later sessions do not re-ask.
- Prefer adding a `just` recipe over documenting a multi-step shell one-liner. New recurring operations become recipes, not tribal knowledge.
</book:entry>

</book:doc>
