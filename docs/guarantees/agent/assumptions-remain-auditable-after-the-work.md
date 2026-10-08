# Assumptions remain auditable after the work

Given a retained agent session, when an agent records assumptions, decisions or
guarantee reviews, then each entry retains its statement, reason, evidence
claimed, affected work, alternatives and falsifier. Guarantee reviews require
the originating requirement and a falsifier; exclusions, witnesses, challenges
and dependency snapshots are separate fields.

When an earlier entry is corrected, the correction names `supersedes` and both
entries remain. The audit view and `audit_assumptions` include all recorded
branches, not only the current branch. The view is searchable, exposes action
requests/results and review decisions, and lets a user append a declaration or
correction and download the full session. An unknown superseded id is refused.

Every agent entry is **model-declared**. An interface annotation is
**user-declared**. Neither establishes truth, complete assumption discovery,
authorship, or the model's hidden reasoning. Empty or missing entries do not
certify absence of consequential assumptions. Work records are available to
investigate omissions. The model is instructed to record consequential
assumptions; discovering and declaring every one is not enforceable.

Sessions are local `.md` records and ordinarily gitignored. Retain or export
them to investigate later; the guarantee makes no backup or immutability claim.

---

Last LLM verification:

- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified
- Requirement: preserve consequential assumptions for retrospective debugging,
  as requested in this conversation; preserve original entries through corrections.
- Evidence: `hickory-agent/src/audit.rs`, `serve/sessions.rs`,
  `components/AssumptionAudit.tsx`, ACP catalogue and native protocol integration.
- Witnesses: `audit::tests`, `AssumptionAudit.test.tsx`, `model_failure_audit`.
- Challenges: abandoned branch, unknown correction, spoofed human status,
  undisclosed assumptions, editable/deleted sessions.
- Executed checks: see the implementation audit's validation record.
- Human judgment: not recorded; actual omission-detection efficacy is unevaluated.
- Exclusions: opaque adapter memory, hidden reasoning, all-assumptions completeness,
  authenticated authorship and durability after session deletion.
