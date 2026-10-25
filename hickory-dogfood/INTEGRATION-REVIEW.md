> Historical isolated-rewrite record. For the current root layout and commands,
> use [the literate workflow](../docs/developers/literate-workflow.md).

# Decide what still needs behavioral evidence

For: Hickory's product owner and maintainers reviewing the reset before treating
its requirements as verified product behavior.

Start with [the reconciliation decisions](reconciliation/REPORT.md). Every
behavior page now states whether its requirement was retained, reconciled or
retired. Its implementation review and executed checks remain separate.

## Review one stakeholder flow

For an author, begin with opening a document, saving a change, generating a file
and carrying a supported output edit back to source. Inspect refusal and recovery
alongside the successful path. Periodic recovery is not an acknowledged save.

For a reviewer, follow one proposed change from captured input to actual output.
Distinguish an authored explanation, a recorded observation, source equality and
semantic correctness. A model's summary cannot substitute for a check receipt.

For a maintainer, change one construction entry and follow all its citing
requirements. Run the named focused witnesses, then check the complete generated
inventory. A shared source file is constructed once.

## Inspect the remaining gaps

- The map lists 562 implementation files without a direct requirement reference.
  Determine whether each is shared infrastructure, an experiment, a remnant or a
  missing contract. Package membership alone is not semantic evidence.
- Earlier shortened citations and Rust facades now have navigation links. Inspect
  actual assertions before claiming that those files protect a promise.
- Later paragraphs remain in the exact-range ledger. Some are check history;
  others may add requirements. Preservation does not settle every amendment.
- Prediction, fleet, adapter and cross-process contracts retain explicit limits.
  A state-machine or frontend witness does not establish host isolation, native
  packaging, provider behavior or every filesystem race.
- All 61 freeform specs remain design history with recorded associations. Their
  earlier product assumptions do not become current merely because the source
  still reconstructs them.

## Finish a verification review

1. Read the current promise and every relevant original clause or amendment.
2. Inspect the implementation path and the test's assertions. Identify a concrete
   counterexample that would disprove the promise.
3. Run the focused witness and retain the observed result and source identity.
4. State remaining exclusions, failed challenges and unrun acceptance work.
5. Record human acceptance separately if it is given.

When intent changes, update the current contract and its focused witness together.
Keep its earlier interpretation linked as history. Dated execution records belong
in evidence; the current contract should not become an append-only activity log.

No commit was created. Broad regression and post-commit status checks remain with
the maintainer. After a later commit, use `gh run list --limit 5` and
`gh run view <run-id>`.
