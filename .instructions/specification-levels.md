---
skills:
  - guarantee-maintainer
---

# Specs and guarantees

Maintain two levels of specs:

- High-level, freeform English specs live in `docs/specs/freeform/`.
- Strict behavioral guarantees live in `docs/guarantees/`.

Each guarantee gets its own Markdown file with a header, the guarantee text, a
horizontal rule, and the latest LLM verification notes. Organize guarantee files
into subdirectories when useful. Verification notes must reference concrete
files, functions, modules, tests, and caveats so the implementation can be found
quickly.

When implementing ticketry tickets or making behavior changes, add, update, or
remove guarantees as needed. Prefer tests for guarantees whenever practical. Any
test that exists to protect one or more guarantees must include a comment naming
the guarantee file path or paths. If a guarantee cannot be fully protected by
tests yet, say that in the guarantee's verification notes and explain what still
requires LLM review.
