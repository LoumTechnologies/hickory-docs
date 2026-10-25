<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/assumptions-remain-auditable-after-the-work.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `6fd009c24110aa3cdc39abb9241886ad2888d236b14806f68c4617bdb6c59eae`.

<book:entry path="docs/guarantees/agent/assumptions-remain-auditable-after-the-work.md">
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

## Shared audit inspection and declaration authority (2026-10-20)

The assumption-audit disclosure now gates its own child controls. Search,
category selection and paged visible-record reading use the actual filtered UI
projection. Unrendered provider payloads require the record's nested details
before their bytes are available. Record/current-source navigation calls the
existing location controller; full session export has an explicit paged byte
read and a browser download request without a claim that a file was saved.
Verification standing is explicitly readable with the same historical/current
and execution-only scope shown to the human.

The nested annotation form exposes draft setters and its current request/result.
It still writes a user declaration through the existing human annotation endpoint.
An agent invoking that submit receives an addressable required-human-declaration
request and performs no write. A real human submit uses the same prepared fields
and returns the actual annotation reply. The built-in/external agent's existing
audit tools retain model attribution; a prepared UI note is not human testimony.
Closing the audit disclosure retires its child action handles without secretly
expanding or reading nested content.

Focused `AssumptionAudit.test.tsx` cases passed for progressive retirement,
visible-versus-raw payload reading, no model impersonation/post, actual mocked
human annotation reply and existing historical/filter behavior. Strict focused
TypeScript, inventory, file-length and diff checks passed. No real annotation,
provider/native/release/deployment witness or broad regression run is claimed.
</book:entry>

</book:doc>
