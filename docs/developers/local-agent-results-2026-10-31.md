# Local agent measurements — 31 October 2026

These are focused local pilots on the Ryzen 5 3600 Linux machine, using installed Qwen3-Coder 30B Q4_K_M through Ollama, CPU execution, 8192-token allocation and six threads. The residency snapshot reports 19.56 GB with zero GPU allocation; the final document condition used a resident model. This is not a peak process-memory measurement. They do not establish Fable 5 parity or a general advantage for literate context.

The implementation adds verified-document handoff, explicit local inference controls, cutoff handling, an opt-in interleaved source reading, and an isolated document-as-state loop. The UI retains its existing review and action authority.

## Equal-information placement pilot

| Layout | Correct handle | Median wall time | Wrong-source selections |
| --- | --- | --- | --- |
| separate | 6/12 | 73.3 s | 6 |
| interleaved | 2/12 | 75.2 s | 10 |

Each pair contained identical source/context blocks and total UTF-8 bytes. The order differed. Tasks selected run, debugger and profile handles amid historical evidence and disabled controls. No live action executed.

Paired outcomes: both correct: 2, separate only: 4, interleaved only: 0, neither correct: 6. The sample contains four seed clusters; these counts do not support an independent-sample significance claim.

This encoding did **not** demonstrate an accuracy advantage for interleaving. Keep it opt-in. Raw JSON adjacency is only one encoding; richer source/evidence presentation and real UI tasks remain separate hypotheses.

## Document-as-state construction and polishing pilot

| Task | Accepted construction | Held-out behavior | Iterations | Wall time |
| --- | --- | --- | --- | --- |
| intervals | True | True | 2 | 77.2 s |
| ledger | True | True | 2 | 44.5 s |
| tokens | True | True | 2 | 60.3 s |

Each iteration revised a whole literate document using only the original task, current document and actual evaluator feedback. Hickory independently reconstructed the output; Monty evaluated public examples, then held-out examples after stopping. Every attempt remains in the journal. After the first passing program, an extra explanatory revision must preserve its executable Python AST; final source bytes are independently reconstructed again. A title and prose threshold check form, not explanation accuracy. All six saved programs from this final condition were independently reconstructed and regraded with fresh interpreter state, and the initial/final program AST fingerprints match. Explanation quality was not independently graded.

An initial polishing run was stopped after repeated Markdown-fence syntax failures; its partial journal is retained. A second polishing condition still repeatedly returned a bare code wrapper and was stopped during the ledger case. Simple changed-word transport probes passed for native chat and explicit ChatML; this does not validate every long-context behavior. The revised condition adds an explicit host-controlled polishing stage, the literal-file rule, and the actual invalid source line in feedback. A third polishing condition exposed an overly narrow presentation check: it refused valid HTML h1 headings. The final condition accepts Markdown and HTML top-level headings and grades each case in a fresh interpreter namespace. Saved candidates were independently regraded with that isolation; see the raw evidence. The earlier unpolished pilot passed all three held-out fixtures in one iteration each, but inspection found bare document wrappers and tentative code-comment reasoning. That motivated a distinct polishing condition; the earlier outputs remain in the evidence. This is sequential exploratory tuning, not a preregistered comparison.

- [intervals accepted document](../experiments/local-agent/2026-10-31/intervals/project.md).
- [ledger accepted document](../experiments/local-agent/2026-10-31/ledger/project.md).
- [tokens accepted document](../experiments/local-agent/2026-10-31/tokens/project.md).

## Action-loop protocol tuning

| Context mode | Completed | Held-out code behavior | Actual handoffs | Wall time |
| --- | --- | --- | --- | --- |
| append | False | n/a (no candidate) | 0 | 180.5 s |
| handoff | False | True | 1 | 124.4 s |

A handoff setting is not a measured handoff. Runs that fail before accepted submission cannot establish its performance effect. Earlier Qwen3.5 full-prompt and Qwen3-Coder action pilots failed to complete. The coder produced passing code but failed the literate submission protocol within its eight-turn budget. The later profile adds exact tool and submission examples and uses a twelve-turn budget; those changes are confounded, so this is protocol tuning, not proof that a prompt change alone caused an improvement. A later tuned pilot exhausted the append-mode budget; its handoff case lost the Ollama connection and is an infrastructure failure, not an accuracy result. The table reports fresh runs with the same tuned configuration. The interrupted Qwen3.5 focused pilot remains in the raw records without a completion claim.

The follow-up handoff-stage run completed: False; saved candidates: 0; held-out behavior: not available; actual handoffs: 0; wall time: 259.2 s. It adds a trusted instruction to assess remaining work and finish after an accepted handoff, rather than restart initial tool work. The original requirement and completion gate remain. This is a one-case exploratory follow-up, with model loading and focused compilation affecting timing, not a general speedup estimate.

This follow-up failed before acceptance, so the new stage instruction was not exercised by the live model and no performance improvement is established. Its transition is covered by focused unit and server integration checks. The built-in action loop remains unreliable in these local trials.

The document pilot supplies public examples initially, while the action pilot fetches them and submits separately. Differences measure feasibility and protocol overhead as well as loop representation. Temperature 0.2 is a tested setting here, not a measured global optimum.

## Evidence and limits

[Raw records and focused check logs](../experiments/local-agent/2026-10-31/records.json) retain prompts, replies, candidates, receipts, configurations and model identity. [The study guide](local-agent-study.md) describes reproduction and the larger evaluation needed before a paper claim. The raw records include unsuccessful and interrupted pilots.

Focused checks cover handoff ancestry, stale acceptance, truncation, original-task retention, forbidden host access, input mutation, aliased source association and branch replay. Source/output reconstruction and ownership audits are separate structural checks. Broad regressions remain a maintainer task; use `just checks-status` for their status.
