"""Exercise the completion popup through the actual engine and child providers."""
import importlib.util
import json
import os
from pathlib import Path
import re
import socket
import sys
import tempfile
import time
ROOT=Path(__file__).resolve().parents[1]

TYPESCRIPT_PATH = "main.ts"
TYPESCRIPT_FRAGMENT = "export const result = hickUniqueAn\n"
TYPESCRIPT_GROUP_FRAGMENT = TYPESCRIPT_FRAGMENT + "export const second = hickUniqueAn\n"
TYPESCRIPT_MODULE = "export function hickUniqueAnswer() { return 42; }\n"
TYPESCRIPT_COMPLETION = (
    'import { hickUniqueAnswer } from "./answers";\n\n'
    "export const result = hickUniqueAnswer\n"
)


def typescript_fixture(root, grouped):
    provider = Path(os.environ["HICKORY_SIMPLE_TEST_TYPESCRIPT_PROVIDER"]).resolve(strict=True)
    if not provider.is_file():
        raise RuntimeError("The installed TypeScript language server entry point must be a file.")
    fragment = TYPESCRIPT_GROUP_FRAGMENT if grouped else TYPESCRIPT_FRAGMENT
    saved = {TYPESCRIPT_PATH: fragment, "answers.ts": TYPESCRIPT_MODULE}
    for path, source in saved.items():
        (root / path).write_bytes(source.encode())
    (root / "tsconfig.json").write_text(json.dumps({
        "compilerOptions": {"target": "ES2022"}, "include": ["*.ts"],
    }))
    return saved, {"typescript": {"command": ["node", str(provider), "--stdio"]}}


def typescript_acceptance(driver, root, saved, size, grouped):
    def carets(source, identifier):
        return [{"from": match.end(), "to": match.end()} for match in re.finditer(rf"\b{identifier}$", source, re.MULTILINE)]

    path = TYPESCRIPT_PATH
    before = saved[path]
    after = TYPESCRIPT_COMPLETION
    if grouped:
        after = after.replace(";", "", 1) + "export const second = hickUniqueAnswer\n"
    before_carets = carets(before, "hickUniqueAn")
    after_carets = carets(after, "hickUniqueAnswer")
    primary = 1 if grouped else 0
    if grouped:
        assert driver.act("editor-selections", path=path, ranges=before_carets, primary=primary) == {
            "ranges": before_carets, "primary": primary,
        }
    assert not any("answers.ts" in label for label in driver.act("tabs"))
    deadline = time.monotonic() + 20
    while True:
        choices = driver.act("completion-open", path=path, offset=before_carets[primary]["to"], keepSelection=grouped)
        if any("hickUniqueAnswer" in choice and "./answers" in choice for choice in choices):
            break
        assert time.monotonic() < deadline, choices
    assert driver.act("search-source", path=path) == before
    notice = driver.act("completion-choose", title="hickUniqueAnswer")
    assert "Completion applied" in notice, notice
    assert driver.act("search-source", path=path) == after
    assert driver.act("editor-selections", path=path) == {"ranges": after_carets, "primary": primary}
    driver.act("undo", path=path)
    assert driver.act("search-source", path=path) == before
    assert driver.act("editor-selections", path=path) == {"ranges": before_carets, "primary": primary}
    driver.act("redo", path=path)
    assert driver.act("search-source", path=path) == after
    assert driver.act("editor-selections", path=path) == {"ranges": after_carets, "primary": primary}
    assert driver.act("editor-is-retained", path=path)
    assert driver.act("editor-size", path=path) == size
    assert not any("answers.ts" in label for label in driver.act("tabs"))
    for name, source in saved.items():
        assert (root / name).read_bytes() == source.encode()
    print("PASS: installed TypeScript provider resolved one shared import, with one Undo/Redo and unchanged saved bytes")


def grouped_snippet_acceptance(driver, root, path, source, size, events):
    original = {"ranges": [{"from": 3, "to": 3}, {"from": 13, "to": 13}], "primary": 1}
    driver.act("editor-selections", path=path, **original)
    deadline = time.monotonic() + 20
    while True:
        choices = driver.act("completion-open", path=path, keepSelection=True)
        if any("python completion" in choice for choice in choices):
            break
        assert time.monotonic() < deadline, choices
    notice = driver.act("completion-choose", title="python completion")
    assert "Completion applied" in notice, notice
    after = "1:😀 + 1:😀 = 1\n2:😀 + 2:😀 = 2\n# imported\n"
    fields = {"ranges": [{"from": 0, "to": 4}, {"from": 16, "to": 20}], "primary": 1}
    assert driver.act("source-text-state", path=path)["source"] == after
    assert driver.act("editor-selections", path=path) == fields
    driver.act("undo", path=path)
    assert driver.act("source-text-state", path=path)["source"] == source
    assert driver.act("editor-selections", path=path) == original
    driver.act("redo", path=path)
    assert driver.act("source-text-state", path=path)["source"] == after
    assert driver.act("editor-selections", path=path) == fields
    driver.act("type-source-selection", path=path, text="linked")
    assert driver.act("source-text-state", path=path)["source"] == "linked + linked = 1\nlinked + linked = 2\n# imported\n"
    assert driver.act("editor-is-retained", path=path)
    assert driver.act("editor-size", path=path) == size
    assert (root / path).read_bytes() == source.encode()
    observed = [json.loads(line) for line in events.read_text().splitlines()]
    assert sum("completion_resolved" in row for row in observed) == 2, observed
    print("PASS: grouped snippets retained both ordinal fields and mirrors through one Undo/Redo with one shared import")


def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-source-completion-") as temporary:
        folder = Path(temporary)
        root = folder / "workspace"
        root.mkdir()
        grouped = "--typescript-group" in sys.argv
        snippet_group = "--snippet-group" in sys.argv
        typescript = grouped or "--typescript" in sys.argv
        if typescript:
            path = TYPESCRIPT_PATH
            saved, servers = typescript_fixture(root, grouped)
        else:
            nested = "--nested" in sys.argv
            snippet = "--snippet" in sys.argv or nested
            path = "grouped_snippet.py" if snippet_group else "nested.py" if nested else "snippet.py" if snippet else "a.py"
            source = "value = 1\nvalue = 2\n" if snippet_group else "value = 1\n"
            (root / path).write_text(source)
            events = folder / "events.jsonl"
            gate = folder / "release"
            command = [sys.executable, str(ROOT / "scripts/controlled-action-provider.py"), "python", str(events), str(gate)]
            servers = {"python": {"command": command}}
        (root / ".hick-lsp.json").write_text(json.dumps({"servers": servers}))
        environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path);deadline=time.monotonic()+20
            if typescript:
                typescript_acceptance(driver, root, saved, size, grouped)
                return
            if snippet_group:
                grouped_snippet_acceptance(driver, root, path, source, size, events)
                return
            while True:
                options=driver.act("completion-open",path=path,offset=3)
                if any("python completion" in value for value in options):break
                assert time.monotonic()<deadline,options;time.sleep(.1)
            assert driver.act("editor-size",path=path)==size;assert driver.act("search-source",path=path)==source
            notice=driver.act("completion-choose",title="python completion");assert "Completion applied" in notice,notice
            after=("value + value = 1\n# imported\n" if nested else "😀 value + 😀 value = 1\n# imported\n" if snippet else "python_completed = 1\n# imported\n");caret=0 if snippet else 16;assert driver.act("search-source",path=path)==after,{"expected":after,"actual":driver.act("search-source",path=path)};assert driver.act("caret-offset",path=path)==caret,{"caret":driver.act("caret-offset",path=path)}
            if snippet:assert driver.act("selected-source-text",path=path)==("value" if nested else "😀 value")
            assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            driver.act("undo",path=path);assert driver.act("search-source",path=path)==source
            driver.act("redo",path=path);assert driver.act("search-source",path=path)==after,{"redo_source":driver.act("search-source",path=path)};assert driver.act("caret-offset",path=path)==caret,{"redo_caret":driver.act("caret-offset",path=path),"selected":driver.act("selected-source-text",path=path)};assert (root/path).read_text()==source
            initial_after=after
            if nested:
                options=driver.act("completion-open",path=path,offset=3);assert any("python completion" in value for value in options),options
                notice=driver.act("completion-choose",title="python completion");assert "Completion applied" in notice,notice
                after="value + value + value + value = 1\n# imported\n# imported\n";assert driver.act("search-source",path=path)==after;assert driver.act("selected-source-text",path=path)=="value"
                driver.act("undo",path=path);assert driver.act("search-source",path=path)==initial_after
                driver.act("redo",path=path);assert driver.act("search-source",path=path)==after;assert driver.act("selected-source-text",path=path)=="value"
            if snippet:
                assert driver.act("selected-source-text",path=path)==("value" if nested else "😀 value")
                driver.act("type-source-selection",path=path,text="renamed")
                typed="renamed + renamed + renamed + renamed = 1\n# imported\n# imported\n" if nested else "renamed + renamed = 1\n# imported\n";assert driver.act("search-source",path=path)==typed;assert driver.act("caret-offset",path=path)==7
                driver.act("undo",path=path);assert driver.act("search-source",path=path)==after
                driver.act("redo",path=path);assert driver.act("search-source",path=path)==typed;assert driver.act("caret-offset",path=path)==7
                driver.act("template-next",path=path);assert driver.act("caret-offset",path=path)==17
                if nested:driver.act("template-next",path=path);assert driver.act("caret-offset",path=path)==37
                assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
                driver.act("undo",path=path);assert driver.act("search-source",path=path)==after
            if nested:driver.act("undo",path=path);assert driver.act("search-source",path=path)==initial_after
            driver.act("undo",path=path);assert driver.act("search-source",path=path)==source
            options=driver.act("completion-open",path=path,offset=2);assert any("python completion" in value for value in options),options
            driver.act("completion-pick",title="python completion");deadline=time.monotonic()+3
            while True:
                observed=[json.loads(line) for line in events.read_text().splitlines()]
                if any(item.get("slow_completion") for item in observed):break
                assert time.monotonic()<deadline,observed;time.sleep(.01)
            driver.act("completion-cancel",path=path);gate.write_text("release");deadline=time.monotonic()+3
            while True:
                observed=[json.loads(line) for line in events.read_text().splitlines()]
                if any("completion_released" in item for item in observed):break
                assert time.monotonic()<deadline,observed;time.sleep(.01)
            driver.act("completion-drain");assert driver.act("search-source",path=path)==source;assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            observed=[json.loads(line) for line in events.read_text().splitlines()];assert [item["completion_resolved"] for item in observed if "completion_resolved" in item]==(["python"]*3 if nested else ["python","python"]),observed
            print("PASS: native completion popup, actual lazy resolution, primary and import edits, "+("nested snippet mirrors, outer-scope return, imports and retained field history, " if nested else "expanded snippet, linked typing, menu traversal and retained field history, " if snippet else "precise caret, ")+"one undo/redo, Escape refusal of a held late reply, retained editor geometry and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
