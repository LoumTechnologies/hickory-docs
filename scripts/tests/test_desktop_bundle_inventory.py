"""Focused stale-installer safety checks; no compiler, installer or upload runs.
Guarantee: docs/guarantees/release/desktop-artifacts-come-from-the-current-build.md
"""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]


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


if __name__ == "__main__":
    unittest.main()
