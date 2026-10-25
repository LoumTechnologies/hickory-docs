# Coverage retains its measured scope

Given collector evidence from automated, human, or AI-driven testing, importing
or comparing it preserves the measured executable denominator, collector data,
source identity and historical metadata. Missing data is unknown rather than
zero. Test success, assertion quality and workflow verification are separate.

Line comparisons require identical source hashes, collector/version,
instrumentation/configuration, scope, report semantics and executable locations.
Branch comparisons additionally require identifiable branch contributions.
Overlapping labels never add percentages: comparisons use sets of locations.
Aggregate imports cannot establish test attribution. Cobertura's aggregate
branch fractions remain in the original report; they cannot establish which
branch was unique to a group.

---

Last LLM verification:

- Date: 2026-10-25
- Reviewer: Codex
- Result: partially verified
- Originating requirement: GitHub #36 and #85, plus the user's request for human
  and AI manual application testing with collector-backed code-path evidence.
- Evidence: `crates/hick-coverage/src/{lib,import,compare}.rs`.
- Test coverage: `crates/hick-coverage/tests/evidence.rs`; independent four-line
  comparison witness, negative function-exit arc, duplicate LCOV records,
  repeated Cobertura method tables, Istanbul counters and malformed imports.
- Executed checks: `cargo test -p hick-coverage --test evidence`: 7 passed.
- Assumptions: executable lines are provider-defined; LCOV counts combine
  duplicate records, while Istanbul uses the maximum statement-start count.
  Source hashes must be provided by collection, not inferred from an old import.
- Exclusions: a reached line does not establish a passed workflow or assertion;
  source maps and uninstrumented code require separate availability disclosure.
- Challenges/falsifier: mixing two source revisions, treating a missing report
  as empty, or deriving branch identity from a fraction would violate this rule.
- Dependencies: the implementation files above at this change; changes to parsers,
  provenance admission or comparison logic require focused re-verification.
- Human judgment: not recorded.
