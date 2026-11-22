<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Build Unstable on every push to master

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

<a id="f-c2750bfe4a13-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c2750bfe4a13-expression_statement">
#!/usr/bin/env python3
"""Publish the committed checkout using an Apple Silicon SSH builder, without Actions."""
</storycode:copy>
<a id="f-c2750bfe4a13-datetime-as-dt"></a>
### `datetime as dt`

<storycode:copy id="f-c2750bfe4a13-datetime-as-dt">
import datetime as dt
</storycode:copy>
<a id="f-c2750bfe4a13-json"></a>
### `json`

<storycode:copy id="f-c2750bfe4a13-json">
import json
</storycode:copy>
<a id="f-c2750bfe4a13-os"></a>
### `os`

<storycode:copy id="f-c2750bfe4a13-os">
import os
</storycode:copy>
<a id="f-c2750bfe4a13-re"></a>
### `re`

<storycode:copy id="f-c2750bfe4a13-re">
import re
</storycode:copy>
<a id="f-c2750bfe4a13-Path"></a>
### `Path`

<storycode:copy id="f-c2750bfe4a13-Path">
from pathlib import Path
</storycode:copy>
<a id="f-c2750bfe4a13-shlex"></a>
### `shlex`

<storycode:copy id="f-c2750bfe4a13-shlex">
import shlex
</storycode:copy>
<a id="f-c2750bfe4a13-subprocess-as-sp"></a>
### `subprocess as sp`

<storycode:copy id="f-c2750bfe4a13-subprocess-as-sp">
import subprocess as sp
</storycode:copy>
<a id="f-c2750bfe4a13-tempfile"></a>
### `tempfile`

<storycode:copy id="f-c2750bfe4a13-tempfile">
import tempfile

</storycode:copy>
<a id="f-c2750bfe4a13-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c2750bfe4a13-expression_statement-2">
REPO = "LoumTechnologies/hickory-docs"
</storycode:copy>
<a id="f-c2750bfe4a13-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c2750bfe4a13-expression_statement-3">
ROOT = Path(__file__).resolve().parents[1]


</storycode:copy>
<a id="f-c2750bfe4a13-run"></a>
### `run`

<storycode:copy id="f-c2750bfe4a13-run">
def run(*args, **kwargs):
    return sp.run(args, check=True, **kwargs)


</storycode:copy>
<a id="f-c2750bfe4a13-output"></a>
### `output`

<storycode:copy id="f-c2750bfe4a13-output">
def output(*args):
    return sp.check_output(args, text=True).strip()


</storycode:copy>
<a id="f-c2750bfe4a13-remote"></a>
### `remote`

<storycode:copy id="f-c2750bfe4a13-remote">
def remote(host, command, **kwargs):
    # SSH joins arguments into shell text; quote both shell layers explicitly.
    return run("ssh", "-oBatchMode=yes", "--", host,
               "bash -lc " + shlex.quote(command), **kwargs)


</storycode:copy>
<a id="f-c2750bfe4a13-manifest"></a>
### `manifest`

<storycode:copy id="f-c2750bfe4a13-manifest">
def manifest(version, signature, archive, sha):
    if not signature.strip():
        raise ValueError("The updater signature is empty")
    return {
        "version": version,
        "notes": f"Unstable build from {sha}. Save your work and reopen the app after installation.",
        "pub_date": dt.datetime.now(dt.timezone.utc).isoformat(),
        "platforms": {"darwin-aarch64": {
            "signature": signature.strip(),
            "url": f"https://github.com/{REPO}/releases/download/unstable/{archive}",
        }},
    }


</storycode:copy>
<a id="f-c2750bfe4a13-publish"></a>
### `publish`

<storycode:copy id="f-c2750bfe4a13-publish">
def publish(directory, version, sha):
    """Upload immutable payloads first, then switch the updater manifest last."""
    prefix = f"hickory-docs-{version}-aarch64-apple-darwin"
    archive = directory / (prefix + ".app.tar.gz")
    signature = directory / (archive.name + ".sig")
    dmg = directory / (prefix + ".dmg")
    for path in (archive, signature, dmg):
        if not path.is_file() or path.stat().st_size == 0:
            raise ValueError(f"Missing build artifact: {path.name}")
    latest = directory / "unstable.json"
    latest.write_text(json.dumps(manifest(version, signature.read_text(), archive.name, sha), indent=2) + "\n")
    if not release_exists():
        run("gh", "release", "create", "unstable", "--repo", REPO, "--title", "Unstable",
            "--prerelease", "--target", sha, "--notes", "Manually published Apple Silicon desktop builds.")
    # Existing archives stay available to clients that already fetched an older manifest.
    payloads = sorted(str(p) for p in directory.iterdir() if p.is_file() and p != latest)
    run("gh", "release", "upload", "unstable", "--repo", REPO, *payloads)
    run("gh", "release", "upload", "unstable", "--repo", REPO, str(latest), "--clobber")
    # Only this designated rolling tag moves; stable version tags are untouched.
    run("gh", "api", f"repos/{REPO}/git/refs/tags/unstable", "--method", "PATCH",
        "-f", f"sha={sha}", "-F", "force=true")
    notes = directory / "release-notes.md"
    notes.write_text(f"Apple Silicon desktop {version}, built from `{sha}`.\n\n"
                     f"For other platforms and CLI builds, see https://github.com/{REPO}/releases.\n")
    run("gh", "release", "edit", "unstable", "--repo", REPO, "--title", "Unstable",
        "--prerelease", "--notes-file", str(notes))


</storycode:copy>
<a id="f-c2750bfe4a13-release_exists"></a>
### `release_exists`

<storycode:copy id="f-c2750bfe4a13-release_exists">
def release_exists():
    # A network/auth error must not be treated as a missing release.
    response = run("gh", "api", "--paginate", "--slurp", f"repos/{REPO}/releases", stdout=sp.PIPE, text=True)
    return any(release["tag_name"] == "unstable" for page in json.loads(response.stdout) for release in page)


</storycode:copy>
<a id="f-c2750bfe4a13-main"></a>
### `main`

<storycode:copy id="f-c2750bfe4a13-main">
def main():
    os.chdir(ROOT)
    host = os.environ.get("HICKORY_MAC_BUILDER", "")
    if not host or host.startswith("-"):
        raise ValueError("Set HICKORY_MAC_BUILDER to an Apple Silicon Mac's SSH alias (see apps/desktop/RELEASING.md)")
    if output("git", "status", "--porcelain", "--untracked-files=no"):
        raise ValueError("Commit tracked changes before publishing; only HEAD is built")
    sha = output("git", "rev-parse", "HEAD")
    # Publishing only pushed commits makes every installed build reproducible.
    run("gh", "api", f"repos/{REPO}/commits/{sha}", stdout=sp.DEVNULL)
    base = json.loads((ROOT / "apps/desktop/src-tauri/tauri.conf.json").read_text())["version"].split("-")[0]
    # A lock on the shared builder covers build AND publication, including competing Linux agents.
    acquire = '''set -e
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || { echo 'Builder must be an Apple Silicon Mac' >&2; exit 1; }
cache="$HOME/.cache/hickory-docs-unstable"
mkdir -p "$cache"
mkdir "$cache/publish.lock" || { echo 'Another publisher holds ~/.cache/hickory-docs-unstable/publish.lock' >&2; exit 1; }
mktemp -d "$cache/build.XXXXXX"
'''
    result = remote(host, acquire, stdout=sp.PIPE, text=True)
    work = result.stdout.strip()
    if not re.fullmatch(r"/[^\n]+/\.cache/hickory-docs-unstable/build\.[A-Za-z0-9]+", work):
        raise ValueError("Builder returned an invalid build path")
    quoted = shlex.quote(work)
    try:
        stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%d%H%M%S%f")
        version = f"{base}-unstable.{stamp}.g{sha[:8]}"
        with tempfile.TemporaryDirectory(prefix="hickory-unstable-") as temp:
            temp = Path(temp)
            source = temp / "source.tgz"
            with source.open("wb") as stream:
                run("git", "archive", "--format=tar.gz", sha, stdout=stream)
            with source.open("rb") as stream:
                remote(host, f"tar -xzf - -C {quoted}", stdin=stream)
            remote(host, f"cd {quoted} && bash scripts/build-unstable-macos.sh {shlex.quote(version)}")
            artifacts = temp / "artifacts"
            artifacts.mkdir()
            # Fetch fixed names through tar; the builder needs no GitHub credentials.
            with (temp / "artifacts.tar").open("wb") as stream:
                remote(host, f"tar -cf - -C {quoted}/dist/desktop .", stdout=stream)
            import tarfile
            with tarfile.open(temp / "artifacts.tar") as tar:
                tar.extractall(artifacts, filter="data")
            publish(artifacts, version, sha)
    finally:
        remote(host, f"rm -rf -- {quoted}; rmdir \"$HOME/.cache/hickory-docs-unstable/publish.lock\"")
    print(f"Published {version}: https://github.com/{REPO}/releases/tag/unstable")


</storycode:copy>
<a id="f-c2750bfe4a13-if_statement"></a>
### `if_statement`

<storycode:copy id="f-c2750bfe4a13-if_statement">
if __name__ == "__main__":
    try:
        main()
    except (ValueError, sp.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
</storycode:copy>

### Assemble `scripts/release-unstable.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/release-unstable.py" executable="true"><storycode:paste select="#f-c2750bfe4a13-expression_statement"/><storycode:paste select="#f-c2750bfe4a13-datetime-as-dt"/><storycode:paste select="#f-c2750bfe4a13-json"/><storycode:paste select="#f-c2750bfe4a13-os"/><storycode:paste select="#f-c2750bfe4a13-re"/><storycode:paste select="#f-c2750bfe4a13-Path"/><storycode:paste select="#f-c2750bfe4a13-shlex"/><storycode:paste select="#f-c2750bfe4a13-subprocess-as-sp"/><storycode:paste select="#f-c2750bfe4a13-tempfile"/><storycode:paste select="#f-c2750bfe4a13-expression_statement-2"/><storycode:paste select="#f-c2750bfe4a13-expression_statement-3"/><storycode:paste select="#f-c2750bfe4a13-run"/><storycode:paste select="#f-c2750bfe4a13-output"/><storycode:paste select="#f-c2750bfe4a13-remote"/><storycode:paste select="#f-c2750bfe4a13-manifest"/><storycode:paste select="#f-c2750bfe4a13-publish"/><storycode:paste select="#f-c2750bfe4a13-release_exists"/><storycode:paste select="#f-c2750bfe4a13-main"/><storycode:paste select="#f-c2750bfe4a13-if_statement"/></storycode:file>

## Implementation review

### Implementation

- [scripts/build-unstable-macos.sh](../../../mechanisms/scripts/build-unstable-macos.sh.md#output-f2dcad3b78b2)
- [scripts/dist-desktop.sh](../../../mechanisms/scripts/dist-desktop.sh.md#output-928dfd6fa282)
- [scripts/release-unstable.py](contract.md#output-c2750bfe4a13)

### Tests and fixtures

- [scripts/tests/test_release_unstable.py](../../../mechanisms/scripts/tests/test_release_unstable.py.md#output-66686df94df4)

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

  simple:
    needs: version
    name: Simple ${{ matrix.target }}
    runs-on: ${{ matrix.runner }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - { platform: browser, target: browser, runner: ubuntu-24.04 }
          - { platform: macos, target: macos-arm64, runner: macos-15 }
          - { platform: macos, target: macos-x64, runner: macos-15-intel }
          - { platform: ios, target: ios-simulator-arm64, runner: macos-15 }
          - { platform: android, target: android, runner: ubuntu-24.04 }
          - { platform: windows, target: windows-x64, runner: windows-2025 }
          - { platform: windows, target: windows-arm64, runner: windows-11-arm }
    env:
      PLATFORM: ${{ matrix.platform }}
      TARGET: ${{ matrix.target }}
      VERSION: ${{ needs.version.outputs.version }}
      CARGO_PROFILE_DEV_DEBUG: '0'
      CARGO_INCREMENTAL: '0'
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.sha }}
          persist-credentials: false
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: npm
          cache-dependency-path: apps/web/package-lock.json
      - uses: extractions/setup-just@v4
      - uses: dtolnay/rust-toolchain@1.96.1
        if: matrix.platform == 'windows' || matrix.platform == 'android'
      - name: Select the native Windows ARM64 Rust toolchain
        if: matrix.target == 'windows-arm64'
        shell: bash
        run: |
          rustup toolchain install 1.96.1-aarch64-pc-windows-msvc --profile minimal
          echo "RUSTUP_TOOLCHAIN=1.96.1-aarch64-pc-windows-msvc" >> "$GITHUB_ENV"
      - name: Install NASM for the x64 workspace host
        if: matrix.target == 'windows-x64'
        shell: bash
        run: |
          choco install nasm -y --no-progress
          echo "C:\Program Files\NASM" >> "$GITHUB_PATH"
      - uses: actions/setup-dotnet@v4
        if: matrix.platform == 'windows'
        with:
          dotnet-version: '10.0.x'
      - uses: actions/setup-java@v4
        if: matrix.platform == 'android'
        with:
          distribution: temurin
          java-version: '17'
      - uses: gradle/actions/setup-gradle@v4
        if: matrix.platform == 'android'
        with:
          gradle-version: '9.3.1'
      - name: Use the hosted Android SDK
        if: matrix.platform == 'android'
        shell: bash
        run: |
          if [[ ! -f "$ANDROID_HOME/platforms/android-37.0/android.jar" ||
                ! -x "$ANDROID_HOME/build-tools/36.0.0/aapt2" ]]; then
            "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" \
              'platforms;android-37.0' 'build-tools;36.0.0'
          fi
      - if: runner.os == 'macOS'
        run: brew install xcodegen
      - run: npm --prefix apps/web ci
      - name: Build and package the Simple frontend
        shell: bash
        run: |
          set -euo pipefail
          mkdir -p dist/simple
          asset="dist/simple/hickory-simple-$VERSION-$TARGET"
          case "$PLATFORM" in
            browser)
              just simple-build browser
              test -f apps/web/dist-simple/simple.html
              tar -czf "$asset.tar.gz" -C apps/web/dist-simple .
              ;;
            macos)
              python3 scripts/prepare-simple-resources.py
              xcodegen generate --spec apps/apple-simple/project.yml
              xcodebuild -project apps/apple-simple/HickorySimple.xcodeproj \
                -target HickorySimpleMac -configuration Release ARCHS="$(uname -m)" \
                SYMROOT="$PWD/target/simple-release" CODE_SIGNING_ALLOWED=NO build
              app=target/simple-release/Release/HickorySimpleMac.app
              test -x "$app/Contents/MacOS/HickorySimpleMac"
              codesign --force --sign - "$app"
              ditto -c -k --keepParent "$app" "$asset.zip"
              ;;
            ios)
              python3 scripts/prepare-simple-resources.py
              xcodegen generate --spec apps/apple-simple/project.yml
              xcodebuild -project apps/apple-simple/HickorySimple.xcodeproj \
                -scheme HickorySimpleMobile -configuration Release -sdk iphonesimulator \
                -destination 'generic/platform=iOS Simulator' -derivedDataPath target/simple-release-ios \
                ARCHS=arm64 CODE_SIGNING_ALLOWED=NO build
              app=target/simple-release-ios/Build/Products/Release-iphonesimulator/HickorySimpleMobile.app
              test -x "$app/HickorySimpleMobile"
              ditto -c -k --keepParent "$app" "$asset.zip"
              ;;
            android)
              just simple-build android
              cp apps/android-simple/app/build/outputs/apk/debug/app-debug.apk "$asset.apk"
              test -s "$asset.apk"
              ;;
            windows)
              just simple-build windows
              test -s target/simple-windows/app/HickorySimple.exe
              test -s target/simple-windows/app/hickory-simple-host.exe
              test -f target/simple-windows/app/web/simple.html
              tar -czf "$asset.tar.gz" -C target/simple-windows/app .
              ;;
          esac
          cd dist/simple
          for file in hickory-simple-*; do
            test -s "$file"
            if command -v sha256sum >/dev/null; then
              sha256sum "$file" > "$file.sha256"
            else
              shasum -a 256 "$file" > "$file.sha256"
            fi
          done
      - uses: actions/upload-artifact@v4
        with:
          name: simple-${{ matrix.target }}
          path: dist/simple/*
          if-no-files-found: error
          retention-days: 7

  kde:
    needs: version
    name: Simple KDE ${{ matrix.arch }}
    runs-on: ${{ matrix.runner }}
    container: debian:13
    strategy:
      fail-fast: false
      matrix:
        include:
          - { arch: x64, runner: ubuntu-24.04 }
          - { arch: arm64, runner: ubuntu-24.04-arm }
    env:
      TARGET: linux-${{ matrix.arch }}
      VERSION: ${{ needs.version.outputs.version }}
      CARGO_PROFILE_DEV_DEBUG: '0'
      CARGO_INCREMENTAL: '0'
    steps:
      - name: Install the Debian KDE toolchain
        run: |
          apt-get update
          apt-get install -y --no-install-recommends ca-certificates git curl xz-utils \
            build-essential cmake pkg-config qt6-base-dev qt6-webengine-dev \
            libkf6coreaddons-dev libkf6xmlgui-dev
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.sha }}
          persist-credentials: false
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: npm
          cache-dependency-path: apps/web/package-lock.json
      - uses: dtolnay/rust-toolchain@1.96.1
      - uses: extractions/setup-just@v4
      - name: Package the relocatable KDE installation tree
        shell: bash
        run: |
          set -euo pipefail
          just simple-build linux
          test -x target/simple-linux/app/bin/hickory-simple
          mkdir -p dist/simple
          file="hickory-simple-$VERSION-$TARGET.tar.gz"
          tar -czf "dist/simple/$file" -C target/simple-linux/app .
          cd dist/simple
          sha256sum "$file" > "$file.sha256"
      - uses: actions/upload-artifact@v4
        with:
          name: simple-linux-${{ matrix.arch }}
          path: dist/simple/*
          if-no-files-found: error
          retention-days: 7

  publish:
    needs: [version, build, simple, kde]
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
          pattern: '{hick-*,hickory-docs-desktop-*,simple-*}'
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

          # Keep the manually signed macOS feed and its payloads in place. Only
          # publish the pointer to this CI release after all its assets are public.
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
            "Latest [Unstable CI builds](https://github.com/$GH_REPO/releases/tag/$TAG). Signed Mac updater downloads are included below when available."
</storycode:entry>

</storycode:doc>
