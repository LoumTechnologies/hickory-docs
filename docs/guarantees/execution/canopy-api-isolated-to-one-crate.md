# Only `hickory-executor-canopy` Knows Cloud Canopy's API

Given that Cloud Canopy's API is under concurrent development, when any code
in this repository talks to canopy (GraphQL, gRPC, sandbox channel protocol,
image path mapping), then that code lives in `crates/hickory-executor-canopy`
and nowhere else; all other code depends only on the `Executor` trait in
`crates/hickory-executor`.

---

Last LLM verification:
- Date: 2026-08-05
- Reviewer: Claude (Fable 5)
- Result: not verified (implementation pending — Phase B)
- Evidence: design in `docs/specs/freeform/architecture.md` and
  `docs/specs/freeform/canopy-integration.md`.
- Test coverage: enforceable by grep in CI (`canopy` outside the adapter
  crate) — to be added.
