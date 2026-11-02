<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Connect pure construction to native execution

For engineers implementing this part of Hickory.

The pure evaluator can replay retained output without starting a process. The
native layer adds include resolution, execution authority, cell scheduling and
filesystem publication. Keeping these concerns separate lets verification remain
read-only and prevents a source inspection from accidentally running a command.

A [capability declaration needs enforcement](guarantees/execution/declared-capabilities-are-enforced/contract.md) at the execution boundary. Merely displaying an image or capability in a document does not confine a host process.

## Assemble the native pipeline

Prepare the complete source closure before choosing a run mode. Retained captures must be validated on both initial preparation and re-preparation; include resolution must not bypass the pure engine's integrity boundary.

<book:include file="mechanisms/crates/hick-literate/src/lib.rs.md" />

## Exercise native project replay

The native witnesses compare complete projects and test scaffold patches without executing the retained generator. Literal paths and bytes, including substitutions that must remain literal, matter at this adapter boundary.

<book:include file="mechanisms/crates/hick-literate/tests/project.rs.md" />

## Review the behavioral constraints

Product reviewers can read the current outcomes, exclusions and unresolved evidence here; engineers should use them when changing the code above.

- [A document says what it needs](guarantees/execution/a-document-says-what-it-needs/contract.md).
- [Declared capabilities are enforced](guarantees/execution/declared-capabilities-are-enforced/contract.md).
- [Candidate revision checks never borrow working bytes](guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md).

Continue with [the construction order](../project.md). Remaining modules are in [the mechanism reference](mechanisms/index.md); their explicit explanation gaps remain open.
</book:doc>
