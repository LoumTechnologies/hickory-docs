<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .instructions/pre-commit-ci-parity.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `f01d4b4dae6caf3661573ec09b6479627916243d4678730b71e94a817cc829f1`.

<book:entry path=".instructions/pre-commit-ci-parity.md">
---
skills:
  - implement-dev-environment
---

# Local Checks / CI Parity And Selective Checks

**2026-10-17 amendment:** Repository checks run through blocking `just checks`.
`post-commit` launches it detached; no pre-commit or pre-push check may block Git.
A dirty working tree has an unassociated result; clean runs use a detached
worktree and record the exact commit. The older gate wording below applies to
check coverage, not permission to block a Git operation. Local execution does
not require GitHub Actions. Existing release workflows are separate.


- **Pre-commit hooks must be at least as strict as CI.** It must never be
  possible to pass hooks, push, and then have CI fail on something a hook
  could have caught — that gap is a bug in the hooks, fix it there.
- **Both pre-commit hooks and CI must be scoped to what actually changed.**
  A docs-only edit must not trigger a full system-test run; a
  backend-only change must not trigger a frontend-only suite. Keep the
  change-detection rules for hooks and CI identical so they can't drift
  apart and reintroduce the gap the first rule forbids.
- **A push (or merge) to `main` always runs the full check suite**,
  unconditionally — selective scoping only ever applies to pull requests.
- **A `main` failure that a PR's selective checks missed is a signal the
  change-detection rules are wrong** and must be fixed, not a one-off to
  shrug at.
- **A PR CI failure that pre-commit should have caught is the same kind of
  signal**, fixed the same way — tighten the hook, don't just note it.
- **Flaky tests are treated as defects and fixed immediately**, not
  retried, quarantined-and-forgotten, or tolerated as background noise.
</book:entry>

</book:doc>
