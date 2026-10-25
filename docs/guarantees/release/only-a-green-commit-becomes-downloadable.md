# Manual Unstable Publication

Given a committed HEAD present on GitHub and a configured Apple Silicon SSH
builder, when a maintainer runs `just release-unstable`, then the command builds
that exact committed source and publishes its DMG and signed app archive to the
`unstable` prerelease titled **Unstable**, creating it if absent. It performs no
GitHub Actions dispatch and does not monitor or gate on regression checks.
The maintainer chooses when to publish.

The updater manifest switches only after payload upload succeeds. Existing
release assets remain downloadable. One builder lock serializes publishers
using that builder; all publishers for this channel must share it. Stable
release behavior is outside this change.

This supersedes the automatic green-CI-only Unstable requirement below at the
user's explicit request on 2026-10-25 to publish from a Linux agent without
GitHub Actions. The historical evidence is retained; it does not verify the
new behavior.

Last LLM verification:

- Date: 2026-10-25
- Reviewer: Codex
- Result: partially verified (implementation reviewed; Mac build and GitHub
  publication not executed).
- Originating requirement: user request for a Just command usable by a Linux
  AI agent to publish Apple Silicon installers and offer in-app updates.
- Evidence: `scripts/release-unstable.py` (`main`, `remote`, `publish`),
  `scripts/build-unstable-macos.sh`, `scripts/dist-desktop.sh`. The previous
  automatic Unstable workflow was removed to prevent release deletion.
- Assumptions and exclusions: the configured builder and its signing config
  are trusted; the repository is public for anonymous updater downloads; SSH
  and GitHub credentials are supplied by the maintainer; no CI gate is implied.
  Separate builders would not share the publication lock and can race.
- Witnesses and challenges: focused tests verify missing-release creation,
  upload ordering, retained assets, missing/empty signatures, pagination, SSH
  quoting, signature staging, and refusal to switch the manifest after payload upload fails.
  GitHub manifest replacement is not atomic and can temporarily remove the
  manifest on failure. A real DMG that fails to open would challenge the
  operational result, which remains unverified.
- Test coverage: `scripts/tests/test_release_unstable.py`, eight tests passed
  via `python3 -m unittest discover -s scripts/tests -p test_release_unstable.py -v`.
  Mocked API tests do not establish real GitHub service behavior.
- Dependencies: source hashes are recorded in
  `docs/guarantees/release/unstable-publication-review.json`; changes to those
  files stale this review.
- Executed checks: eight focused publishing/signature-staging tests and two update-policy tests
  passed via `just test-unstable-release`; Linux desktop library and updater
  API compilation passed. No Mac build, release upload, or broad regression suite run.
- Human judgment: not recorded. Builder selection/configuration remains open.

---

## Historical contract and review (superseded 2026-10-25)

# Only A Green Commit Becomes Downloadable

Given a push to `master`, when CI fails, then no downloadable artifact is
produced and the `unstable` release keeps pointing at the last commit that
passed. Given CI passes, then the exact commit CI verified — not the tip of
`master` at publish time — is what gets built and published.

Unstable Release is chained off CI's `workflow_run` for that reason, and
builds `github.event.workflow_run.head_sha` rather than a branch name. This
is the same shape Deploy Production already uses, and for the same reason: a
`workflow_run` subscriber that checks out a branch deploys whatever landed
while it was queueing.

Adding this subscriber must not change what Deploy Production sees.
`workflow_run` fans out to every subscribing workflow independently, and
neither release workflow modifies `ci.yml`, so the deploy trigger is
unaffected. The `unstable` git tag the release moves lives under `refs/tags/`,
which no workflow in this repository triggers on.

The two channels are named for what they are. **Release** language belongs to
the downloadable product and **promote** language to the hosted service
(`.instructions/continuous-delivery-shared.md`), so the workflows are
"Unstable Release" and "Stable Release", never "Edge" or "Prerelease" —
GitHub's `prerelease: true` flag is used underneath purely so the unstable
release does not answer `/releases/latest`.

Stable versions are immutable. A `patch`/`minor`/`major` bump that would land
on an existing `vX.Y.Z` tag fails the run rather than overwriting it; only
`bump: none` may re-attach assets to a version that already exists, which is
the rollback and rebuild case, and even then the tag is not moved.

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: partially verified — see caveats
- Evidence:
  - `.github/workflows/unstable-release.yml`: `on.workflow_run` with
    `workflows: [CI]`, `branches: [master]`; the `version` job is guarded by
    `if: github.event.workflow_run.conclusion == 'success'`, and every
    downstream job `needs` it, so a red CI run produces nothing. The commit
    built is `github.event.workflow_run.head_sha`, passed as the `ref` input
    to the reusable build workflow, which checks it out explicitly.
  - There is no deploy-to-production workflow to interact with: the product
    ships as a download, and `deploy-site.yml` publishes only the marketing
    site, on a `push` trigger of its own.
  - `ci.yml` is unmodified, so what CI reports — and therefore what the deploy
    gate keys off — is unchanged.
  - Immutability: the `version` job in `stable-release.yml` checks
    `git rev-parse refs/tags/v$version` and exits non-zero with an operator-
    facing message when the tag exists and `bump != none`.
  - Rolling channel hygiene: the unstable publish deletes the release and its
    tag (`gh release delete unstable --yes --cleanup-tag`) before recreating
    it, so assets from earlier commits — whose filenames carry the old commit
    — cannot accumulate and leave the installer several equally-valid choices.
- Caveats — what LLM review could NOT establish without a real run:
  - No release workflow has ever run. That CI's `workflow_run` event reaches
    two subscribers without either interfering is a documented GitHub
    behaviour, not something observed here.
  - `gh release delete --cleanup-tag` followed by `gh release create --target`
    within one job has not been observed against the real API.
  - The version-arithmetic shell in `stable-release.yml` was reviewed but not
    executed; there are no `v*` tags in the repository yet, so the
    no-previous-tag branch is the one that would run first.
- Test coverage: none automated. This guarantee is about workflow wiring,
  which is only observable by running it; the first Unstable Release run is
  its first real verification.
