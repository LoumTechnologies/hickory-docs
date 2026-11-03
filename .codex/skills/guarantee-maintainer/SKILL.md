---
name: guarantee-maintainer
description: Maintain and discover strict software guarantee specifications. Use when Codex needs to infer guarantees from an existing codebase, create an initial docs/guarantees setup, add/update/remove/audit/verify guarantee Markdown files, move or organize high-level specs, connect tests to behavioral guarantees, record LLM verification evidence and untestable caveats, or report likely bugs where code appears intended to provide a guarantee but may not.
---

# Guarantee Maintainer

Use this workflow when a repo has, or should have, a two-level specification system:
freeform design specs plus strict guarantee specs.

## Core Workflow

1. Locate the repo's guarantee directory and local instructions. Common defaults:
   `docs/guarantees/` for strict guarantees and `docs/specs/freeform/` for
   freeform specs. These are sibling trees under `docs/`, not audience folders
   (`docs/users/`, `docs/developers/`); audience docs and guarantees coexist
   without nesting one inside the other.
2. Read existing guarantee files before changing behavior. Treat them as product
   contracts, not comments.
3. When behavior changes, decide whether guarantees must be added, updated, moved,
   or removed. Keep edits in the same change as the implementation whenever
   practical.
4. Prefer tests for each guarantee. When adding or updating tests, include a code
   comment naming the guarantee file path or paths protected by that test.
5. If a guarantee cannot be fully protected by tests, record why in the guarantee's
   LLM verification section.
6. Verification notes must cite concrete files, functions, modules, structs, tests,
   and caveats. Do not mark a guarantee verified unless you inspected the relevant
   implementation path.

## Initial Guarantee Audit

Use this mode when asked to point the skill at a codebase and infer the guarantees
it appears to provide.

1. Read repo instructions, existing specs, public APIs, tests, and critical modules.
   Prefer project search tools when available, then inspect full files only where
   needed.
2. Identify candidate guarantees from behavior exposed in APIs, tests, docs,
   invariants, replay/sync/persistence paths, authorization checks, validation,
   error handling, and compatibility boundaries.
3. Separate findings into:
   - implemented guarantees with concrete evidence;
   - partially implemented or ambiguous guarantees;
   - intended guarantees that appear buggy or unsupported;
   - missing tests or untestable areas needing refactor hooks.
4. Create guarantee files only for guarantees that can be stated as current or
   desired product contracts. Use `verified` only when the inspected code and
   tests support the guarantee as written. Use `partially verified` or
   `not verified` when evidence is incomplete.
5. Write an audit report when the user asks for a broad inference pass. Put it in
   the repo's guarantee area when appropriate, for example
   `docs/guarantees/audits/YYYY-MM-DD-initial-audit.md`. The report is for human
   review and should list candidate guarantees created, suspected bugs, weak
   evidence, missing tests, and areas not inspected.
6. Do not call LLM reasoning "proof" unless the repo has formal proof artifacts.
   Use terms like evidence, verification notes, implementation review, or
   confidence. Cite formal proofs separately when they exist.

For suspected bugs, use code-review style: lead with the behavioral risk, cite the
file/function/test evidence, explain which guarantee would be violated, and avoid
rewriting the guarantee to hide the bug.

## Guarantee File Shape

Use one Markdown file per guarantee. Subdirectories are allowed.

```markdown
# Short Guarantee Name

Given ...
When ...
Then ...

---

Last LLM verification:

- Date: YYYY-MM-DD
- Reviewer: model or agent name
- Result: verified | partially verified | not verified
- Evidence: cite implementation files, functions, modules, structs, and tests.
- Test coverage: cite tests, or explain why no useful regression test exists yet.
```

Given/when/then is the default, but use clearer prose when that structure becomes
awkward.

## Verification Standards

- `verified`: the inspected code and tests support the guarantee as written.
- `partially verified`: important parts are supported, but caveats or uninspected
  paths remain.
- `not verified`: the guarantee is desired or newly recorded, but implementation
  evidence has not been established.

Be precise about scope. Narrow an overbroad guarantee instead of relying on vague
verification language.

## Test Traceability

Use comments near the test body, not only in commit messages. Examples:

```rust
// Guarantee: docs/guarantees/subscriptions/initial-snapshot-single-add.md
#[test]
fn new_subscription_emits_single_add_for_snapshot_rows() {
    // ...
}
```

```ts
// Guarantees:
// - docs/guarantees/sync/events-processed-at-least-once.md
// - docs/guarantees/views/deterministic-deltas.md
test("replay processes merged events", () => {
  // ...
});
```

A test can protect multiple guarantees, and a guarantee can be protected by
multiple tests.
