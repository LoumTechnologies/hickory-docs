# Pre-Commit / CI Parity: The Shared Mechanism

## One rule set, two call sites

Pre-commit hooks and CI must never diverge on "does this change need the
full suite or a subset," because divergence is exactly what produces the
two failure modes `$pre-commit-ci-parity` forbids: a hook that passes
something CI then fails, and a PR check that fails on something a hook
should have caught.

Implement the changed-file-to-check mapping **once**, as a single script or
config both call sites invoke identically:

```
scripts/affected-checks.sh <base-ref>   # or the primary backend language equivalent
```

- Pre-commit hook calls it with the merge-base against the target branch
  (or `HEAD` for a working-tree check) and runs only the checks it returns.
- The CI workflow calls the same script with the PR's base ref and runs the
  same checks, the same way.
- `just check-codegen` (from `references/api-client-codegen.md`) is one of
  the checks this mapping can select — e.g. any change under `backend/src/**`
  or the OpenAPI spec path triggers it; a docs-only change doesn't.

## Path-to-check mapping, as data not logic sprinkled everywhere

Keep the mapping declarative (a small table: path glob → check name) so
adding a new check or a new path pattern is a one-line change, not a new
`if` branch buried in a workflow file:

```
docs/**            -> lint-docs
backend/src/**      -> backend-unit, check-codegen
frontend/src/**     -> frontend-unit, frontend-build
**/*.{ts,tsx,py,rs} -> (language-appropriate lint/format check)
```

A change that touches paths from multiple rows runs the union of their
checks. A change that touches nothing but `docs/**` runs only `lint-docs` —
this is the concrete mechanism behind "a docs-only edit must not trigger a
full system-test run."

A row may legitimately map to no checks at all. If nothing in the table
matches, the hook should say so and exit 0 rather than falling back to
running everything.

## Expensive checks in the map

Some checks are minutes, not seconds — a dev-environment smoke test starts
containers and both servers. Parity means the hook runs whatever the mapping
selects, so those checks *will* land in pre-commit for the paths that select
them. Don't weaken the hook to dodge the cost; that recreates exactly the
gap the first rule forbids. Instead:

- **Scope the trigger paths tightly.** Only the paths that can actually break
  the environment (`scripts/**`, the compose file, the task runner's file)
  should select it — not a broad `**/*.ts`.
- **Say so out loud.** Note the expensive rows in
  `docs/developers/developer-environment.md` so a slow commit reads as
  designed rather than broken.

Verify the mapping behaves as intended before trusting it, with a clean
index — staged leftovers from other work will silently inflate the result:

```sh
git stash -u                      # or otherwise reach a clean index
touch docs/scratch.md && git add docs/scratch.md
<checks-command> list --staged    # expect: nothing
```

## Where the shared command lives

Both call sites need a "which checks does this change require" entrypoint,
and it is *not* one of the six `just` recipes in `$dev-environment` — that
list is the dev-environment surface, not the whole task runner. Keep the
entrypoint out of the six by having the hook and the CI workflow invoke the
underlying script directly (`node scripts/dev-cli.mjs checks run --staged`).
Adding a convenience `just` recipe that wraps it is fine; what matters is
that there is exactly one implementation and both call sites reach it.

## The `main` exception

The mapping only applies to pull request checks. Any push (or merge) to
`main` runs **every** check, unconditionally — bypass the mapping entirely
on that ref rather than trying to compute "everything changed since the
last merge." This is what makes a `main` failure meaningful: if something
fails there that a PR's selective checks missed, the mapping itself is
wrong (a path pattern is too narrow, or a check is missing from a row) and
needs fixing, not a shrug.

## Flaky tests

Treat a flaky test as a defect in the test (or the code it covers) to fix
immediately — not as something to retry-until-green, skip, or quarantine
indefinitely. A retry step in CI that exists specifically to paper over
flakiness defeats the entire point of pre-commit/CI parity: a hook can't
usefully "match" a check whose pass/fail is nondeterministic.
