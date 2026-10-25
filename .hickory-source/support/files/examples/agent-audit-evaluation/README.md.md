<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/agent-audit-evaluation/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `5b1199a1c3f0f26560db7ce41ba1acc6727d5c8e44e73ee59c596bcde1a0fa52`.

<book:entry path="examples/agent-audit-evaluation/README.md">
# Evaluate discovery of misleading output

This is an evaluation kit, not evidence that Hickory improves human detection.
Use a disposable copy of this folder. Do not execute seeded hostile instructions.
An observer prepares each case from `observer-cases.json`; participants receive
only the source and the prepared conversation, not the expected findings.

For each case, prepare two equivalent sessions: one with a misleading answer
and one correct control. Record actual inputs, declarations, branches and tool
results through the ordinary tools; do not manufacture a passed harness result.
Use a deterministic adapter/scripted client for behavior that could damage files.
The observer may construct **clearly staged, never executed** conversations for
semantic cases. Preserve that distinction in the session bytes and interface.

Ask a participant to decide whether to accept the proposed edit or rely on the
answer for its stated task. Do not first tell them which failure to look for.
Counterbalance case order and include correct cases to measure false alarms.
At the decision point ask what they believe each receipt establishes. Afterwards
ask them to debug an older decision, including a correction and abandoned branch.

Record one JSON object per trial in a file outside the participant workspace:

```json
{"participant":"anonymous-01","case":"conditional-deadline","control":false,
 "detected_before_reliance":true,"decision":"reject","seconds_to_decision":48,
 "evidence_opened":["historical receipt"],"receipt_understanding":"source availability only",
 "false_alarm":false,"retrospective_explanation_correct":true}
```

Report detection before reliance, time to detection, false alarms, incorrect
interpretation of receipts as proof, and successful retrospective reconstruction.
Keep missed cases and exclusions visible. Report participant count and task
experience; do not generalize from component tests, one participant, or a single
model. A separate live-model study measures whether agents disclose assumptions
and obey boundaries; model runs cannot substitute for this human study.
</book:entry>

</book:doc>
