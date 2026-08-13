# A Report Is Not a Reproducible Artifact

Given a document whose root declares `volatile="true"`, when `hick test`
runs, then its woven output is excluded from drift comparison while every
other output of that document is still compared byte-for-byte, and every
`hick:expect` expectation is still enforced.

Drift checking asks one question: *do these bytes reproduce?* That question is
only meaningful when the inputs are fixed. A document measuring live data — a
corpus that grows, a dashboard, anything sampling the world — answers "no"
every single time, through no fault of anyone. A check that always fails is a
check people learn to skip, and skipping it costs the drift guarantee on every
*other* file in the document too. So the two claims are separated:
reproducibility is opt-out per output; the behavioural claim (`hick:expect`)
is not opt-out at all.

This came out of dogfooding `docs/analysis/claude-code-corpus.hick`, which
measures the Claude Code session logs on this machine. Its numbers change
between the run that writes the report and the check that verifies it, because
running the tool writes more logs. The report is volatile; `corpus_scan.py`,
assembled from fragments in the same document, is not — edit a fragment
without re-running and the check still fails.

The flag is read from `HickDocument::volatile`, parsed from the root element,
*not* through `find_tags("doc")`. The root is the document, not one of its
child nodes, so the obvious-looking `find_tags` version matched nothing and
disabled the feature silently. That is pinned by its own test.

---

Last LLM verification:
- Date: 2026-08-07
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-lang/src/lib.rs` parses `volatile` onto
  `HickDocument` beside `weave_path`; `crates/hickory-cli/src/lib.rs`
  `volatile_outputs()` collects the woven path when the root is volatile plus
  any `hick:file` marked `volatile="true"`, and `check_failures()` skips only
  those paths — the expectation loop above it is untouched.
- Test coverage: `crates/hickory-cli/tests/volatile_outputs.rs` —
  `a_volatile_report_does_not_drift_but_its_files_still_do` runs a document
  whose exec output changes every run, asserts a clean check, then tampers
  with the tangled script and asserts exactly one drift naming that script;
  `the_root_volatile_attribute_is_actually_parsed` pins the parse path and
  asserts `find_tags("doc")` is empty so the silent-miss cannot return.
  Driven for real against `docs/analysis/claude-code-corpus.hick`.
