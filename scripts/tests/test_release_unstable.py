"""Focused publishing safety tests; no SSH, build, or GitHub writes."""
import importlib.util
import json
import os
import shutil
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("release_unstable", Path(__file__).parents[1] / "release-unstable.py")
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


# Guarantee: docs/guarantees/release/only-a-green-commit-becomes-downloadable.md
class PublishTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.version = "0.1.0-unstable.20261025123456789012.abcdef01"
        prefix = f"hickory-docs-{self.version}-aarch64-apple-darwin"
        for suffix in (".dmg", ".app.tar.gz", ".app.tar.gz.sig"):
            (self.directory / (prefix + suffix)).write_text("signed artifact")

    def publish(self):
        release.publish(self.directory, self.version, "abcdef01")

    @patch.object(release, "run")
    @patch.object(release, "release_exists", return_value=False)
    def test_creates_missing_release_and_publishes_manifest_after_payload(self, exists, run):
        self.publish()
        calls = [call.args for call in run.call_args_list]
        self.assertEqual(calls[0][:3], ("gh", "release", "create"))
        uploads = [call for call in calls if call[:3] == ("gh", "release", "upload")]
        self.assertEqual(len(uploads), 2)
        self.assertNotIn("--clobber", uploads[0])
        self.assertTrue(any(arg.endswith(".app.tar.gz.sig") for arg in uploads[0]))
        self.assertEqual(uploads[1][-2:], (str(self.directory / "unstable.json"), "--clobber"))
        data = json.loads((self.directory / "unstable.json").read_text())
        platform = data["platforms"]["darwin-aarch64"]
        self.assertEqual(platform["signature"], "signed artifact")
        self.assertTrue(platform["url"].endswith(".app.tar.gz"))
        self.assertIn("/releases/download/unstable/", platform["url"])

    @patch.object(release, "run")
    @patch.object(release, "release_exists", return_value=True)
    def test_existing_release_is_never_deleted(self, exists, run):
        self.publish()
        for call in run.call_args_list:
            self.assertNotIn(call.args[2], ("create", "delete"))

    @patch.object(release, "run", side_effect=subprocess.CalledProcessError(1, "gh"))
    @patch.object(release, "release_exists", return_value=True)
    def test_failed_payload_upload_leaves_previous_manifest_alone(self, exists, run):
        with self.assertRaises(subprocess.CalledProcessError):
            self.publish()
        self.assertEqual(run.call_count, 1)
        self.assertNotIn("--clobber", run.call_args.args)

    @patch.object(release, "run")
    def test_missing_signature_refuses_publication(self, run):
        next(self.directory.glob("*.sig")).unlink()
        with self.assertRaisesRegex(ValueError, "Missing build artifact"):
            self.publish()
        run.assert_not_called()

    @patch.object(release, "run")
    def test_empty_signature_refuses_publication(self, run):
        next(self.directory.glob("*.sig")).write_text("\n")
        with self.assertRaisesRegex(ValueError, "empty"):
            self.publish()
        run.assert_not_called()

    @patch.object(release, "run")
    def test_pagination_finds_an_existing_release_on_later_pages(self, run):
        run.return_value.stdout = '[[{"tag_name":"v0.1.0"}],[{"tag_name":"unstable"}]]'
        self.assertTrue(release.release_exists())

    @patch.object(release, "run")
    def test_ssh_command_quotes_paths_and_does_not_expand_locally(self, run):
        release.remote("builder", "cd '/path with spaces' && echo '$secret'")
        self.assertEqual(run.call_args.args[:4], ("ssh", "-oBatchMode=yes", "--", "builder"))
        self.assertTrue(run.call_args.args[4].startswith("bash -lc "))


class BundleSignatureTests(unittest.TestCase):
    def test_desktop_distribution_preserves_update_archive_signature(self):
        # Guarantee: docs/guarantees/release/unstable-updates-require-consent.md
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = Path(__file__).resolve().parents[2]
            (root / "scripts").mkdir()
            for name in ("dist-desktop.sh", "desktop-bundle-inventory.mjs"):
                shutil.copy(source / "scripts" / name, root / "scripts" / name)
            (root / "apps/desktop/src-tauri").mkdir(parents=True)
            (root / "apps/web/node_modules").mkdir(parents=True)
            bin_dir = root / "fakebin"
            bin_dir.mkdir()
            tools = {
                "rustc": "echo 'host: aarch64-apple-darwin'",
                "cargo-tauri": ":",
                "just": ":",
                "cargo": """mkdir -p target/release/bundle/macos target/release/bundle/dmg
printf 'archive' > 'target/release/bundle/macos/Hickory Docs.app.tar.gz'
printf 'exact signature content' > 'target/release/bundle/macos/Hickory Docs.app.tar.gz.sig'
printf 'installer' > 'target/release/bundle/dmg/Hickory Docs.dmg'
""",
            }
            for name, script in tools.items():
                tool = bin_dir / name
                tool.write_text("#!/usr/bin/env bash\nset -eu\n" + script + "\n")
                tool.chmod(0o755)
            env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ["PATH"])
            result = subprocess.run(["bash", "scripts/dist-desktop.sh", "0.1.0-unstable.123.abcdef01",
                                     "aarch64-apple-darwin"], cwd=root, env=env,
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            out = root / "dist/desktop"
            signature = out / "hickory-docs-0.1.0-unstable.123.abcdef01-aarch64-apple-darwin.app.tar.gz.sig"
            self.assertEqual(signature.read_text(), "exact signature content")
            self.assertTrue(Path(str(signature) + ".sha256").is_file())
            self.assertEqual(len(list(out.glob("*.dmg"))), 1)


if __name__ == "__main__":
    unittest.main()
