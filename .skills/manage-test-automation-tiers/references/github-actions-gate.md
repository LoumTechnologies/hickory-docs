# The `watched` tier's human-confirm gate

This is not a hack bolted onto GitHub Actions — it's exactly what GitHub
Environment protection rules exist for: a job can declare `environment:
some-name`, and if that environment has a required reviewer configured, the
job pauses in the Actions UI until someone with reviewer permission approves
or rejects it. The environment's deployment history
(`gh api repos/:owner/:repo/deployments`) is then a free, tamper-evident audit
log of who approved/rejected and when — the same mechanism this portfolio
already uses for production-promotion approval (`$plan-deploy-shared`).

## One-time setup

1. Create a GitHub Environment named `test-tier-watch` in the target repo
   (Settings → Environments → New environment).
2. Add a required reviewer — typically the repo's maintainer(s). Do **not**
   add deployment branch restrictions; this environment gates a review
   decision, not a deploy target.
3. No secrets/variables need to live in this environment — it exists purely
   for the approval gate. If the target repo already has `staging`/
   `production` environments per `$plan-deploy-shared`, `test-tier-watch` is a
   third, separate one — don't overload an existing deploy environment with
   an unrelated approval concern.

## Workflow shape

Add this as a job sequence in the target repo's existing CI workflow (or a
dedicated `test-tiers.yml` if the existing CI is already large and this
would blow past `$continuous-integration`'s file-size/complexity limits):

```yaml
jobs:
  test-tier-check:
    runs-on: ubuntu-latest
    outputs:
      watched_tests: ${{ steps.check.outputs.watched_tests }}   # JSON array of test ids still needing a human this cycle
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }   # blast-radius diffing needs history, not a shallow clone
      - name: Compute demotions and run automated-tier tests
        id: check
        run: just test-tier-check

  run-watched:
    needs: test-tier-check
    if: fromJson(needs.test-tier-check.outputs.watched_tests)[0] != null
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run each watched-tier test's command, upload artifacts
        run: just test-tier-run-watched   # runs test.run.watched for each id in watched_tests, saves logs/traces
      - uses: actions/upload-artifact@v4
        with: { name: watched-tier-results, path: .test-tiers/artifacts/ }

  await-confirmation:
    needs: run-watched
    if: needs.run-watched.result == 'success'
    environment: test-tier-watch
    runs-on: ubuntu-latest
    steps:
      - run: echo "Approved by ${{ github.actor }} — reviewer looked at run-watched's uploaded artifacts before approving"

  record-watched-result:
    needs: await-confirmation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Record the confirmed pass for each watched test this cycle
        run: |
          for id in $(echo '${{ needs.test-tier-check.outputs.watched_tests }}' | jq -r '.[]'); do
            just test-tier-confirm "$id" pass "Confirmed via CI approval by ${{ github.actor }} on run ${{ github.run_id }}"
          done
      - run: git push   # the confirm recipe commits the test-tiers.yaml update; push it
```

`eyeball`-tier tests are deliberately **not** in this workflow at all — there
is nothing to run unattended. `test-tier-check`'s summary output (and `just
test-tier-status`) is where they surface as "still needs a human to run this
by hand," which a maintainer acts on outside CI (running `just test-tier-
confirm <id> ...` locally after doing the manual walkthrough).

## Why the reviewer must actually look at something

`await-confirmation` approving blindly (a reviewer who clicks "Approve"
without reading `run-watched`'s uploaded artifacts) defeats the entire point
of the `watched` tier — it would just be `automated` with extra latency. Say
so explicitly in the environment's description in GitHub's UI, and make sure
`run-watched` uploads whatever a human needs to actually judge the result
(screenshots, Playwright traces, walkthrough transcript) rather than just a
pass/fail exit code.

## Failure path

If `run-watched` itself fails (the test genuinely broke), `await-confirmation`
should not run at all (`needs.run-watched.result == 'success'` above) — a
broken test is just a normal CI failure, and per `SKILL.md`'s guardrails, it
must not reach the human-confirm gate framed as something to bless. The next
`test-tier-check` cycle will see the failed result and demote the test per
`tier-ladder.md`'s algorithm.
