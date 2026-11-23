<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Build Unstable on every push to master

## Simple frontend cutover

Stable and Unstable use the same release build, pinned to the requested revision.
Publication waits for CLI and Simple frontend packages and their checksums.
The rolling Unstable release links to the newest successful CI build.
The separately signed Tauri feed belongs to the historical release procedure.

## Earlier release design and evidence

For: Maintainers publishing installers and people checking what a download contains.

Build downloads for the exact commit in each push to `master`. Publication waits for the packaging matrix and its focused checks.

## What must happen

The separate manual signing route builds the exact committed HEAD present on GitHub through the configured Apple Silicon SSH builder. It uploads the DMG and signed app archive, then switches the updater manifest only after payload uploads succeed. Existing assets stay downloadable. Publishers using that builder share its lock. The maintainer chooses when to publish the signed update feed.

## Limits of the evidence

Running the publishing workflow is the integration check for hosted packaging and GitHub credentials. Each build has its own release. The rolling tag links to the newest successfully published CI run. GitHub replaces the link asset in two requests, so a client may need to retry briefly.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/only-a-green-commit-becomes-downloadable.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Automatic Unstable builds are required on each push to `master`. The October 25 manual route remains available for signed Apple Silicon updates.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-c2750bfe4a13"></a>
## `scripts/release-unstable.py`

Related contracts: [Publish the committed Unstable build chosen by the maintainer](contract.md).

### Assemble `scripts/release-unstable.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

## Implementation review

### Implementation

- scripts/build-unstable-macos.sh
- scripts/dist-desktop.sh
- scripts/release-unstable.py

### Tests and fixtures

- scripts/tests/test_release_unstable.py

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

## Build and publish the pushed revision

Reuse the release matrix for the CLI and full desktop app. Simple frontends use
their native build commands and carry the host or web assets each launcher needs.
Use the public standard runners and Debian 13 for Qt 6.8 and KDE Frameworks 6.

A draft keeps partially uploaded assets out of the download channel. A finished
release gets a commit-specific tag and stays available for exact installs. The
serialized publisher advances the rolling pointer by run number, so a slow older
build cannot displace a newer one. Keep signed updater assets on the rolling
release for the separate Mac signing route.

<storycode:entry path=".github/workflows/unstable-release.yml">
name: Unstable Release

on:
  push:
    branches: [master]

permissions:
  contents: read

jobs:
  version:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
      tag: ${{ steps.version.outputs.tag }}
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.sha }}
          persist-credentials: false
      - id: version
        shell: bash
        run: |
          set -euo pipefail
          short=${GITHUB_SHA:0:8}
          base=$(sed -n 's/^version = "\([0-9]*\.[0-9]*\.[0-9]*\)"$/\1/p' crates/hickory-cli/Cargo.toml)
          [[ "$base" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
          echo "version=$base-unstable.$GITHUB_RUN_NUMBER.g$short" >> "$GITHUB_OUTPUT"
          echo "tag=unstable-ci-$GITHUB_RUN_NUMBER-$short" >> "$GITHUB_OUTPUT"

  build:
    needs: version
    uses: ./.github/workflows/release-build.yml
    with:
      ref: ${{ github.sha }}
      version: ${{ needs.version.outputs.version }}
      channel: unstable

  publish:
    needs: [version, build]
    runs-on: ubuntu-latest
    # The organization caps the workflow token at read. Only this job uses the
    # repository-scoped release credential already used by Stable Release.
    environment: production
    concurrency:
      group: unstable-ci-channel
      queue: max
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: '{hick-*,simple-*}'
          path: dist
          merge-multiple: true
      - name: Publish this commit and advance the Unstable channel
        shell: bash
        env:
          GH_TOKEN: ${{ secrets.RELEASE_TOKEN }}
          GH_REPO: ${{ github.repository }}
          VERSION: ${{ needs.version.outputs.version }}
          TAG: ${{ needs.version.outputs.tag }}
        run: |
          set -euo pipefail
          (cd dist && sha256sum -c -- *.sha256)
          # A draft keeps partial uploads out of downloads. A rerun can finish
          # a draft or advance the pointer after its release was already made public.
          if ! draft=$(gh release view "$TAG" --json isDraft --jq .isDraft); then
            gh release create "$TAG" --draft --prerelease --latest=false \
              --target "$GITHUB_SHA" --title "Unstable $VERSION" \
              --notes "Build of master commit $GITHUB_SHA. Choose the archive for your platform and verify its SHA-256."
            draft=true
          fi
          if [ "$draft" = true ]; then
            gh release upload "$TAG" dist/* --clobber
            gh release edit "$TAG" --draft=false --prerelease --latest=false
          fi

          # Advance the rolling pointer after all Simple and CLI assets are public.
          if ! gh release view unstable >/dev/null; then
            gh release create unstable --prerelease --latest=false \
              --target "$GITHUB_SHA" --title Unstable --notes 'Unstable builds of master.'
          fi
          previous=''
          if [ "$(gh release view unstable --json assets --jq '[.assets[] | select(.name == "ci-tag.txt")] | length')" = 1 ]; then
            gh release download unstable --pattern ci-tag.txt --output previous.txt
            previous=$(cat previous.txt)
            [[ "$previous" =~ ^unstable-ci-([1-9][0-9]*)-[a-f0-9]{8}$ ]]
            if (( BASH_REMATCH[1] > GITHUB_RUN_NUMBER )); then exit 0; fi
          fi
          if [[ "$previous" != "$TAG" ]]; then
            printf '%s\n' "$TAG" > ci-tag.txt
            gh release upload unstable ci-tag.txt --clobber
          fi
          gh release edit unstable --prerelease --latest=false --notes \
            "Latest [Unstable CI builds](https://github.com/$GH_REPO/releases/tag/$TAG)."
</storycode:entry>

</storycode:doc>
