# `.hick` Documents Merge Through Hick, And A Clone That Cannot Is Told

Given a repository set up with `hick init`, when git merges a `.hick`
document, then it runs hick's merge driver rather than its own line merge —
and when a clone has the routing but not the driver definition, then that is
reported at project open and by `hick test`, because git otherwise falls back
silently.

Two halves, and only one of them can be committed:

- **The routing** — `*.hick merge=hick` in `.gitattributes` — is a file in the
  repository and reaches every clone.
- **The definition** — `merge.hick.driver` in `.git/config` — cannot be, because
  it is an executable command and git will not let a repository hand a clone
  one. So **every clone runs `hick init` once**, and an undefined driver makes
  git fall back to the default line merge with no warning and nothing to notice
  afterwards.

Corollaries that are part of the guarantee:

- **The check does not live in the pre-commit hook**, because `hick init`
  installs the hook: a clone that never ran it has neither the driver nor the
  thing that would report the driver missing, which is precisely the clone the
  check exists for. It lives at project open in the app and in `hick test`.
- **It is checked at open, not at commit.** A check at commit time tells you
  after the damage.
- **`hick test`'s exit code never changes because of it.** The four outcomes
  are a contract CI scripts branch on, and CI never merges — so a missing
  driver is reported and nothing more.
- **The status is read from git, not from a file.** `git check-attr merge --
  a.hick` answers what git would actually do, which `.gitattributes` alone
  cannot: attributes come from several files and from `info/attributes`.
- **A folder that is not a repository has no merges to route**, and says so
  rather than warning.
- **The app reports it once and lets it be dismissed for the window.** It is a
  fact about this clone, so a dismissal elsewhere has fixed nothing here.
- **`hick init` is idempotent** in both halves.
- **A clean line merge that produces an unreadable document is reported as a
  conflict.** Two sides can each be correct and still not compose; handing
  back a `.hick` file nothing can parse as "clean" is worse than saying so.
  This is the one thing the driver does today that git's fallback cannot.

What this does NOT claim: the driver's merge is a three-way merge of the
document text, invoked deliberately rather than fallen into. It is not yet
document-aware, and it records no correspondence. The merge tab and the
recorded correspondence are steps 2–3 of
`docs/specs/freeform/provenance-across-versions.md` and are not built. The
value delivered here is that every `.hick` merge goes through one path that is
ours, that an unreadable result is refused, and that a repository can answer
whether any of it is wired up.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/merge_driver.rs` — `ATTRIBUTES_LINE`,
    `ensure_attributes`, `ensure_driver_config` (naming this binary by its own
    path, since a merge git starts has git's `PATH`), `status` (via
    `git check-attr` and `git config --get`), and `run` (three-way merge then
    the parse check).
  - `crates/hickory-cli/src/init.rs` — both halves wired into `run_init`, and
    the report lines that say the definition is per clone.
  - `crates/hickory-cli/src/main.rs` — the hidden `merge-driver` subcommand
    honouring git's contract (result always written to `%A`, non-zero on
    conflict), and the report in `cmd_test` that never changes the exit code.
  - `crates/hickory-cli/src/serve/history.rs` — `GET /api/git/merge-driver`.
  - `apps/web/src/components/MergeDriverNotice.tsx` — the project-open notice.
  - Tests: `crates/hickory-cli/tests/replay_and_floor.rs`
    (`init_routes_hick_documents_at_the_driver_and_defines_it`,
    `a_clone_that_never_ran_init_is_told_the_driver_is_missing`,
    `hick_test_reports_a_missing_driver_without_changing_its_exit_code`,
    `a_clean_line_merge_that_does_not_parse_is_reported_as_a_conflict`,
    `a_real_merge_of_two_documents_goes_through_hick`);
    `apps/web/src/components/MergeDriverNotice.test.tsx` (four cases).
- Caveat requiring LLM review: `ensure_driver_config` writes the absolute path
  of the running binary into `.git/config`. That is right for a merge git
  starts, and it goes stale if the binary moves — a reinstall to a different
  location wants `hick init` run again. Nothing detects that today.
