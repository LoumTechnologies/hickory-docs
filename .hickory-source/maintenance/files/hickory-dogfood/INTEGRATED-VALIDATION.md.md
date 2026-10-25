<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/INTEGRATED-VALIDATION.md

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="hickory-dogfood/INTEGRATED-VALIDATION.md">
> Historical isolated-rewrite record. For the current root layout and commands,
> use [the literate workflow](../docs/developers/literate-workflow.md).

# Inspect the evidence for the reset

For: reviewers checking that the guarantee-organized source preserves the program
and distinguishing fresh checks from inherited claims.

Date: 2026-10-25. The starting revision, dirty state and exact file identities
are in `baseline.json`. No original repository files were changed.

## Complete-source reconstruction passed

The current Markdown closure constructs all 2,829 captured files, totaling
27,762,533 bytes. Content, file kinds and executable status match the independent
capture. A fresh Git-free directory containing only the Markdown inputs also
reconstructed that tree and passed the native complete-project verifier.

Negative controls rejected an extra file and changed bytes. The native writer
refused an output symlink pointing outside the project and left the outside
sentinel unchanged. Successful commands and observed output are recorded in
`reconciliation/generate.log` and `reconciliation/verification.log`.

The small adapter uses the integrated Hickory evaluator and native writer.
The whole-tree CLI weave path retains its repeated-lineage-parsing scale issue.
The source closure needs no original code tree, Git history or transcript cache
to supply program bytes. Bootstrap executables, Rust and installed dependencies
remain prerequisites for running the construction and tests.

## Seventeen focused runtime witnesses passed

Eight existing Rust tests exercise closure reconstruction, refusal of hidden
inputs and unordered transforms, portable inventory, exact patch application,
byte mapping, synthetic-range refusal and raw-text parsing.

Nine frontend assertions run against the integrated generated source exercise:

- recovery write/discard ordering and a failed Save keeping its view open;
- serialized generated saves and preservation of newer edits during discard;
- selected-context Unicode coordinates, omitted ranges and stale fold state;
- debugger prerequisite installation, dismissed setup preventing late launch,
  and disabled document actions during an active run.

The runner requires exactly one named passing assertion for each invocation.
A zero-test exit cannot count as success. Other tests in a selected frontend
file are skipped, not credited. `focused-results.json` retains exact commands,
output and hashes of cited files; the Markdown evidence page carries the
portable witness list.

Frontend checks use installed local dependencies, controlled callbacks and jsdom.
A temporary dependency link and external cache are cleaned up after each run.
They do not establish native window destruction, a real download or provider
execution. Cited-file hashes are not a complete transitive dependency closure.

## Requirement accounting passed

All 334 guarantee-area Markdown files have a disposition: 326 behavior pages and
eight historical audit/evidence records. All 61 freeform specs remain design
history. The behavior pages comprise 32 newly reconciled requirements, eight
rewritten earlier, 277 retained requirements and nine historical contracts.

The paragraph ledger accounts for 3,715 exact original ranges across the 326
contracts. Three focused controls rejected a missing interval, changed hash and
wrong destination. These checks validate accounting, not semantic equivalence
or completeness of discovered assumptions.

Every captured program file has one constructor. Local editorial file links
resolve; code examples and heading fragments are outside that link check.
277 repeated opening paragraphs were removed from detail pages without removing
the requirements stated in their overviews.

## Evidence that remains open

The reference map still lists 562 implementation files with no direct requirement
association. Unique shorthand citations and explicit forwarding-module links help
navigation; they do not establish test coverage. Later amendment paragraphs stay
in the ledger for clause-level review rather than being silently discarded.

No broad regressions, exhaustive semantic audit, whole-application build,
native/browser acceptance tour, paid-provider flow or release publication was
performed. Human acceptance of individual replacement requirements is unrecorded.
The user confirmed stakeholder priorities and preserving source bytes.

No commit was created. Broad regression and post-commit status checks remain
with the maintainer. After a later commit, `gh run list --limit 5` and
`gh run view <run-id>` show GitHub Actions status.
</maint:entry>
</maint:doc>
