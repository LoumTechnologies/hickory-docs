# `hickory check` Fails When Documented Output Drifts From Real Output

Given a document containing `<hick:expect>` blocks, when `hickory check`
re-executes the document and any produced output no longer satisfies its
expectation (`exact` byte equality, or `regex-lines` full-line regex match),
then the command exits non-zero and reports the failing block with its source
span — documentation drift is a build failure, never a warning.

---

Last LLM verification:
- Date: 2026-08-05
- Reviewer: Claude (Fable 5)
- Result: not verified (implementation pending — Phase B)
- Evidence: design in `docs/specs/freeform/architecture.md` (Native
  verification section).
- Test coverage: to be added with `hickory-cli` check command; a deliberately
  drifted fixture must fail.
