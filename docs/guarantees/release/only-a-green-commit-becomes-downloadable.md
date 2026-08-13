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
