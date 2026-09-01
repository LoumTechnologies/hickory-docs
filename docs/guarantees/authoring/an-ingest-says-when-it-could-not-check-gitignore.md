# An Ingest Says When It Could Not Check `.gitignore`

Given a directory that is not a git repository, when `hick ingest --from`
runs there, then it says on stderr that `.gitignore` filtering was skipped
and every produced file was ingested unfiltered — never silently, and never
indistinguishable from a real run that checked and found nothing to skip.

## Why

`gitignored()` (`crates/hickory-cli/src/ingest_exec.rs`) already documents
the correct contract in its own doc comment: *"A directory that is not a
repository … filters nothing and says so — never silently, because 'no files
were skipped' and 'nothing could be checked' are different facts."* The
implementation returned `Ok(None)` for the not-a-repository case without
actually saying anything — `ingest_from_exec`'s `if let Some(ignored) =
&ignored` silently skipped the whole filtering branch, and the resulting
`<hick:ingested skipped="0">` looked identical to a real repository that
genuinely had nothing to skip. An author reading `skipped="0"` had no way to
tell "the filter ran and found nothing" from "the filter never ran at all",
and a scratch directory that was never `git init`'d — exactly what a
tutorial or a quick experiment is — is the common case that hits this, not
an edge case.

## What changed, and what didn't

The `<hick:ingested>` element's `skipped=` attribute stays a plain count —
this is not a document-schema change, because a count that can also mean
"unknown" would need a schema change to represent honestly, and the missing
information belongs in the command's own output rather than in the document
it produced. The fix is a stderr warning at the call site in
`ingest_from_exec`, matching how other pre-run checks
(`output_collision_warnings`, `absolute_mount_warnings`) already report to
stderr rather than the document.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/ingest_exec.rs`'s `ingest_from_exec`, the
  `match &ignored { None => log::warn!(...) }` branch right after the call to
  `gitignored`.
- Test coverage:
  `ingesting_outside_a_git_repository_warns_that_gitignore_was_never_consulted`
  in `crates/hickory-cli/tests/ingest_scaffold.rs` — drives the real shipped
  binary against a directory deliberately not `git init`'d, confirms the
  build-output file a real project would gitignore (`obj/build.log`) is
  ingested rather than dropped, confirms `skipped="0"` on the document, and
  confirms stderr explains why that count means "never checked" here.
- Caveat requiring LLM review: `ExecutorChoice::from_env()` being read fresh
  by each CLI invocation (so `HICKORY_EXECUTOR=local` must be set on `hick
  ingest` itself, not only on a preceding `hick run`) was investigated
  alongside this and found to be ordinary per-process env-var semantics, not
  a defect — the real gap was AGENTS.md never mentioning the variable at
  all, which is documentation, not this guarantee.
