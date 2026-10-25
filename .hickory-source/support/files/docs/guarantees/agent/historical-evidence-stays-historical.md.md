<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/historical-evidence-stays-historical.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `386ffbb5e6dd00a8bd72244a068040c931a9b2fac3183240d42d4679da435eb6`.

<book:entry path="docs/guarantees/agent/historical-evidence-stays-historical.md">
# Historical evidence stays historical

Given a new completed read receipt, when its session is inspected later, then
the audit retains the exact returned source passage, its original range and
whole-source SHA-256. The passage does not become today's file contents. A
separate current-source action opens today's file; current source is labeled
unchanged, changed or unavailable by a confined mechanical digest comparison.
Guarantee dependency snapshots use the same independent current check.

Declared citation checks establish only current path/range availability; they
do not certify relevance, entailment, omitted counterevidence or semantic truth.
Receipt chips with matching snapshots open an immutable historical reading with
the citing paragraph. Literal quotation comparisons check occurrence only in the
recorded cited range; an occurrence does not establish speaker attribution or
semantic agreement (including negation). Abandoned-branch snapshots do not count
as that turn's recorded access. Adapter locations/diffs render as declarations,
with a visible adapter-report label, rather than checked byte lineage.

Native context uses the recorded request; adapter context states its observable
scope. New write matching uses SHA-256 line fingerprints, and weak legacy or
ambiguous duplicate matches remain unresolved.

---

Last LLM verification:

- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified
- Evidence: `session.rs`, `context.rs`, `serve/audit_evidence.rs`,
  `AssumptionAudit.tsx`, `serve/acp/{read_receipt,mcp,client}.rs`.
- Witnesses: `audit_evidence::tests`, `context::tests`, `model_failure_audit`,
  `AssumptionAudit.test.tsx` and existing ACP range tests.
- Challenges: changed/missing/outside sources, old 16-bit hash collision,
  identical line duplicates, forged nested verification metadata.
- Exclusions: legacy receipts without captured passages; semantic support;
  unrecorded reads and externally edited historical records. Legacy ribbons without captured snapshots
  still navigate current files; historical bytes are in the session audit.
- Human judgment: not recorded.

- Executed checks: see [implementation audit](../audits/2026-10-07-agent-misleading-output.md).
</book:entry>

</book:doc>
