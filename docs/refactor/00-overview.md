# Refactor Overview

Five categories of improvement, ordered by impact-to-effort ratio.

| # | Document | Theme | Effort |
|---|----------|-------|--------|
| 1 | [01-crate-naming.md](01-crate-naming.md) | Rename generic/inconsistent crates | Low–Medium |
| 2 | [02-shim-crate-elimination.md](02-shim-crate-elimination.md) | Collapse pure re-export shims | Medium |
| 3 | [03-dead-code-and-cruft.md](03-dead-code-and-cruft.md) | Delete unused extensions, scratch files | Low |
| 4 | [04-directory-organization.md](04-directory-organization.md) | Re-home misplaced crates | Medium |
| 5 | [05-doc-and-policy-accuracy.md](05-doc-and-policy-accuracy.md) | Fix CLAUDE.md, spec gaps | Low |

## Why this matters

The repo has grown through rapid iteration and consolidation work (task-43:
retire `iron-peak`/`copper-ledge`, task-27: config-driven extension wiring,
task-36: XML session logging). Each consolidation left behind scaffolding that
was never cleaned up: re-export shims that outlived their purpose, generic
crate names from early prototyping, and informal notes at the repo root.

The result is a codebase where:
- A new contributor can't tell what `network`, `bridge`, or `container` refer to
  without reading the workspace Cargo.toml
- Two crates (`hick-agent-engine`, `hick-agent-core`) exist solely to forward
  symbols from other crates
- An entire extension (`hick-agent-mcp-graphql`) is compiled but never enabled
- The architectural guidance in CLAUDE.md describes a binary-per-config pattern
  that was already retired

None of these are blockers, but they accumulate as tax on every change.

## Recommended execution order

1. **Dead code and cruft first** (doc #3): no refactoring risk, immediate
   clarity gain, frees reviewers from chasing dead ends.
2. **Crate naming** (doc #1): `cargo rename` + sed pass; changes are mechanical
   and contained to `Cargo.toml` files and `use` statements.
3. **Shim elimination** (doc #2): depends on naming being settled first; choose
   one option per shim before starting.
4. **Directory re-homing** (doc #4): do after shims are resolved; fewer crates
   means fewer moves.
5. **Doc/policy accuracy** (doc #5): last, so changes reflect the final state.
