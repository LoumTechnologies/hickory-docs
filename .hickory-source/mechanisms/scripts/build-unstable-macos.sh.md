<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/build-unstable-macos.sh

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f2dcad3b78b2"></a>
## `scripts/build-unstable-macos.sh`

Related contracts: [Publish the committed Unstable build chosen by the maintainer](../../guarantees/release/only-a-green-commit-becomes-downloadable/contract.md), [Unstable Updates Require Consent](../../guarantees/release/unstable-updates-require-consent/contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="scripts/build-unstable-macos.sh" executable="true">
#!/usr/bin/env bash
# Runs inside a committed source archive on the SSH builder.
set -euo pipefail
cd "$(dirname "$0")/.."
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || {
  echo 'An Apple Silicon Mac is required.' >&2; exit 1;
}
# Trusted, local shell configuration: secrets stay on the builder.
set -a
source "$HOME/.config/hickory-docs/unstable.env"
set +a
: "${TAURI_SIGNING_PRIVATE_KEY:?Set the updater private key in unstable.env}"
: "${HICKORY_UPDATER_PUBLIC_KEY:?Set the updater public key content in unstable.env}"
: "${APPLE_SIGNING_IDENTITY:?Set a Developer ID signing identity, or - for personal ad-hoc builds}"
for tool in cargo cargo-tauri rustc node npm just uv python3; do
  command -v "$tool" >/dev/null || { echo "Missing builder tool: $tool" >&2; exit 1; }
done
# Reuse compilation caches while each publish gets fresh source and artifacts.
cache="$HOME/.cache/hickory-docs-unstable"
mkdir -p "$cache/cli-target" "$cache/desktop-target" .dev
ln -s "$cache/cli-target" target
ln -s "$cache/desktop-target" apps/desktop/src-tauri/target
unset CARGO_TARGET_DIR
python3 - <<'PY'
import json, os
from pathlib import Path
Path('.dev/unstable-updater.json').write_text(json.dumps({
    'bundle': {'createUpdaterArtifacts': True},
    'plugins': {'updater': {
        'pubkey': os.environ['HICKORY_UPDATER_PUBLIC_KEY'],
        'endpoints': ['https://github.com/LoumTechnologies/hickory-docs/releases/download/unstable/unstable.json'],
    }},
}))
PY
export HICKORY_UPDATER_CONFIG=../../../.dev/unstable-updater.json
export TAURI_BUNDLES=app,dmg
bash scripts/dist-desktop.sh "$1" aarch64-apple-darwin
</storycode:entry>

</storycode:doc>
