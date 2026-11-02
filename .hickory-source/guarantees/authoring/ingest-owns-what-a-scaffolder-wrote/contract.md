<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Scaffolder's Output Becomes Bytes The Document Owns, Or The Ingest Is Refused

For: Authors editing executable documents and the files those documents produce.

Given an `<hick:exec>` cell whose command is wrapped in a `<hick:copy id=…>`
child and which mounts a volume declared with `output=`, when `hick ingest
--from '#id' <doc>.md` runs, then the cell's output volume is written into
the document as immutable literal `<hick:file>` blocks inside an `<hick:ingested>`
element recording the run — and the document is verified to weave those exact
bytes before the ingest is accepted.

An executable Markdown fence may also carry the selected cell id and output-volume mount. On ingestion, expand that fence into an explicit cell so its captured files become structured children. Preserve the command and runtime attributes, discard fence-only classification attributes, and apply the same byte comparison before accepting the document. Refuse literal Hick markup that expansion would reinterpret before executing the cell. Native cells keep their existing representation.

The focused CLI witness covers both fence markers and single-document replay; see [the readiness comparison](../../../evidence/engineer-first-readiness.md). This amendment does not renew the broader historical ingestion evidence.

The capture's `sha256` is recomputed from its sorted paths and exact UTF-8 bytes before execution, replay or project verification. Invalid/missing hashes, changed files and nonliteral construction inside the base are refused. Hickory's source and output editors, source-save paths and reverse-edit paths preserve it; authored changes use patches outside the capture. A content hash detects a mismatch, not a hostile external rewrite of both data and hash. No sidecar or earlier Git revision is needed to validate it. Refreshing an existing scaffold remains deferred.

The earlier editable-base wording in retained detail is historical and superseded by this requirement. Other existing requirements are retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/ingest-owns-what-a-scaffolder-wrote.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-exec/src/dag.rs](../../../mechanisms/crates/hick-exec/src/dag.rs.md#output-e6f480388088)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-cli/src/ingest_exec.rs](../../../mechanisms/crates/hickory-cli/src/ingest_exec.rs.md#output-4e4b92369bcd)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [crates/hickory-cli/tests/ingest_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/ingest_scaffold.rs.md#output-6682c6282b3c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
