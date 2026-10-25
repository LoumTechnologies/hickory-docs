<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# experiments/token-economics/tasks/e2-read-surface.json

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `fb263e34f94e8fd9385417be0be81edcd74af79581d13650da3c5acf8cbfe687`.

<book:entry path="experiments/token-economics/tasks/e2-read-surface.json">
{
  "experiment": "E2-read-surface",
  "description": "Read surface: starting from the .md document vs starting from the woven outputs. Metric: input tokens to first correct edit. Static token sizes of doc vs outputs come from `token-economics count-tokens` (count_tokens is the only instrument).",
  "corpus": "../corpus",
  "arms": [
    {
      "name": "outputs-first",
      "baseline": true,
      "context": "outputs"
    },
    {
      "name": "doc-first",
      "context": "doc"
    }
  ],
  "tasks": [
    {
      "id": "locate-seed",
      "prompt": "Working in {corpus}: find where the deterministic RNG seed of the grand tour is defined and change it to 4242 at the correct source location, then finish.",
      "doc": "grand-tour.md",
      "outputs": ["grand-tour.md", "analysis.py"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "locate-docstring",
      "prompt": "Working in {corpus}: the assembled analysis module has a module docstring. Append the line 'Reviewed: yes.' to it at the correct source location, then finish.",
      "doc": "grand-tour.md",
      "outputs": ["grand-tour.md", "analysis.py"],
      "check_cmd": "${HICKORY_BIN:-hickory} check grand-tour.md"
    },
    {
      "id": "locate-expectation",
      "prompt": "Working in {corpus}: one expectation ties the text-tools tour's output to an exact string. Make the pipeline output trailing-newline-free and adjust that expectation, then finish.",
      "doc": "text-tools-tour.md",
      "outputs": ["text-tools-tour.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check text-tools-tour.md"
    },
    {
      "id": "locate-interval-prose",
      "prompt": "Working in {corpus}: the bootstrap-ci document states its confidence level in prose. Change the stated level and the computation to 99%, then finish.",
      "doc": "bootstrap-ci.md",
      "outputs": ["bootstrap-ci.md"],
      "check_cmd": "${HICKORY_BIN:-hickory} check bootstrap-ci.md"
    }
  ]
}
</book:entry>

</book:doc>
