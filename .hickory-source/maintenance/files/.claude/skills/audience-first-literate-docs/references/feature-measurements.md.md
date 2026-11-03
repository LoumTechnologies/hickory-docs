<guidance:doc xmlns:guidance="http://www.hickorydocs.com/1.0">
# Maintain .claude/skills/audience-first-literate-docs/references/feature-measurements.md

For maintainers updating the repository’s agent guidance.

This literal entry preserves the skill refresh committed in `efbe277f`. Keeping the
installed guidance in the declared source makes regeneration reproduce the same
instructions. Embedded Markdown and Hickory examples remain literal payload.

<guidance:entry path=".claude/skills/audience-first-literate-docs/references/feature-measurements.md">
# Gather evidence while using Hickory

Use ordinary Hickory implementation work as the task population. In the agent
pane, select Codex ACP. The human checks **Measure this request with and without
Hickory features**, selects the interventions, and presses Send with the actual
request. The checkbox is off by default; only human interaction authorizes the
extra runs. Acceptance of Send clears the opt-in for the next request. A selected intervention adds two ACP runs, each bounded by the shown
time limit, after the normal implementation. Do not silently opt in or replace
this workflow with a standalone query runner.

Hickory freezes named editor overlays and regular project files before the
primary turn. Comparison arms start fresh in separate working copies through
Hickory's ordinary ACP client and tool bridge. They receive the same query and
selected settings, changing one feature at a time. They do not replay earlier
conversation history. For context-dependent requests, make the task sufficiently
explicit to stand alone, or record that limitation rather than rewriting one arm.
The copies are not an OS sandbox; external adapter internals remain opaque.

The initial interventions remove literate source indexes or lineage output tools.
Index availability includes the captured project construction pages and current
source-backed views. It does not prove the agent consulted them, and removing
maintained literate source may also change which authoring workflows are possible.
State the exact intervention when interpreting results. Unsupported features need
an implemented intervention, representative tasks and independent witnesses;
the contract inventory is a coverage denominator, not proof of every feature.

Open **Measurements and dataset** to inspect each retained answer, working copy
and ACP transcript. Run focused checks or inspect the actual requested behavior
in each copy. Attach concrete correctness evidence and have the human mark the
result correct or incorrect. An agent's completion message and self-reported
success are not independent witnesses. Keep tests and human judgment separate.

Export the dataset from that panel. Snapshots, settings, source hashes, arm order,
raw results, failures and append-only reviews live in personal project storage.
Do not discard a failed or slower arm, conflate source revisions/models, or treat
missing token counters as zero. ACP context occupancy is not billed input-token
usage. Codex ACP 2.0.1 exposes last-call counts; Hickory qualifies prompt totals
against the audited 2.5.0 adapter and leaves other versions unqualified. Those
totals still do not establish hidden child-thread or provider-wide spend. Use
**Update Codex adapter for token totals** in the measurement panel when needed;
this is an explicit download through Hickory’s existing installer. Query savings cover the prompt call and its tool work; setup and end-to-end
arm timings are recorded separately. They exclude snapshot preparation cost and
may not capture the human effort to author and maintain indexes.

Only completed, model-matched arms accepted as correct contribute savings.
Positive values mean disabled minus enabled; negative values mean regression.
One request supplies a paired observation. Accumulate different tasks, repeated
runs and representative users before making a general performance claim. Report
correctness, failures, sample size, selection effects and unknown metrics beside
any time/token claim. Byte-equivalent views establish reconstruction, not an
accurate explanation or a performance benefit.

When changing the measurement implementation, begin at the project's maintained
entry and owning contract. In Hickory Docs that contract is
`.hickory-source/guarantees/agent/feature-experiments/contract.md`.
</guidance:entry>
</guidance:doc>
