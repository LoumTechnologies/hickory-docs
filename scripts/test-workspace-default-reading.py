#!/usr/bin/env python3
"""Drive current-entry proof through compiler and verification routes."""
import importlib.util
import os
from pathlib import Path
import tempfile
import urllib.error

ROOT = Path(__file__).resolve().parents[1]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def expect_refusal(call, origin, route, body):
    try:
        call(origin, "POST", route, body)
    except urllib.error.HTTPError as error:
        detail = error.read().decode()
        assert error.code == 409 and "entry" in detail.lower(), (error.code, detail)
    else:
        raise AssertionError(f"Accepted obsolete entry proof in {route}")


def main():
    fixture = load("fixture", "scripts/test-simple-browser.py")
    persistence = load("persistence", "scripts/test-agent-proposal-persistence.py")
    call = persistence.call
    with tempfile.TemporaryDirectory(prefix="hickory-default-reading-") as temporary:
        folder = Path(temporary).resolve()
        root = folder / "workspace"
        root.mkdir()
        state = folder / "state"
        source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="part.md"/></h:doc>'
        part = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="out.py">count = 2</h:file></h:doc>'
        original = root / "project.md"
        original.write_text(source)
        (root / "part.md").write_text(part)
        binary = os.environ.get("HICKORY_SIMPLE_TEST_ENGINE", str(ROOT / "target/source-edit-focus/debug/examples/engine_client"))
        engine = fixture.start(
            [binary, str(root), "window", "http://127.0.0.1:1"],
            env={**os.environ, "HICKORY_STATE_DIR": str(state)},
        )
        try:
            origin = fixture.origin(engine)
            proof = call(origin, "GET", "/api/workspace/entry")
            explicit = {"entry": "project.md", "focus": "out.py", "sources": {}}
            construction = {**explicit, "entry_selection": proof}
            context = {
                **construction, "version": 1, "missing_sources": [],
                "unaddressed_sources": {}, "saved_guards": {},
            }
            captured = call(origin, "POST", "/api/construction/reading", construction)
            assert captured["entry"] == "project.md" and captured["materialized"] is False
            assert captured["output_count"] == 1
            behavior = call(origin, "POST", "/api/behavior/reading", context)
            assert behavior["context"]["entry_selection"] == proof
            expect_refusal(call, origin, "/api/construction/reading", {**construction, "entry": "part.md"})

            replacement = root / "replacement"
            replacement.write_text(source)
            os.replace(replacement, original)
            fresh = call(origin, "GET", "/api/workspace/entry")
            assert fresh["entry"] == proof["entry"] and fresh["source_hash"] == proof["source_hash"]
            assert fresh["source_guard"] != proof["source_guard"]
            expect_refusal(call, origin, "/api/construction/reading", construction)
            expect_refusal(call, origin, "/api/behavior/reading", context)
            expect_refusal(call, origin, "/api/behavior/verify", {
                "version": 1, "engine_workspace_id": behavior["engine_workspace_id"],
                "workspace_id": behavior["workspace_id"], "reading_fingerprint": behavior["reading_fingerprint"],
                "check_id": "old-check", "request_id": "1" * 32, "context": context, "profile": None,
            })
            manual = call(origin, "POST", "/api/construction/reading", explicit)
            assert manual["entry"] == "project.md" and manual["output_count"] == 1
            next_read = call(origin, "POST", "/api/behavior/reading", {**context, "entry_selection": fresh})
            assert next_read["context"]["entry_selection"] == fresh
            assert original.read_text() == source and (root / "part.md").read_text() == part
            assert not (root / "out.py").exists()
        finally:
            persistence.stop_engine(fixture, engine, state)
    print("PASS current entry proof in both readings; foreign address and equal-byte physical replacement rejected before reading or verification; explicit reading preserved; no source/output effects")


if __name__ == "__main__":
    main()
