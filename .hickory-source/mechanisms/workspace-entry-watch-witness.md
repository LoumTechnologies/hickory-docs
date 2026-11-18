<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise folder and explicit construction watches through the engine

For maintainers changing entry selection in the always-on CLI. The fixture calls
the production `engine::up` with an isolated workspace and engine state. A
folder subscription starts with the retained project entry. An explicit file
subscription reads its supplied address, including a non-Markdown extension.
The watches publish only their own generated entries.

<ui:copy id="workspace-entry-watch-live-witness">
#!/usr/bin/env python3
"""Check genuine folder and explicit authoring subscriptions in isolated fixtures."""
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def load(name, script):
    spec = importlib.util.spec_from_file_location(name, ROOT / script)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def entry(part):
    return f'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="{part}"/></h:doc>'


def content(output, text):
    return f'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="{output}">{text}</h:file></h:doc>'


def main():
    fixture = load("fixture", "scripts/test-simple-browser.py")
    owned = load("owned", "scripts/test-agent-proposal-persistence.py")
    binary = os.environ.get("HICKORY_SIMPLE_TEST_ENGINE", str(ROOT / "target/source-edit-focus/debug/examples/engine_client"))
    with tempfile.TemporaryDirectory(prefix="hickory-entry-watch-") as temporary:
        parent = Path(temporary).resolve()
        for explicit in (False, True):
            folder = parent / ("explicit" if explicit else "implicit")
            root = folder / "workspace"
            root.mkdir(parents=True)
            state = folder / "state"
            main = root / "project.md"
            selected = root / "literal #entry.spec" if explicit else main
            selected.write_text(entry("part.md"))
            if explicit:
                main.write_text(entry("other.md"))
                (root / "other.md").write_text(content("other.txt", "foreign"))
            source = root / "part.md"
            source.write_text(content("out.txt", "first"))
            output = root / "out.txt"
            watch = fixture.start(
                [binary, "--watch", str(selected if explicit else root)],
                env={**os.environ, "HICKORY_STATE_DIR": str(state)},
            )
            try:
                def read_output():
                    if watch.poll() is not None:
                        raise AssertionError(watch.stdout.read())
                    return output.read_text() if output.is_file() else None

                fixture.eventually(read_output, "first")
                source.write_text(content("out.txt", "second"))
                fixture.eventually(read_output, "second")
                assert selected.read_text() == entry("part.md")
                assert not (root / "other.txt").exists()
            finally:
                owned.stop_engine(fixture, watch, state)
            if explicit:
                plain = root / "plain.spec"
                plain.write_text("ordinary source")
                refused = subprocess.run(
                    [binary, "--watch", str(plain)],
                    env={**os.environ, "HICKORY_STATE_DIR": str(state)},
                    capture_output=True, text=True, timeout=15,
                )
                assert refused.returncode != 0 and "project" in refused.stderr.lower(), refused.stderr
                assert plain.read_text() == "ordinary source" and not (root / "other.txt").exists()
    print("PASS implicit and explicit watch publication, repeated updates, exact explicit entry with no fallback")


if __name__ == "__main__":
    main()
</ui:copy>

<ui:file path="scripts/test-workspace-entry-watch.py"><ui:paste select="#workspace-entry-watch-live-witness"/></ui:file>
</ui:doc>
