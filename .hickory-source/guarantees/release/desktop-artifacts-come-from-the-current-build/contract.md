<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Desktop collection excludes unchanged cached installers

For: Maintainers publishing installers and people checking what a download contains.

Given an isolated desktop bundle build, when `scripts/dist-desktop.sh` collects
installable files, then it selects only files whose existence or observed
identity changed since the invocation's pre-build snapshot. An unchanged cached
installer is never renamed as the current version merely because it remains in
the bundle directory. No changed installable file, or multiple changed files of
one format, refuses collection before replacing the previous dist selection.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/desktop-artifacts-come-from-the-current-build.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-5dec26c79103"></a>
## `scripts/desktop-bundle-inventory.mjs`

Related contracts: [Desktop collection excludes unchanged cached installers](contract.md).

<a id="f-5dec26c79103-hash_bang_line"></a>
### `hash_bang_line`

<storycode:copy id="f-5dec26c79103-hash_bang_line">
#!/usr/bin/env node
</storycode:copy>
<a id="f-5dec26c79103-import_statement"></a>
### `import_statement`

A local build-change check, not artifact authentication or concurrent-build locking.

<storycode:copy id="f-5dec26c79103-import_statement">
// A local build-change check, not artifact authentication or concurrent-build locking.
import { createHash } from "node:crypto";
</storycode:copy>
<a id="f-5dec26c79103-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-5dec26c79103-import_statement-2">
import { lstatSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
</storycode:copy>
<a id="f-5dec26c79103-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-5dec26c79103-import_statement-3">
import { join, resolve } from "node:path";
</storycode:copy>
<a id="f-5dec26c79103-mode-directory-manifest"></a>
### `[mode, directory, manifest]`

<storycode:copy id="f-5dec26c79103-mode-directory-manifest">
const [mode, directory, manifest] = process.argv.slice(2);
</storycode:copy>
<a id="f-5dec26c79103-if_statement"></a>
### `if_statement`

<storycode:copy id="f-5dec26c79103-if_statement">
if (!["snapshot", "changed"].includes(mode) || !directory || !manifest) throw Error("Usage: desktop-bundle-inventory.mjs snapshot|changed BUNDLE_DIR MANIFEST");
</storycode:copy>
<a id="f-5dec26c79103-installable"></a>
### `installable`

<storycode:copy id="f-5dec26c79103-installable">
const installable = path => /\.(deb|rpm|AppImage|dmg|msi|exe|app\.tar\.gz)$/.test(path);
</storycode:copy>
<a id="f-5dec26c79103-inventory"></a>
### `inventory`

<storycode:copy id="f-5dec26c79103-inventory">
function inventory(root) {
  const entries = {};
  function walk(path) {
    let stat;
    try { stat = lstatSync(path, { bigint: true }); } catch (error) { if (error.code === "ENOENT") return; throw error; }
    if (stat.isDirectory()) { for (const name of readdirSync(path).sort()) walk(join(path, name)); }
    else if (stat.isFile() && installable(path)) entries[resolve(path)] = {
      size: String(stat.size), modified_ns: String(stat.mtimeNs), inode: String(stat.ino), device: String(stat.dev),
      sha256: createHash("sha256").update(readFileSync(path)).digest("hex"),
    };
  }
  walk(root); return entries;
}
</storycode:copy>
<a id="f-5dec26c79103-root"></a>
### `root`

<storycode:copy id="f-5dec26c79103-root">
const root = resolve(directory);
</storycode:copy>
<a id="f-5dec26c79103-if_statement-2"></a>
### `if_statement`

<storycode:copy id="f-5dec26c79103-if_statement-2">
if (mode === "snapshot") writeFileSync(manifest, JSON.stringify({ version: 1, root, entries: inventory(root) }));
else {
  const before = JSON.parse(readFileSync(manifest, "utf8"));
  if (before.version !== 1 || before.root !== root || !before.entries || typeof before.entries !== "object") throw Error("Bundle snapshot does not belong to this build output");
  const changed = Object.entries(inventory(root)).filter(([path, identity]) => JSON.stringify(before.entries[path]) !== JSON.stringify(identity)).map(([path]) => path);
  const formats = new Set();
  for (const path of changed) {
    const format = path.endsWith(".app.tar.gz") ? "app.tar.gz" : path.split(".").at(-1);
    if (formats.has(format)) throw Error(`Ambiguous current bundle format ${format}; use an isolated build output`);
    formats.add(format);
  }
  for (const path of changed) process.stdout.write(`${path}\0`);
}
</storycode:copy>

### Assemble `scripts/desktop-bundle-inventory.mjs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/desktop-bundle-inventory.mjs"><storycode:paste select="#f-5dec26c79103-hash_bang_line"/><storycode:paste select="#f-5dec26c79103-import_statement"/><storycode:paste select="#f-5dec26c79103-import_statement-2"/><storycode:paste select="#f-5dec26c79103-import_statement-3"/><storycode:paste select="#f-5dec26c79103-mode-directory-manifest"/><storycode:paste select="#f-5dec26c79103-if_statement"/><storycode:paste select="#f-5dec26c79103-installable"/><storycode:paste select="#f-5dec26c79103-inventory"/><storycode:paste select="#f-5dec26c79103-root"/><storycode:paste select="#f-5dec26c79103-if_statement-2"/></storycode:file>

<a id="output-f5d2783ec820"></a>
## `scripts/tests/test_desktop_bundle_inventory.py`

Related contracts: [Desktop collection excludes unchanged cached installers](contract.md).

<a id="f-f5d2783ec820-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f5d2783ec820-expression_statement">
"""Focused stale-installer safety checks; no compiler, installer or upload runs.
Guarantee: docs/guarantees/release/desktop-artifacts-come-from-the-current-build.md
"""
</storycode:copy>
<a id="f-f5d2783ec820-os"></a>
### `os`

<storycode:copy id="f-f5d2783ec820-os">
import os
</storycode:copy>
<a id="f-f5d2783ec820-Path"></a>
### `Path`

<storycode:copy id="f-f5d2783ec820-Path">
from pathlib import Path
</storycode:copy>
<a id="f-f5d2783ec820-shutil"></a>
### `shutil`

<storycode:copy id="f-f5d2783ec820-shutil">
import shutil
</storycode:copy>
<a id="f-f5d2783ec820-subprocess"></a>
### `subprocess`

<storycode:copy id="f-f5d2783ec820-subprocess">
import subprocess
</storycode:copy>
<a id="f-f5d2783ec820-tempfile"></a>
### `tempfile`

<storycode:copy id="f-f5d2783ec820-tempfile">
import tempfile
</storycode:copy>
<a id="f-f5d2783ec820-unittest"></a>
### `unittest`

<storycode:copy id="f-f5d2783ec820-unittest">
import unittest

</storycode:copy>
<a id="f-f5d2783ec820-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f5d2783ec820-expression_statement-2">
REPO = Path(__file__).resolve().parents[2]


</storycode:copy>
<a id="f-f5d2783ec820-DesktopBundles"></a>
### `DesktopBundles`

<storycode:copy id="f-f5d2783ec820-DesktopBundles">
class DesktopBundles(unittest.TestCase):
    def fixture(self, root, mode):
        scripts = root / "scripts"
        scripts.mkdir()
        for name in ("dist-desktop.sh", "desktop-bundle-inventory.mjs"):
            shutil.copy2(REPO / "scripts" / name, scripts / name)
        (root / "apps/desktop/src-tauri").mkdir(parents=True)
        (root / "apps/web/node_modules").mkdir(parents=True)
        binary = root / "fake-bin"
        binary.mkdir()
        commands = {
            "cargo-tauri": "#!/bin/sh\nexit 0\n",
            "just": "#!/bin/sh\nmkdir -p .dev\nprintf '{}' > .dev/desktop-cli.json\n",
            "rustc": "#!/bin/sh\nprintf 'host: x86_64-unknown-linux-gnu\\n'\n",
            "cargo": """#!/bin/sh
set -eu
out=target/release/bundle/deb
mkdir -p "$out"
case "$FAKE_BUNDLE_MODE" in
 fresh) printf fresh > "$out/current.deb" ;;
 ambiguous) printf fresh > "$out/current.deb"; printf other > "$out/other.deb" ;;
esac
""",
        }
        for name, text in commands.items():
            path = binary / name
            path.write_text(text)
            path.chmod(0o755)
        bundle = root / "apps/desktop/src-tauri/target/release/bundle"
        old = bundle / "appimage/old.AppImage"
        old.parent.mkdir(parents=True)
        old.write_text("old installer")
        # A future timestamp must not make an unchanged cached artifact current.
        os.utime(old, (2100000000, 2100000000))
        previous = root / "dist/desktop/previous.deb"
        previous.parent.mkdir(parents=True)
        previous.write_text("previous selected output")
        env = dict(os.environ, PATH=f"{binary}{os.pathsep}{os.environ['PATH']}", FAKE_BUNDLE_MODE=mode)
        env.pop("CARGO_TARGET_DIR", None)
        return subprocess.run(["bash", str(scripts / "dist-desktop.sh"), "0.1.0-test.current", "x86_64-unknown-linux-gnu"], cwd=root, env=env, capture_output=True, text=True)

    def test_cached_formats_are_not_renamed_as_current(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            result = self.fixture(root, "fresh")
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            files = sorted(path.name for path in (root / "dist/desktop").iterdir())
            name = "hickory-docs-0.1.0-test.current-x86_64-unknown-linux-gnu.deb"
            self.assertEqual(files, [name, name + ".sha256"])
            self.assertEqual((root / "dist/desktop" / name).read_text(), "fresh")

    def test_no_fresh_bundle_preserves_previous_selection_and_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            result = self.fixture(root, "none")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("cached installers were not selected", result.stderr)
            self.assertEqual((root / "dist/desktop/previous.deb").read_text(), "previous selected output")

    def test_two_changed_bundles_of_one_format_refuse_before_replacement(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            result = self.fixture(root, "ambiguous")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Ambiguous current bundle format", result.stderr)
            self.assertTrue((root / "dist/desktop/previous.deb").is_file())


</storycode:copy>
<a id="f-f5d2783ec820-if_statement"></a>
### `if_statement`

<storycode:copy id="f-f5d2783ec820-if_statement">
if __name__ == "__main__":
    unittest.main()
</storycode:copy>

### Assemble `scripts/tests/test_desktop_bundle_inventory.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/tests/test_desktop_bundle_inventory.py"><storycode:paste select="#f-f5d2783ec820-expression_statement"/><storycode:paste select="#f-f5d2783ec820-os"/><storycode:paste select="#f-f5d2783ec820-Path"/><storycode:paste select="#f-f5d2783ec820-shutil"/><storycode:paste select="#f-f5d2783ec820-subprocess"/><storycode:paste select="#f-f5d2783ec820-tempfile"/><storycode:paste select="#f-f5d2783ec820-unittest"/><storycode:paste select="#f-f5d2783ec820-expression_statement-2"/><storycode:paste select="#f-f5d2783ec820-DesktopBundles"/><storycode:paste select="#f-f5d2783ec820-if_statement"/></storycode:file>

## Implementation review

### Implementation

- [scripts/desktop-bundle-inventory.mjs](contract.md#output-5dec26c79103)
- [scripts/dist-desktop.sh](../../../mechanisms/scripts/dist-desktop.sh.md#output-928dfd6fa282)

### Tests and fixtures

- [scripts/tests/test_desktop_bundle_inventory.py](contract.md#output-f5d2783ec820)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
