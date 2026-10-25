<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# experiments/token-economics/tasks/e1-edit-surface.json

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `45a844c2afb2c58899acf207c7ccb1849dd57c15e252cc5cb2e5216eddb04abd`.

<book:entry path="experiments/token-economics/tasks/e1-edit-surface.json">
{
  "experiment": "E1-edit-surface",
  "description": "Edit surface: lineage tools vs script-writes over the grand tour + tours corpus. Primary metric: total USD per completed task; threshold ≥25% median reduction with equal-or-better check-pass rate.",
  "corpus": "../corpus",
  "arms": [
    {
      "name": "script-only",
      "baseline": true,
      "tools": false,
      "context": "doc"
    },
    {
      "name": "tools",
      "tools": true,
      "context": "doc"
    }
  ],
  "tasks": [
    {
      "id": "rename-lcg",
      "prompt": "In {corpus}/grand-tour.md, rename the linear congruential generator function `lcg` to `linear_congruential` everywhere it appears (definition, calls, and any prose that names it). Keep the document weaving cleanly.",
      "doc": "grand-tour.md",
      "outputs": ["grand-tour.md", "analysis.py"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "change-seed-constant",
      "prompt": "In {corpus}/grand-tour.md, change the deterministic RNG seed constant from its current value to 20260806 and update any woven output that shows the derived numbers by re-running the document.",
      "doc": "grand-tour.md",
      "outputs": ["grand-tour.md", "analysis.py"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "add-median-function",
      "prompt": "Add a `median(xs)` function to the woven analysis module of {corpus}/grand-tour.md (the document assembles analysis.py from prose-ordered fragments). Place it with the other statistics helpers and add a one-sentence prose introduction in the document.",
      "doc": "grand-tour.md",
      "outputs": ["analysis.py"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "fix-expectation",
      "prompt": "One expectation in {corpus}/grand-tour.md asserts an exact figure from the quarterly table. Change the underlying computation to round to 2 decimal places and update the expectation so `hick check` passes.",
      "doc": "grand-tour.md",
      "outputs": ["grand-tour.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "reword-intro-prose",
      "prompt": "Reword the opening paragraph of {corpus}/grand-tour.md (the one beginning 'This is an executable document') to be two sentences, keeping the meaning. Do not touch any code cell.",
      "doc": "grand-tour.md",
      "outputs": ["grand-tour.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "add-cell-summary-stats",
      "prompt": "Add a new executable cell near the end of {corpus}/grand-tour.md that prints the min, max, and mean of the quarterly totals, with a short prose lead-in.",
      "doc": "grand-tour.md",
      "outputs": ["grand-tour.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "rename-docstring-artifact",
      "prompt": "In {corpus}/grand-tour.md, the assembled module docstring names the source path. Update the docstring fragment so it also states the license line 'License: MIT.' as its final line.",
      "doc": "grand-tour.md",
      "outputs": ["analysis.py"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "tt-change-constant",
      "prompt": "In {corpus}/text-tools-tour.md, change the sample input text so it contains exactly five words, and update every expectation that depends on the word count.",
      "doc": "text-tools-tour.md",
      "outputs": ["text-tools-tour.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check text-tools-tour.md"
    },
    {
      "id": "tt-add-step",
      "prompt": "Add one more pipeline step to {corpus}/text-tools-tour.md that uppercases the final output, with a one-line prose explanation and a matching expectation.",
      "doc": "text-tools-tour.md",
      "outputs": ["text-tools-tour.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check text-tools-tour.md"
    },
    {
      "id": "bc-tighten-interval",
      "prompt": "In {corpus}/bootstrap-ci.md, change the confidence level from its current value to 90% and update the prose sentence that states the level plus any expectation on the interval bounds.",
      "doc": "bootstrap-ci.md",
      "outputs": ["bootstrap-ci.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check bootstrap-ci.md"
    },
    {
      "id": "bc-rename-variable",
      "prompt": "In {corpus}/bootstrap-ci.md, rename the variable holding the resample count to `n_resamples` everywhere (code and prose).",
      "doc": "bootstrap-ci.md",
      "outputs": ["bootstrap-ci.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check bootstrap-ci.md"
    },
    {
      "id": "gt-fix-typo-woven",
      "prompt": "The woven file {corpus}/grand-tour.md contains the phrase 'prose order'. Change that phrase to 'reading order' at its first occurrence by editing at the correct source location so the change survives a re-weave.",
      "doc": "grand-tour.md",
      "outputs": ["grand-tour.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    }
  ]
}
</book:entry>

</book:doc>
