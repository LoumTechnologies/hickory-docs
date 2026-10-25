<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# experiments/token-economics/tasks/e4-effort-sweep.json

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `491f676dbad887f0bd46ca799671fe0076c141ff0516b00c27adc47676e68dfc`.

<book:entry path="experiments/token-economics/tasks/e4-effort-sweep.json">
{
  "experiment": "E4-effort-sweep",
  "description": "Effort sweep: same tasks at low/medium/high/xhigh. Deliverable: a recommended default per operation class. Expect non-monotonic cost — higher effort often means fewer turns.",
  "corpus": "../corpus",
  "arms": [
    { "name": "effort-low", "effort": "low", "context": "doc" },
    { "name": "effort-medium", "effort": "medium", "context": "doc" },
    { "name": "effort-high", "baseline": true, "effort": "high", "context": "doc" },
    { "name": "effort-xhigh", "effort": "xhigh", "context": "doc" }
  ],
  "tasks": [
    {
      "id": "cheap-lookup",
      "prompt": "Working in {corpus}: state which file assembles analysis.py and which section contributes its docstring, as your final answer. Do not edit anything.",
      "doc": "grand-tour.md"
    },
    {
      "id": "single-edit",
      "prompt": "Working in {corpus}: change the grand tour's RNG seed constant to 777 at its source location, then finish.",
      "doc": "grand-tour.md",
      "outputs": ["analysis.py"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "multi-file-refactor",
      "prompt": "Working in {corpus}: rename the grand tour's `lcg` function to `linear_congruential` everywhere (definition, calls, prose), keeping `hick check` green, then finish.",
      "doc": "grand-tour.md",
      "outputs": ["grand-tour.md", "analysis.py"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "fix-under-constraint",
      "prompt": "Working in {corpus}: make the text-tools tour's final output uppercase while keeping every existing expectation passing (update expectations only where the uppercase change forces it), then finish.",
      "doc": "text-tools-tour.md",
      "outputs": ["text-tools-tour.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check text-tools-tour.md"
    }
  ]
}
</book:entry>

</book:doc>
