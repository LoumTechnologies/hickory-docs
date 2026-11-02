<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep generated boilerplate immutable; author changes above it

For engineers implementing this part of Hickory.

The workspace began with captured Cargo output. Its original bytes belong to the
scaffolder; our changes belong to explicit patches. Reconstruction must work from
Markdown alone, while a second ingestion or an editor action must not rewrite the
base. Refreshing a scaffold is deferred.

The base must [survive a clone as document-owned bytes](guarantees/authoring/ingest-owns-what-a-scaffolder-wrote/contract.md). A cache reference cannot satisfy that requirement. A changed capture must fail before an authored patch is applied.

## Authenticate paths and literal bytes

Validate a capture before processing nested construction or includes. Hash sorted paths and exact bytes with length prefixes; reject unsupported shape and metadata rather than authenticate only part of the capture.

<book:include file="mechanisms/crates/hick-lang/src/scaffold.rs.md" />

## Execute and ingest once

The author explicitly chooses a producer. Ingest reads that run's named output volume, filters ignored artifacts and retains UTF-8 bytes inline. A post-ingestion replay gate must reproduce each file before the document update is accepted.

<book:include file="mechanisms/crates/hickory-cli/src/ingest_exec.rs.md" />

## Exercise the real command-line boundary

The witness invokes the actual binary and copies only Markdown into a fresh directory. This tests execution, capture and replay together; the pure evaluator tests cannot establish that the command actually ran.

<book:include file="mechanisms/crates/hickory-cli/tests/ingest_scaffold.rs.md" />

## Protect source and output editor transactions

A read-only visual decoration is insufficient: typing, full-document replacement, generated-output edits and collaborative reconciliation all pass through transaction boundaries. Preserve authored patch edits while refusing changes to the capture.

<book:include file="mechanisms/apps/web/src/editor/immutableScaffolds.ts.md" />

## Assert editable changes and immutable bases

These focused editor-state tests cover hash replacement, deletion, namespace changes, multi-edit batches, Unicode and output-range movement. They are not a substitute for visual acceptance of the full application.

<book:include file="mechanisms/apps/web/src/editor/immutableScaffolds.test.ts.md" />

## Review the behavioral constraints

Product reviewers can read the current outcomes, exclusions and unresolved evidence here; engineers should use them when changing the code above.

- [A Scaffolder's Output Becomes Bytes The Document Owns, Or The Ingest Is Refused](guarantees/authoring/ingest-owns-what-a-scaffolder-wrote/contract.md).
- [An Ingested Scaffold Opens As A Tree](guarantees/authoring/an-ingested-scaffold-opens-as-a-tree/contract.md).
- [Scaffold provenance remains navigable while its base stays immutable](guarantees/lineage/ingested-bytes-are-not-yours/contract.md).

Continue with [the construction order](../project.md). Remaining modules are in [the mechanism reference](mechanisms/index.md); their explicit explanation gaps remain open.
</book:doc>
