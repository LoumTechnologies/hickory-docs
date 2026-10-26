<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Select the first literate CI checks

For: maintainers deciding what the experimental CI invocation must check.

The [runner contract](guarantees/verification/literate-ci/contract.md) explains
host execution and evidence limits. The publication behavior owns its own check;
the project selects it alongside the runner's safety checks. These are independent
and need no artificial dependency. Definition order is not an implied prerequisite.
To add a check, construct a definition beside its owning explanation, list its
file here, and select its ID (or depend on it). `needs` names check IDs.

This policy is opt-in locally and via the GitHub Actions **Literate CI pilot**
workflow's Run workflow button. It does not replace or add a merge gate. Start
with these focused witnesses before choosing an automatic trigger or expanding
the selected checks. After regeneration, plan with `python3 scripts/literate-ci.py`
and run with `--execute`. Review the printed receipt directory; commands inherit
the environment and run as trusted host code. Never use this runner on untrusted
code with secrets or privileges you would not give that code directly.

## Select checks without duplicating their commands

<book:copy id="ci-project-policy">
{
  "version": 1,
  "definitions": [
    "ci/checks/runner.json",
    "ci/checks/publication.json"
  ],
  "required": [
    "ci-runner-safety",
    "publication-input-safety"
  ]
}
</book:copy>
<book:file path="ci/policy.json"><book:paste select="#ci-project-policy"/></book:file>

## Let GitHub launch the same local runner

The host provisions Python and preserves the logs even after failure. It grants
read-only repository access and selects no secrets. Local and hosted execution
use the same policy and command; no broad regression suite runs in this pilot.

<book:copy id="ci-host-workflow">
name: Literate CI pilot
on:
  workflow_dispatch:
permissions:
  contents: read
jobs:
  selected-checks:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - name: Execute the project policy
        run: python3 scripts/literate-ci.py --execute
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: literate-ci-${{ github.sha }}-${{ github.run_attempt }}
          path: target/literate-ci/
          if-no-files-found: error
</book:copy>
<book:file path=".github/workflows/literate-ci.yml"><book:paste select="#ci-host-workflow"/></book:file>
</book:doc>
