"""Actual engine restores a canonical role only through explicit review."""
import importlib.util
import os
import tempfile
import urllib.error
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    fixture = load_script("test-simple-browser")
    owned = load_script("test-agent-proposal-persistence")
    with tempfile.TemporaryDirectory(prefix="entry-restoration-") as temporary:
        folder = Path(temporary).resolve()
        root, state = folder / "workspace", folder / "state"
        root.mkdir()
        executable = os.environ.get("HICKORY_SIMPLE_TEST_ENGINE", str(ROOT / "target/source-edit-focus/debug/examples/engine_client"))
        engine = fixture.start([executable, str(root), "window", "http://127.0.0.1:1"], env={**os.environ, "HICKORY_STATE_DIR": str(state)})
        try:
            check(root, fixture.origin(engine), owned.call)
        finally:
            owned.stop_engine(fixture, engine, state)


def check(root, origin, request):
    source = '\x3ch:doc xmlns:h="http://www.hickorydocs.com/1.0">\x3ch:project sources=""/>\x3c/h:doc>'
    main_path, moved_path = "project.md", "moved.spec"

    def get(route):
        return request(origin, "GET", "/api/" + route)

    def post(route, body):
        return request(origin, "POST", "/api/" + route, body)

    def acquire(capture, token=None):
        if token is None:
            inputs = post("changes/resource-inputs", capture)
        else:
            for _ in range(2):
                inputs = post("workspace/entry/inputs", {"version": 1, "token": token, "capture": capture})
        assert not inputs["refusals"], inputs
        return {"inputs": inputs, "capture": capture}

    def entry_proposal(operation):
        selection = get("workspace/entry")
        intent = post("workspace/entry/intent", {"version": 1, "entry_selection": selection, "operation": operation})
        capture = {"context": intent["context"], "edit": intent["edit"]}
        if selection.get("source_guard"):
            capture["saved_guards"] = {selection["entry"]["path"]: selection["source_guard"]}
        return intent["title"], acquire(capture, intent["token"])

    def publish(title, proposal, identifier):
        proposal["plan"] = post("changes/plan-resources", {"context": proposal["capture"]["context"], "title": title, "edit": proposal["capture"]["edit"], "sources": proposal["inputs"]["sources"]})["plan"]
        facts = post("changes/resource-consequences", proposal)
        return post("changes/resource-save", {"request_id": identifier, "reviewed_fingerprint": facts["fingerprint"], "proposal": proposal})

    def recovery(result):
        identifier = Path(result["outcome"]["recovery"]).parent.name
        return get("changes/resource-recovery?id=" + identifier)

    def restore(result, identifier):
        record = recovery(result)
        _, proposal = entry_proposal({"kind": "restoration", "recovery_id": record["id"], "recovery_fingerprint": record["fingerprint"]})
        proposal["recovery_id"] = record["id"]
        reviewed = post("changes/prepare-restoration", proposal)
        return post("changes/restore-resources", {"request_id": identifier, "reviewed_fingerprint": reviewed["consequences"]["fingerprint"], "reviewed_recovery_fingerprint": reviewed["restoration"]["recovery_fingerprint"], "proposal": proposal})

    created = publish("Create canonical entry", acquire({"context": "create-canonical", "edit": {"documentChanges": [
        {"kind": "create", "uri": "hick:///project.md"},
        {"textDocument": {"uri": "hick:///project.md", "version": None}, "edits": [
            {"range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}}, "newText": source}
        ]},
    ]}}), "a" * 32)
    baseline = created["entry_outcome"]["current_entry"]

    def assert_entry(result, path, generation):
        assert result["outcome"]["complete"] and result["entry_outcome"]["state"] == "confirmed", result
        expected = {**baseline, "path": path, "generation": baseline["generation"] + generation}
        assert result["entry_outcome"]["current_entry"] == expected == get("workspace/entry")["entry"]
        for relative in (main_path, moved_path):
            if relative == path:
                assert (root / relative).read_text() == source
            else:
                assert not (root / relative).exists()

    assert_entry(created, main_path, 0)
    removed = restore(created, "b" * 32)
    assert_entry(removed, None, 1)
    assert_entry(restore(removed, "c" * 32), main_path, 2)
    title, proposal = entry_proposal({"kind": "move", "destination": moved_path})
    moved = publish(title, proposal, "d" * 32)
    assert_entry(moved, moved_path, 3)

    journals = root / ".hick-cache/resource-changes"
    original = set(journals.iterdir())
    record = recovery(moved)
    generic = acquire({"context": "unbound-inverse", "edit": {"changes": {"hick:///" + path: [] for path in (main_path, moved_path)}}})
    try:
        post("changes/prepare-restoration", {**generic, "recovery_id": record["id"]})
    except urllib.error.HTTPError as error:
        detail = error.read().decode()
        assert error.code == 409 and "fresh workspace-entry review" in detail, (error.code, detail)
    else:
        raise AssertionError("Unbound inverse acquired a retained main transition")
    assert set(journals.iterdir()) == original
    assert_entry(moved, moved_path, 3)
    assert_entry(restore(moved, "e" * 32), main_path, 4)
    print("PASS: explicit canonical creation Undo/Redo, unbound Move inverse refusal, guarded Move inverse, exact entry generations and bytes")


if __name__ == "__main__":
    main()
