<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/privatePredictionCoach.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8905b58855c2"></a>
## `apps/web/src/components/privatePredictionCoach.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/privatePredictionCoach.css">
.private-prediction-coach { box-sizing: border-box; max-width: 1100px; margin: auto; padding: clamp(12px, 3vw, 24px); overflow-wrap: anywhere; }
.private-prediction-coach section { margin-block: 1.5rem; min-width: 0; }
.private-prediction-fields { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1rem; margin-block: 1rem; }
.private-prediction-fields label { display: flex; flex-direction: column; gap: .35rem; min-width: 0; }
.private-prediction-request { grid-column: 1 / -1; }
.private-prediction-fields input, .private-prediction-fields textarea { box-sizing: border-box; width: 100%; min-width: 0; font: inherit; }
.private-prediction-fields textarea { min-height: 8rem; resize: vertical; }
.private-prediction-controls { display: flex; flex-wrap: wrap; gap: .5rem; }
@media (max-width: 540px) { .private-prediction-fields { grid-template-columns: minmax(0, 1fr); } }
/* Long captured lines scroll inside CodeMirror, not the entire private page. */
.private-prediction-coach .comparison-code .cm-editor { flex: 1; min-width: 0; max-width: 100%; }
</storycode:entry>

</storycode:doc>
