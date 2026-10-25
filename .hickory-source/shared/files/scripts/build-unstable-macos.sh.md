<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# scripts/build-unstable-macos.sh

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Manual Unstable Publication](../../../guarantees/release/only-a-green-commit-becomes-downloadable/contract.md)
- [Unstable Updates Require Consent](../../../guarantees/release/unstable-updates-require-consent/contract.md)

Captured SHA-256: `d5f0c12066eaf1fd1e1b5a0ec7e6c3e3b900663e46c3ebfde6b9a44c95d3b7ff`.

<book:entry path="scripts/build-unstable-macos.sh" executable="true">
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
</book:entry>

</book:doc>
