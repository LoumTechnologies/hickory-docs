<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/build-python-agent-runtime.sh

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5f2e12b18746"></a>
## `scripts/build-python-agent-runtime.sh`

Related contracts: [A desktop app bundles its own Python agent](../../guarantees/agent/a-desktop-app-bundles-its-python-agent/contract.md), [A Local Install Is a Launchable macOS App](../../guarantees/release/a-local-install-is-a-launchable-macos-app/contract.md), [The Desktop App Is Its Own Download](../../guarantees/release/the-desktop-app-is-its-own-download/contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="scripts/build-python-agent-runtime.sh" executable="true">
#!/usr/bin/env bash
# Build a self-contained Python agent runtime to ship inside the desktop app.
#
# The source of truth is the locked environment under apps/agent, installed by
# `uv` with `--no-editable` (the sandbox cannot import an editable checkout).
# `uv` builds that environment as a virtualenv whose interpreter is a symlink
# into a relocatable python-build-standalone install in the user's data
# directory. A .dmg can carry neither the symlink target nor a path that only
# exists on the maintainer's machine, so this script folds the two together:
# the standalone interpreter and its standard library, the environment's
# installed packages, and the `monty` interpreter — in one directory whose
# `bin/python` resolves inside it.
#
# Output (gitignored, under .dev/):
#   .dev/python-agent/runtime/            the runtime to bundle
#   .dev/desktop-python-agent.json        the Tauri `bundle.macOS.files` entry
#
# A desktop build stages this before `cargo tauri build`; `just local-install`
# and `scripts/dist-desktop.sh` both call it. The app finds the result through
# `hickory_cli::serve::bundled_runtime`, which reads the runtime's location and
# never requires the person to type a path.
set -euo pipefail
cd "$(dirname "$0")/.."

OUT=".dev/python-agent/runtime"
CONFIG=".dev/desktop-python-agent.json"

if ! command -v uv >/dev/null 2>&1; then
  echo "::error::uv is required to build the Python agent runtime." >&2
  echo "  Install it: https://docs.astral.sh/uv/getting-started/installation/" >&2
  exit 1
fi

echo "==> installing the locked Python agent environment"
# The same lock `just test-python-agent` and CI use; `--no-editable` is
# load-bearing because the confined worker imports an installed package.
uv sync --project apps/agent --locked --no-editable

VENV="apps/agent/.venv"
PYVENV_CFG="$VENV/pyvenv.cfg"
[ -f "$PYVENV_CFG" ] || {
  echo "::error::uv did not create $PYVENV_CFG." >&2
  exit 1
}
# `home` is the standalone interpreter's bin directory: the environment is a
# thin layer over a relocatable CPython that must travel with it.
home="$(sed -n 's/^home = //p' "$PYVENV_CFG" | head -1)"
[ -n "$home" ] || {
  echo "::error::no interpreter home in $PYVENV_CFG." >&2
  exit 1
}
STANDALONE="$(cd "$home/.." && pwd -P)"
[ -d "$STANDALONE/lib" ] || {
  echo "::error::$STANDALONE does not look like a Python installation." >&2
  exit 1
}

# One stdlib version directory, e.g. python3.14. Not hardcoded: the lock pins a
# version and a future bump must not silently ship the wrong packages.
VENV_LIB="$(cd "$VENV/lib" && ls -d python* | head -1)"
[ -n "$VENV_LIB" ] || {
  echo "::error::no python library directory under $VENV/lib." >&2
  exit 1
}

echo "==> assembling $OUT"
rm -rf "$OUT"
mkdir -p "$OUT"
# `-R -P` copies the interpreter and its symlinks without following them, so
# `bin/python` stays a relative link to the real binary instead of a duplicate.
cp -R -P "$STANDALONE/." "$OUT/"
# The installed packages overlay the stdlib's (empty) site-packages.
mkdir -p "$OUT/lib/$VENV_LIB/site-packages"
cp -R -P "$VENV/lib/$VENV_LIB/site-packages/." "$OUT/lib/$VENV_LIB/site-packages/"
# The worker pins `monty` beside its interpreter (`Path(sys.executable).parent
# / "monty"`), so it has to land in bin.
cp "$VENV/bin/monty" "$OUT/bin/monty"
# `hickory_executor_sandbox::agent_process` accepts exactly an installed
# virtual environment and demands a `pyvenv.cfg` beside `bin/`; this is that
# marker. It points at the bundled interpreter rather than the builder's.
cat > "$OUT/pyvenv.cfg" <<EOF
home = $OUT/bin
implementation = CPython
include-system-site-packages = false
prompt = hickory-agent-worker
EOF
# The sandbox needs a stable, version-independent way in.
if [ ! -e "$OUT/bin/python" ]; then
  ln -s "$(basename "$(ls "$OUT"/bin/python3.* | head -1)")" "$OUT/bin/python"
fi

echo "==> verifying the assembled runtime imports the worker"
"$OUT/bin/python" -I -B -c 'import hickory_agent_worker' || {
  echo "::error::the bundled runtime cannot import hickory_agent_worker." >&2
  exit 1
}
[ -x "$OUT/bin/monty" ] || {
  echo "::error::monty is missing from $OUT/bin." >&2
  exit 1
}

# A notarized release rejects unsigned nested code, and Tauri signs only the
# app bundle, not files copied into Resources. Signing each native binary now
# embeds a signature the later app signature can seal. Ad-hoc (`-`) local
# builds skip this: they are not notarized, and `codesign --verify --deep`
# accepts unsigned resources there.
if [ -n "${APPLE_SIGNING_IDENTITY:-}" ] && command -v codesign >/dev/null 2>&1; then
  echo "==> signing the bundled runtime with \"$APPLE_SIGNING_IDENTITY\""
  # Only native Mach-O files are code; signing the `.py` launchers beside
  # them would be meaningless noise. `file` is the honest test.
  while IFS= read -r -d '' candidate; do
    file -b "$candidate" | grep -q 'Mach-O' || continue
    codesign --force -s "$APPLE_SIGNING_IDENTITY" "$candidate" >/dev/null 2>&1 || {
      echo "::error::could not sign $candidate." >&2
      exit 1
    }
  done < <(
    find "$OUT/bin" -type f -print0
    find "$OUT/lib" -type f \( -name '*.so' -o -name '*.dylib' \) -print0
  )
fi

mkdir -p "$(dirname "$CONFIG")"
# The map is target-under-Contents -> source, resolved relative to
# apps/desktop/src-tauri/tauri.conf.json. `files` (unlike `resources`) copies
# through `copy_dir`, which preserves the symlinks this runtime depends on.
printf '%s\n' \
  '{"bundle":{"macOS":{"files":{"Resources/python-agent":"../../../.dev/python-agent/runtime"}}}}' \
  > "$CONFIG"

echo "==> pinned Python agent runtime ready: $OUT"
du -sh "$OUT" 2>/dev/null || true
echo "    Tauri config: $CONFIG"
</storycode:entry>

</storycode:doc>
