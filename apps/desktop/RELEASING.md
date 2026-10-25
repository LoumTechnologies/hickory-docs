# Publish Unstable from a Linux agent

For a maintainer working on Linux who wants the Apple Silicon desktop app on
another Mac to offer each new build as an update.

Run `just release-unstable` after committing and pushing. It sends the committed
source to a Mac over SSH, builds a DMG and signed update archive there, and
uploads them to the GitHub release titled **Unstable** (tag `unstable`). It
creates the release when needed. GitHub Actions does not participate.

## Set up the Mac builder once

Use an Apple Silicon Mac with Remote Login enabled. It can be the Mac running
the app, provided it remains awake and reachable from Linux. The builder needs
Xcode Command Line Tools, Rust, Node/npm, Just, uv, and Python 3. Install the
Tauri CLI and generate a persistent updater signing key on that Mac:

```sh
cargo install tauri-cli --version '^2' --locked
mkdir -p ~/.config/hickory-docs
chmod 700 ~/.config/hickory-docs
cargo tauri signer generate -w ~/.config/hickory-docs/unstable.key
```

Keep that private key backed up: installed apps trust its matching public key.
Create `~/.config/hickory-docs/unstable.env` on the Mac:

```sh
TAURI_SIGNING_PRIVATE_KEY="$HOME/.config/hickory-docs/unstable.key"
TAURI_SIGNING_PRIVATE_KEY_PASSWORD='your key password'
HICKORY_UPDATER_PUBLIC_KEY='paste the complete content of unstable.key.pub here'
APPLE_SIGNING_IDENTITY='Developer ID Application: Your Name (TEAMID)'
# For Apple notarization, also configure one of Tauri's supported credential sets:
APPLE_ID='your Apple ID'
APPLE_PASSWORD='an app-specific password'
APPLE_TEAM_ID='TEAMID'
```

```sh
chmod 600 ~/.config/hickory-docs/unstable.env ~/.config/hickory-docs/unstable.key
```

This is trusted shell configuration, sourced by the builder. Credentials stay
on the Mac. Its login shell must expose the tools on PATH. For personal builds,
`APPLE_SIGNING_IDENTITY='-'` requests ad-hoc signing; omit the Apple notarization
credentials in that case. Downloaded ad-hoc builds may need manual macOS
approval. Developer ID signing and notarization give the normal download and
open experience. See [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/).

## Configure and publish from Linux

Use a Python 3.12+ installation, Git, OpenSSH, Just, and an authenticated GitHub
CLI (`gh auth login`) with Contents write permission on this repository.
Configure an SSH alias that reaches the builder without a password prompt,
then export its name in the Linux agent's environment:

```sh
export HICKORY_MAC_BUILDER=hickory-mac
ssh -oBatchMode=yes hickory-mac 'uname -sm'
# Expected: Darwin arm64

git add <files you changed>
git commit -m 'Describe the change'
git push
just release-unstable
```

The publisher refuses tracked uncommitted changes and a HEAD commit missing
from GitHub. Untracked files are excluded. It builds a fresh source archive,
so the Mac needs no clone or GitHub credentials. Compilation caches live under
`~/.cache/hickory-docs-unstable/` on the Mac. A lock there serializes publication
across agents using this builder; use the same builder for this channel.

Each version includes a UTC timestamp and commit hash, for example
`0.1.0-unstable.20261025123456789012.gabcdef01`. Timestamps make subsequent
builds sort newer under SemVer. A `g` before the hash keeps all-numeric
hashes valid under SemVer too. The rolling tag moves to the published commit after the updater manifest is
uploaded. Each artifact's version and updater notes also identify its source
commit.

## Install once, then accept updates

Download the newest Apple Silicon DMG from the
[Unstable release](https://github.com/LoumTechnologies/hickory-docs/releases/tag/unstable),
open it, and copy **Hickory Docs.app** to a writable Applications folder.
Launch that copy. An older local install needs this initial manual replacement
to gain the updater and its public key.

Packaged Unstable Apple Silicon builds check at startup after the window opens
and every 30 minutes. Stable builds and development runs do not check. When a
newer Unstable version is available, a native dialog offers **Download and
install** or **Later**. Later defers that version until the next launch.
Installation verifies the archive's signature, replaces the app on disk, and
asks you to save, quit all Hickory Docs windows, and reopen. It never forces a
restart past the editor's save/discard or workspace shutdown checks.

Network failures leave the running app usable and retry on the next interval.
An installation failure shows an error and allows a later retry. Open folder
processes share an installation lock so they cannot replace the app at once.

The DMG is the first-install download. The updater downloads a signed
`.app.tar.gz`, as required by the
[Tauri updater](https://v2.tauri.app/plugin/updater/). Payloads upload before
`unstable.json` switches to the new version. Previous payloads remain available
for clients that fetched an earlier manifest. Do not delete old assets without
considering those clients. The former automatic Unstable Actions publisher was
removed because it deleted the entire release, including updater assets.

## Focused validation

Run `just test-unstable-release` on Linux to check publication order, signature
staging, and channel/version policy. This uses mocked GitHub/build calls and
does not publish or prove macOS installation works.

## Check status or recover an interrupted publish

The Just command prints build output and exits after publication. To keep a
log yourself, run `just release-unstable > unstable-build.log 2>&1`. Inspect the
release assets with:

```sh
gh release view unstable --repo LoumTechnologies/hickory-docs
```

After a killed SSH session, confirm no other publish is active, then remove
`~/.cache/hickory-docs-unstable/publish.lock` on the builder with `rmdir` before
retrying. Abandoned `build.*` directories in that cache can be removed once
no build is using them. A failed payload upload leaves the previous manifest
in place; a failure while replacing the manifest can temporarily remove it,
and another successful publish restores checks. GitHub's asset replacement
is not atomic.

Post-commit regression checks are separate from publishing. Run
`just checks-status` yourself for the local hook results. Its output is also
recorded under the Git common directory in `checks/hook.log`. For GitHub checks,
use `gh run list` and `gh run view <run-id>`. Publishing does not wait for or
inspect those checks. This command publishes Apple Silicon desktop assets only;
existing CLI and other-platform assets remain at their previous versions.
