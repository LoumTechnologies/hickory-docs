---
skills:
  - audience-first-docs
---

# Documentation layout

- In the docs folder in the repo root, maintain a sub folder for each target audience we need documentation for. Place markdown files, potentially in nested structures inside each of these sub folders.
  - Each sub folder should contain a target-audience.md file that describes the target audience.
  - Always keep the target audience in mind when writing documentation.
  - Maintain a docs/developers folder with markdown files for developers working on this codebase.
  - If there are different kinds of users, maintain separate folders for each.
  - If there is only one known kind of user, maintain a single folder called docs/users.
- **Guarantees and freeform specs are not audience folders:** keep them as sibling trees `docs/guarantees/` and `docs/specs/freeform/` under `docs/` (see the `guarantee-maintainer` skill); do not nest them inside `docs/users/` or `docs/developers/`.
