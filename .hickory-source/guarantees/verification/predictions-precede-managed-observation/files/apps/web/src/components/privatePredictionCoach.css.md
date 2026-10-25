<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/privatePredictionCoach.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `ccc41cb0c221e8aa97de05f1d10e1852bdce79da6e373cb777f28df5ed53af79`.

<book:entry path="apps/web/src/components/privatePredictionCoach.css">
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
</book:entry>

</book:doc>
