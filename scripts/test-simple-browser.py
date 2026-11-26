# Contract: .hickory-source/frontends/contract.md
"""Focused real-engine browser acceptance for the independent frontend."""
import json
import os
from pathlib import Path
import queue
import re
import signal
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
from urllib.parse import urlencode

ROOT = Path(__file__).resolve().parents[1]

def start(command, *, cwd=ROOT, env=None):
    process = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
    return process

def origin(process):
    lines = queue.Queue()
    def read():
        for line in process.stdout:
            lines.put(line)
    threading.Thread(target=read, daemon=True).start()
    deadline = time.monotonic() + 45
    seen = []
    while time.monotonic() < deadline:
        try: line = lines.get(timeout=.2)
        except queue.Empty:
            if process.poll() is not None: break
            continue
        seen.append(line)
        match = re.search(r"http://127\.0\.0\.1:\d+", line)
        if match: return match.group(0)
    raise RuntimeError("No server origin: " + "".join(seen)[-1500:])

def stop(process):
    if process.poll() is None:
        os.killpg(process.pid, signal.SIGTERM)
        try: process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL); process.wait(timeout=5)

def eventually(read, expected):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        actual = read()
        if actual == expected: return
        time.sleep(.05)
    raise AssertionError((actual, expected))

class BrowserDriver:
    def __init__(self, url, *, navigate=True):
        self.url = url
        self.process = start(["node", "scripts/simple-browser-driver.mjs"])
        try: self.act("launch", url=url if navigate else None)
        except Exception:
            stop(self.process); raise
    def act(self, action, **arguments):
        self.process.stdin.write(json.dumps({"action": action, **arguments}) + "\n"); self.process.stdin.flush()
        line = self.process.stdout.readline()
        response = json.loads(line)
        if "error" in response: raise AssertionError(response["error"])
        return response["value"]
    def close(self):
        try: self.act("quit")
        finally: stop(self.process)

def welcome_acceptance(driver):
    driver.act("workspace-welcome", url=driver.url)

def exclusive_new_window_acceptance(driver, folder):
    path = "notes.txt"
    driver.act("open", path=path)
    driver.act("append", path=path, text=" unsaved")
    driver.act("exclusive-new-window", path=path, draft="before unsaved")
    assert (folder / path).read_text() == "before"
    driver.act("open", path=path)
    assert driver.act("content", path=path) == "before"


def early_source_acceptance(driver, folder):
    path, sibling = "nested/README #é.md", "nested/peer.txt"
    text = (folder / path).read_text()
    selection = text.index("Readable paragraph 48")
    for fail in [False, True]:
        driver.act("startup-held", url=driver.url + "?" + urlencode({"path": path}),
                   gates={"owner": "**/api/files", "parser": "**/hick_lang_bg.wasm*"})
        eventually(lambda: driver.act("capability-held", name="owner"), True)
        eventually(lambda: driver.act("capability-held", name="parser"), True)
        def state(): return driver.act("startup-state", path=path, sibling=sibling, outside="notes.txt")
        early = state()
        assert early["readonly"] and not early["siblingVisible"], early
        driver.act("remember-editor", path=path)
        driver.act("source-text-select", path=path, anchor=selection, head=selection + 8)
        driver.act("source-text-type", path=path, text="denied")
        assert driver.act("source-text-state", path=path)["source"] == text
        before = driver.act("editor-position", path=path, y=1250)
        assert before["y"] > 0, before
        reply = driver.act("capability-release", name="owner", fail=fail)
        if fail:
            assert reply is None
            after = state()
            assert after["readonly"] and not after["siblingVisible"], after
            assert driver.act("editor-is-retained", path=path)
            assert driver.act("editor-position", path=path) == before
            driver.act("source-file-refresh")
        else:
            assert reply["status"] == 200, reply
        eventually(lambda: state()["readonly"], False)
        after = state()
        assert after["siblingVisible"] and not after["outsidePresent"], after
        assert driver.act("editor-is-retained", path=path)
        assert driver.act("editor-position", path=path) == before
        driver.act("source-text-type", path=path, text="draft ")
        assert driver.act("source-text-state", path=path)["source"] != text
        driver.act("undo", path=path)
        assert driver.act("source-text-state", path=path)["source"] == text
        assert (folder / path).read_text() == text
        driver.act("capability-release", name="parser")

def delayed_capabilities_acceptance(driver, folder):
    def launch(path, sibling, readonly=False, **gates):
        driver.act("startup-held", url=driver.url + "?" + urlencode({"path": path}), gates=gates)
        def startup_state(): return driver.act("startup-state", path=path, sibling=sibling, outside="notes.txt")
        def authority_ready():
            result = startup_state()
            return result["siblingVisible"] and result["readonly"] == readonly
        eventually(authority_ready, True)
        state = startup_state()
        assert state["siblingVisible"] and not state["outsidePresent"] and state["readonly"] == readonly, state
        assert sorted(state["tabs"]) == sorted(["Files", Path(path).name]), state
        driver.act("remember-editor", path=path)
        return state

    def held(name):
        eventually(lambda: driver.act("capability-held", name=name), True)

    def release(name, **options):
        reply = driver.act("capability-release", name=name, **options)
        assert reply is None if options.get("fail") else reply["status"] == 200, reply
        return reply

    def retain(path, before):
        assert driver.act("editor-is-retained", path=path)
        assert driver.act("source-text-state", path=path) == before

    path, sibling = "nested/README #é.md", "nested/peer.txt"
    source = (folder / path).read_text()
    for render_fails in (False, True):
        render_gate = {"stylesheet": "**/katex.min.css?inline*"} if render_fails else {"render": "**/api/docs/*/render"}
        launch(path, sibling, parser="**/hick_lang_bg.wasm*", **render_gate)
        held("parser")
        if not render_fails:
            held("render")
        assert not driver.act("capability-ui", path=path, kind="markdown")
        assert driver.act("source-text-state", path=path)["source"] == source
        start = source.index("Readable paragraph 48")
        driver.act("source-text-select", path=path, anchor=start, head=start)
        driver.act("source-text-type", path=path, text="draft ")
        driver.act("source-text-select", path=path, anchor=start + 6, head=start + 14)
        position = driver.act("editor-position", path=path, y=1260)
        visible = driver.act("visible-document-line", path=path)
        draft = driver.act("source-text-state", path=path)
        geometry = driver.act("editor-size", path=path)
        assert position["y"] > 0 and visible, (position, visible, geometry)
        assert draft["source"] == source[:start] + "draft " + source[start:]

        release("parser")
        eventually(lambda: driver.act("capability-ui", path=path, kind="markdown"), True)
        if render_fails:
            held("stylesheet")
            parsed_position = driver.act("editor-position", path=path)
            release("stylesheet", fail=True)
            eventually(lambda: "Inline rendering unavailable" in driver.act("workspace-status"), True)
            assert driver.act("capability-ui", path=path, kind="markdown")
            assert driver.act("editor-position", path=path) == parsed_position
        else:
            release("render")
        retain(path, draft)
        assert driver.act("visible-document-line", path=path).lstrip("# ") == visible.lstrip("# ")
        assert driver.act("editor-size", path=path) == geometry
        driver.act("undo", path=path)
        assert driver.act("source-text-state", path=path)["source"] == source
        assert (folder / path).read_text() == source

    path, sibling = "nested/test_delay.py", "nested/peer.txt"
    source = (folder / path).read_text()
    launch(path, sibling, discovery="**/api/tests/source-candidates")
    held("discovery")
    assert not driver.act("capability-ui", path=path, kind="test", name="test_before")
    start = source.index("before")
    driver.act("source-text-select", path=path, anchor=start, head=start + len("before"))
    driver.act("source-text-type", path=path, text="after")
    draft = driver.act("source-text-state", path=path)
    reply = release("discovery", read=True)
    assert any(row["name"] == "test_before" for row in reply["body"]["candidates"]), reply
    eventually(lambda: driver.act("capability-ui", path=path, kind="test", name="test_after"), True)
    assert not driver.act("capability-ui", path=path, kind="test", name="test_before")
    retain(path, draft)
    driver.act("undo", path=path)
    assert driver.act("source-text-state", path=path)["source"] == source
    assert (folder / path).read_text() == source

    path = "nested/generated.txt"
    source = (folder / path).read_text()
    launch(path, sibling, readonly=True, parser="**/hick_lang_bg.wasm*")
    held("parser")
    assert driver.act("source-text-state", path=path)["source"] == source
    release("parser", fail=True)
    driver.act("source-text-type", path=path, text="denied")
    assert driver.act("source-text-state", path=path)["source"] == source
    assert (folder / path).read_text() == source

    path = "nested/delay.csv"
    source = (folder / path).read_text()
    for fail in [False, True]:
        launch(path, sibling, grid="**/src/styles.css?inline*")
        held("grid")
        assert not driver.act("capability-ui", path=path, kind="csv")["available"]
        assert driver.act("source-text-state", path=path)["source"] == source
        start = source.index("product-55")
        driver.act("source-text-select", path=path, anchor=start, head=start)
        driver.act("source-text-type", path=path, text="draft-")
        driver.act("source-text-select", path=path, anchor=start + 6, head=start + 11)
        position = driver.act("editor-position", path=path, y=1100)
        draft = driver.act("source-text-state", path=path)
        assert position["y"] > 0, position
        release("grid", fail=fail)
        eventually(lambda: driver.act("capability-ui", path=path, kind="csv"), {
            "available": not fail,
            "reason": "Table grid unavailable. Keep editing source or reload to retry the grid." if fail else None,
        })
        retain(path, draft)
        assert driver.act("editor-position", path=path) == position
        driver.act("undo", path=path)
        assert driver.act("source-text-state", path=path)["source"] == source
        assert (folder / path).read_text() == source

def workspace_changes_acceptance(driver, folder):
    first, second, third = "renamea.py", "renameb.py", "renamec.py"
    originals = {path: (folder / path).read_text() for path in (first, second, third)}
    renamed = {path: source.replace("value", "renamed_value") for path, source in originals.items()}

    def assert_drafts(expected):
        for path, source in expected.items():
            driver.act("open", path=path)
            actual = driver.act("search-source", path=path)
            assert actual == source, (path, actual, source)
        for path, source in originals.items():
            assert (folder / path).read_bytes() == source.encode()

    def prepare_rename():
        driver.act("change-open")
        deadline = time.monotonic() + 20
        while True:
            result = driver.act("change-prepare", name="renamed_value")
            if "3 source files" in result and "Apply to source drafts" in result:
                return
            assert time.monotonic() < deadline, result
            time.sleep(.25)

    driver.act("open", path=first)
    driver.act("remember-editor", path=first)
    size = driver.act("editor-size", path=first)
    initially_open = driver.act("tabs")
    assert not any(path in label for path in (second, third) for label in initially_open)
    prepare_rename()
    reviewed_tabs = driver.act("tabs")
    assert not any(path in label for path in (second, third) for label in reviewed_tabs)
    assert driver.act("editor-is-retained", path=first)
    assert driver.act("change-state", path=first)["size"] == size
    for target in (second, third):
        driver.act("change-file", path=target)
        assert "renamed_value" in driver.act("change-state", path=first)["text"]
    for path, source in originals.items():
        assert (folder / path).read_bytes() == source.encode()

    # A target changed after review cannot lend its old authority to the batch.
    intervening = "# intervening draft\n"
    driver.act("open", path=second)
    driver.act("append", path=second, text=intervening)
    driver.act("change-apply")
    refused = driver.act("change-state", path=second)["text"]
    assert "changed" in refused and "Applied 3 source drafts" not in refused, refused
    driver.act("change-close")
    assert_drafts({**originals, second: originals[second] + intervening})
    driver.act("open", path=second)
    driver.act("undo", path=second)
    assert driver.act("search-source", path=second) == originals[second]

    driver.act("open", path=first)
    prepare_rename()
    driver.act("change-apply")
    assert "Applied 3 source drafts" in driver.act("change-state", path=first)["text"]
    driver.act("change-close")
    assert_drafts(renamed)
    for command in ("Undo", "Redo"):
        driver.act("change-open", command=f"{command} workspace change")
        assert f"Workspace {command.lower()} applied to 3 drafts" in driver.act("change-state", path=third)["text"]
        driver.act("change-close")
        assert_drafts(originals if command == "Undo" else renamed)

    # A later edit also protects the complete redo batch from coordinated Undo.
    later = "\n# later draft"
    driver.act("open", path=second)
    driver.act("append", path=second, text=later)
    later_draft = driver.act("search-source", path=second)
    assert later_draft in {renamed[second] + later, renamed[second].rstrip("\n") + later}
    driver.act("change-open", command="Undo workspace change")
    assert "Source changed" in driver.act("change-state", path=second)["text"]
    driver.act("change-close")
    assert_drafts({**renamed, second: later_draft})
    driver.act("open", path=first)
    assert driver.act("editor-is-retained", path=first)
    assert driver.act("editor-size", path=first) == size


FORMAT_OWNER = "format-owner.md"
FORMAT_OUTPUT = "format-output.cpp"
FORMAT_ENTRY = (
    '<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n'
    f'<hick:project sources="{FORMAT_OWNER}"/>\n'
    '</hick:doc>\n'
)
FORMAT_BEFORE = "int square( int value ){return value*value;}\nint main(){if(true){return square(3);}return 0;}\n"
FORMAT_AFTER = (
    "int square(int value) { return value * value; }\n"
    "int main() {\n  if (true) {\n    return square(3);\n  }\n  return 0;\n}\n"
)
FORMAT_PROSE = "Leave this line alone."
FORMAT_SOURCE = (
    '<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n'
    f'\n# Owned prose 🧪\n\n{FORMAT_PROSE}\n\n'
    f'<hick:file id="source" path="{FORMAT_OUTPUT}">\n'
    f'{FORMAT_BEFORE}</hick:file>\n\nRetain this paragraph too.\n\n</hick:doc>\n'
).replace("\n", "\r\n")


def managed_formatting_fixture(folder):
    clangd = shutil.which("clangd")
    if clangd is None:
        raise RuntimeError("The managed formatting witness requires clangd on PATH.")
    subprocess.run(["git", "init", "-q", str(folder)], check=True, timeout=10)
    (folder / "project.md").write_bytes(FORMAT_ENTRY.encode())
    (folder / FORMAT_OWNER).write_bytes(FORMAT_SOURCE.encode())
    command = [clangd, "--background-index=false", "--fallback-style=LLVM", "--log=error"]
    (folder / ".hick-lsp.json").write_text(json.dumps({"servers": {"cpp": {"command": command}}}))


def managed_formatting_acceptance(driver, folder):
    def source_state():
        return driver.act("source-text-state", path=FORMAT_OWNER)

    driver.act("open", path=FORMAT_OWNER)
    assert source_state()["source"] == FORMAT_SOURCE
    prefix = FORMAT_SOURCE.split(FORMAT_PROSE, 1)[0]
    offset = len(prefix.encode("utf-16-le")) // 2
    driver.act("source-text-select", path=FORMAT_OWNER, anchor=offset, head=offset)
    driver.act("source-text-type", path=FORMAT_OWNER, text="Draft: ")
    draft = FORMAT_SOURCE.replace(FORMAT_PROSE, "Draft: " + FORMAT_PROSE, 1)
    formatted = draft.replace(FORMAT_BEFORE.replace("\n", "\r\n"), FORMAT_AFTER.replace("\n", "\r\n"), 1)
    draft_state = source_state()
    assert draft_state["source"] == draft
    driver.act("remember-editor", path=FORMAT_OWNER)
    size = driver.act("editor-size", path=FORMAT_OWNER)

    driver.act("change-open", command="Preview formatting")
    deadline = time.monotonic() + 35
    while True:
        preview = driver.act("change-prepare", command="Prepare formatting")
        if "Apply to source drafts" in preview:
            break
        assert time.monotonic() < deadline, preview
        time.sleep(0.3)
    assert FORMAT_OWNER in preview and "return square(3);" in preview
    assert source_state() == draft_state
    driver.act("change-apply")
    apply_notice = driver.act("change-state", path=FORMAT_OWNER)["text"]
    assert "Applied 1 source drafts" in apply_notice, apply_notice
    driver.act("change-close")
    assert source_state()["source"] == formatted

    for command, expected in (("Undo", draft), ("Redo", formatted)):
        driver.act("change-open", command=f"{command} workspace change")
        history_notice = driver.act("change-state", path=FORMAT_OWNER)["text"]
        assert f"Workspace {command.lower()} applied to 1 drafts" in history_notice, history_notice
        driver.act("change-close")
        assert source_state()["source"] == expected

    assert driver.act("editor-is-retained", path=FORMAT_OWNER)
    assert driver.act("editor-size", path=FORMAT_OWNER) == size
    assert (folder / FORMAT_OWNER).read_bytes() == FORMAT_SOURCE.encode()
    assert (folder / "project.md").read_bytes() == FORMAT_ENTRY.encode()
    assert not (folder / FORMAT_OUTPUT).exists()


def workspace_search_acceptance(driver, folder):
    # Contract: .hickory-source/frontends/contract.md. Real exhaustive API and retained editor.
    path = "search.txt"; original = (folder / path).read_text()
    driver.act("open", path=path); driver.act("remember-editor", path=path)
    size = driver.act("editor-size", path=path)
    result = driver.act("workspace-search-query", path=path, query=r"name=\d+", regex=True, matchCase=True, wholeWord=True)
    assert "2 occurrences" in result["text"] and result["size"] == size, result
    driver.act("workspace-search-hit", label="search.txt:1 · 😀 name=12")
    eventually(lambda: driver.act("caret-offset", path=path), 3)
    assert driver.act("editor-is-retained", path=path), "Workspace search replaced the source editor"
    driver.act("append", path=path, text="\nunsaved draft")
    result = driver.act("workspace-search-query", path=path, query="name=12")
    assert "1 occurrences" in result["text"] and result["size"] == size, result
    driver.act("workspace-search-hit", label="search.txt:1 · 😀 name=12")
    eventually(lambda: "Search source changed" in driver.act("workspace-search-notice"), True)
    assert (folder / path).read_text() == original, "Workspace search saved a draft"
    assert "unsaved draft" in driver.act("content", path=path)
    driver.act("workspace-search-close"); driver.act("undo", path=path)
    eventually(lambda: driver.act("search-source", path=path), original)


def search_everywhere_acceptance(driver,folder):
    # Contract: .hickory-source/frontends/contract.md. Real provider, menus and keyboard through one target model.
    driver.act("everywhere-open",via="menu");driver.act("everywhere-query",kind="action",query="Rename symbol")
    initial=driver.act("everywhere-state");assert initial["disabled"]==1 and "Open a source file first" in initial["text"],initial
    driver.act("everywhere-close")
    path="cache.py";original=(folder/path).read_text();symbols=(folder/"symbols.py").read_text()
    driver.act("open",path=path);driver.act("append",path=path,text="\n# retained search draft")
    identity=driver.act("mark-editor",path=path);rect=driver.act("editor-rect",path=path)
    driver.act("everywhere-open",via="shortcut");driver.act("everywhere-query",kind="type",query="wdgt")
    assert driver.act("everywhere-state",path=path)["options"]==[]
    driver.act("everywhere-start");deadline=time.monotonic()+120
    while time.monotonic()<deadline:
        report=driver.act("everywhere-state",path=path)
        assert report["rect"]==rect and report["identity"]==identity and "# retained search draft" in report["source"],report
        if report["finished"]:break
        time.sleep(.1)
    assert report["finished"] and len(report["options"])==1 and "Widget" in report["options"][0] and "symbols.py" in report["options"][0],report
    driver.act("everywhere-choose",keyboard=True)
    eventually(lambda:driver.act("caret-offset",path="symbols.py"),6)
    assert driver.act("editor-rect",path="symbols.py")==rect
    driver.act("everywhere-open",via="menu");driver.act("everywhere-query",kind="member",query="render")
    report=driver.act("everywhere-state",path="symbols.py");assert any("render" in item and "symbols.py" in item for item in report["options"]),report
    driver.act("everywhere-query",kind="action",query="Problems");driver.act("everywhere-choose")
    eventually(lambda:"background.py" in driver.act("problems-state",path="symbols.py")["text"],True);driver.act("problems-close")
    driver.act("append",path="symbols.py",text="\n# changed source")
    driver.act("everywhere-open",via="shortcut");driver.act("everywhere-query",kind="type",query="Widget")
    assert driver.act("everywhere-state",path="symbols.py")["options"]==[],"Changed source kept an old type target"
    driver.act("everywhere-query",kind="file",query="cache.py");driver.act("everywhere-choose",keyboard=True)
    driver.act("everywhere-open",via="menu");driver.act("everywhere-query",kind="file",query="cache.py")
    final=driver.act("everywhere-state",path=path)
    assert final["identity"]==identity and final["rect"]==rect and "# retained search draft" in final["source"],final
    driver.act("everywhere-close")
    assert (folder/path).read_text()==original and (folder/"symbols.py").read_text()==symbols,"Search saved source drafts"


def workspace_analysis_acceptance(driver, folder):
    # Contract: .hickory-source/frontends/contract.md. Unopened sources join the retained report index.
    path="cache.py"; original=(folder/path).read_text()
    driver.act("open",path=path);driver.act("append",path=path,text="\n# retained draft")
    size=driver.act("editor-size",path=path)
    driver.act("analysis-open",via="menu");driver.act("analysis-start")
    deadline=time.monotonic()+120
    while time.monotonic()<deadline:
        report=driver.act("analysis-state",path=path)
        assert report["size"]==size,"Analysis resized code"
        assert "# retained draft" in report["source"],"Analysis replaced the active draft"
        if report["finished"]: break
        time.sleep(.1)
    assert report["finished"],report
    assert "background.py" in report["text"] and "unclassified excluded" in report["text"],report
    assert "unavailable" not in report["text"],report
    driver.act("analysis-close");driver.act("problems-open",via="status")
    eventually(lambda:"background.py" in driver.act("problems-state",path=path)["text"],True)
    driver.act("problems-close");driver.act("analysis-open",via="status")
    assert driver.act("analysis-state",path=path)["size"]==size
    driver.act("analysis-close")
    assert (folder/path).read_text()==original,"Analysis saved a draft"
    assert (folder/"background.py").read_text()=='def background():\n    amount: int = "wrong"\n    return amount\n'


def problems_acceptance(driver, folder):
    # Contract: .hickory-source/frontends/contract.md. Real provider messages and UI geometry.
    path = "problem.py"; original = (folder / path).read_text()
    driver.act("open", path=path)
    size = driver.act("editor-size", path=path)
    eventually(lambda: driver.act("problem-count") > 0, True)
    driver.act("problems-open", via="status")
    eventually(lambda: driver.act("problems-state", path=path)["current"] > 0, True)
    result = driver.act("problems-state", path=path)
    assert path in result["text"], result
    assert result["size"] == size, "Problems resized code"
    driver.act("problems-close")
    driver.act("problem-next")
    eventually(lambda: driver.act("caret-offset", path=path) > 0, True)
    driver.act("append", path=path, text="\n# retained draft")
    driver.act("problems-open", via="menu")
    result = driver.act("problems-state", path=path)
    assert path in result["text"] and result["size"] == size, result
    assert (folder / path).read_text() == original, "Problems saved source"
    driver.act("problems-close"); driver.act("undo", path=path)


def semantic_navigation_acceptance(driver, folder, held_origin=False):
    # Contract: .hickory-source/frontends/contract.md. Real provider and visible menu/results.
    path = "navigation.py"
    original = {name: (folder / name).read_text() for name in [path, "semantic_target.py"]}
    driver.act("open", path=path)
    driver.act("remember-editor", path=path)
    size = driver.act("editor-size", path=path)
    result = driver.act("semantic-query", path=path, query="Go to definition", offset=41)
    # A newly installed provider may still be indexing imports. Retry this read
    # through the same menu; never manufacture a location or retry a mutation.
    deadline = time.monotonic() + 15
    while "semantic_target.py:1:13" not in result["text"] and time.monotonic() < deadline:
        driver.act("semantic-close")
        time.sleep(.25)
        result = driver.act("semantic-query", path=path, query="Go to definition", offset=41)
    assert result["size"] == size, "Semantic results resized the editor"
    assert "semantic_target.py:1:13" in result["text"], result
    if held_origin:
        held = driver.act("semantic-retire-origin", path=path, target="semantic_target.py", label="semantic_target.py:1:13")
        assert held == {"status": 200, "targetTabs": 0, "back": False, "caret": 42, "active": True, "retained": True}, held
        result = driver.act("semantic-query", path=path, query="Go to definition", offset=41)
        assert "semantic_target.py:1:13" in result["text"], result
    driver.act("semantic-open", path="semantic_target.py", label="semantic_target.py:1:13")
    assert "value = 42" in driver.act("content", path="semantic_target.py")
    eventually(lambda: driver.act("caret-offset", path="semantic_target.py"), 12)
    driver.act("navigate-history", direction="back", path=path)
    eventually(lambda: driver.act("caret-offset", path=path), 41)
    assert driver.act("editor-is-retained", path=path), "History replaced the retained source editor"
    if held_origin:
        assert original == {name: (folder / name).read_text() for name in original}
        return
    driver.act("navigate-history", direction="forward", path="semantic_target.py")
    eventually(lambda: driver.act("caret-offset", path="semantic_target.py"), 12)
    result = driver.act("semantic-query", path="semantic_target.py", query="Find usages", offset=13)
    deadline = time.monotonic() + 15
    while "navigation.py:" not in result["text"] and time.monotonic() < deadline:
        driver.act("semantic-close"); time.sleep(.25)
        result = driver.act("semantic-query", path="semantic_target.py", query="Find usages", offset=13)
    assert "navigation.py:" in result["text"], result
    assert result["size"] == size, "Usages resized the editor"
    driver.act("semantic-close")
    outline = driver.act("semantic-query", path="semantic_target.py", query="File outline", offset=13)
    assert "value · semantic_target.py:1:13" in outline["text"], outline
    assert outline["size"] == size
    driver.act("semantic-close")
    symbols = driver.act("semantic-query", path="semantic_target.py", query="Search workspace symbols", offset=13, term="calculate")
    assert "calculate" in symbols["text"] and "navigation.py:" in symbols["text"], symbols
    driver.act("semantic-close")
    driver.act("open", path="navigation.py")
    offset = original[path].index("calculate") + 2
    for query in ["Find callers", "Find callees"]:
        calls = driver.act("semantic-query", path="navigation.py", query=query, offset=offset)
        assert "navigation.py:" in calls["text"], calls
        assert calls["size"] == size
        driver.act("semantic-close")
    for name, source in original.items(): assert (folder / name).read_text() == source


CALL_HIERARCHY_SOURCE = {
    "origin.py": "from dependency import owned_leaf\n\ndef owned_middle() -> int:\n    return owned_leaf()\n",
    "caller.py": "from origin import owned_middle\n\n# Owned caller 🧪.\ndef owned_foreign_caller() -> int:\n    return owned_middle()\n",
    "dependency.py": "# Owned dependency 🧪.\n\ndef owned_leaf() -> int:\n    return 42\n",
    "stale_caller.py": "from origin import owned_middle\n\ndef owned_stale_caller() -> int:\n    return owned_middle()\n",
}


def call_hierarchy_acceptance(driver, folder):
    path = "origin.py"
    source = CALL_HIERARCHY_SOURCE[path]
    offset = source.index("owned_middle") + 2
    change = "# unsaved caller inquiry\n"
    driver.act("open", path=path)
    driver.act("source-tests-edit", path=path, text=change)
    assert driver.act("search-source", path=path) == source + change
    driver.act("remember-editor", path=path)
    size = driver.act("editor-size", path=path)

    def read(query, label):
        deadline = time.monotonic() + 25
        while True:
            result = driver.act("semantic-query", path=path, offset=offset, query=query)
            if label in result["text"]:
                assert result["size"] == size, result
                return
            driver.act("semantic-close")
            assert time.monotonic() < deadline, result
            time.sleep(.2)

    for query, target, symbol, position in (
        ("Find callers", "caller.py", "owned_foreign_caller", "4:5"),
        ("Find callees", "dependency.py", "owned_leaf", "3:5"),
    ):
        label = f"{symbol} · {target}:{position}"
        read(query, label)
        driver.act("semantic-open", path=target, label=label)
        before = CALL_HIERARCHY_SOURCE[target].split(symbol, 1)[0]
        eventually(lambda: driver.act("caret-offset", path=target), len(before.encode("utf-16-le")) // 2)
        driver.act("navigate-history", direction="back", path=path)
        eventually(lambda: driver.act("caret-offset", path=path), offset)
        assert driver.act("editor-is-retained", path=path)

    target = "stale_caller.py"
    label = "owned_stale_caller · stale_caller.py:3:5"
    read("Find callers", label)
    renamed = CALL_HIERARCHY_SOURCE[target].replace("owned_stale_caller()", "owned_stale_caller_new()")
    (folder / target).write_text(renamed)
    assert driver.act("semantic-refuse-name", path=path, target=target, label=label) == {
        "sourceVisible": True, "targetVisible": False,
    }
    assert driver.act("editor-is-retained", path=path)
    assert driver.act("editor-size", path=path) == size
    assert driver.act("search-source", path=path) == source + change
    for name, expected in {**CALL_HIERARCHY_SOURCE, target: renamed}.items():
        assert (folder / name).read_bytes() == expected.encode()


def language_cache_acceptance(driver, folder):
    # Requires the real Python language server; never substitutes canned tokens.
    path = "cache.py"
    original = (folder / path).read_text()
    driver.act("open", path=path)
    eventually(lambda: "counter" in driver.act("semantic-colors", path=path), True)
    for _ in range(3):
        colors = driver.act("append-with-cached-colors", path=path, text="\n# pending")
        assert "counter" in colors, "An edit cleared cached semantic colors"
    assert (folder / path).read_text() == original, "Language refresh must not save edits"
    driver.act("undo", path=path)
    assert "counter" in driver.act("semantic-colors", path=path)


def debug_thread_acceptance(driver, folder):
    path="%2F λ?#.py" if "--debug-literal" in sys.argv else "thread_debug.py";source=(folder/path).read_text();driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
    driver.act("debug-breakpoint",line=14);driver.act("debug-start",path=path)
    try: eventually(lambda:any("paused" in state for state in driver.act("debug-state")),True)
    except AssertionError as error: raise AssertionError(driver.act("debug-diagnostics")) from error
    reading=driver.act("debug-threads");workers=[item for item in reading["threads"] if "Hickory worker" in item["name"]];assert len(workers)==1,reading
    original=reading["selected"];driver.act("debug-thread-select",id=workers[0]["id"]);position=driver.act("debug-frame-select",name="worker");assert position["line"]>0,position
    expected=sum(len(line) for line in source.splitlines(keepends=True)[:position["line"]-1]);eventually(lambda:driver.act("construction-source",path=path,selection=True)["head"],expected)
    eventually(lambda:"worker_value = 'worker' · str" in driver.act("debug-locals"),True)
    assert "WORKER" in driver.act("debug-watch",expression="worker_value.upper()"),"Worker expression was not evaluated in its selected frame"
    expression="{'nested': {'answer': 41}}";driver.act("debug-watch",expression=expression);assert "nested" in driver.act("debug-expand",name=expression);expanded=driver.act("debug-expand",name="nested");assert "answer" in expanded and "41" in expanded,expanded
    wire=driver.act("debug-wire");children=[item for item in wire if item.get("op")=="children"];assert len(children)>=2 and all(all(item.get(field) is not None for field in ["stop","thread","frame","request_id"]) for item in children),children

    assert driver.act("editor-is-retained",path=path) and driver.act("editor-size",path=path)==size and (folder/path).read_text()==source
    driver.act("debug-thread-select",id=original);position=driver.act("debug-frame-select",name="<module>");assert position["line"]>0,position
    expected=sum(len(line) for line in source.splitlines(keepends=True)[:position["line"]-1]);eventually(lambda:driver.act("construction-source",path=path,selection=True)["head"],expected)
    try: eventually(lambda:"main_value = 'main' · str" in driver.act("debug-locals"),True)
    except AssertionError as error: raise AssertionError({"values":driver.act("debug-values"),"surface":driver.act("debug-diagnostics")}) from error
    assert "MAIN" in driver.act("debug-watch",expression="main_value.upper()"),"Main expression was not evaluated after thread return"
    assert driver.act("debug-move",kind="run_to",line=15)["supported"],"Engine-composed Run to line unavailable";assert driver.act("debug-frame-select",name="<module>")["line"]==15
    jump=driver.act("debug-move",kind="jump",line=14)
    if jump["supported"]:assert driver.act("debug-frame-select",name="<module>")["line"]==14

    driver.act("debug-stop");eventually(lambda:not any("paused" in state for state in driver.act("debug-state")),True)
    assert driver.act("editor-is-retained",path=path) and driver.act("editor-size",path=path)==size and (folder/path).read_text()==source

def model_diagram_acceptance(driver, folder):
    """Read the repository script and observe an edge update without a refresh click."""
    path = "workbook-demo/architecture.md"
    driver.act("browser-surface", url=driver.url + "?" + urlencode({"path": path}))
    driver.act("architecture-preview", path=path, edge_count=1, inspect_source=True)
    receipt = folder / "workbook-demo/shop/receipt.ts"
    receipt.write_text('export const receiptTotalCents = (quantity: number) => 800 * quantity;\n')
    driver.act("architecture-preview", path=path, edge_count=0)
    assert driver.act("architecture-link", path="workbook-demo/shop/receipt.ts") == 0


def workbook_acceptance(driver, folder):
    """The same saved sample exercises links, context breakpoints and an ordinary fence."""
    demo = folder / "workbook-demo"
    path = "workbook-demo/workbook.md"
    source = (demo / "workbook.md").read_text()
    rows = source.splitlines()
    def reveal(line):
        prefix = "".join(source.splitlines(keepends=True)[:line - 1])
        offset = len(prefix.encode("utf-16-le")) // 2
        driver.act("source-text-select", path=path, anchor=offset, head=offset)
        return offset

    driver.act("browser-surface", url=driver.url + "?" + urlencode({"path": "workbook-demo/project.md"}))
    driver.act("workbook-link", path="workbook-demo/project.md")
    assert driver.act("construction-source", path=path) == source
    context = next(i for i, row in enumerate(rows, 1) if row.startswith("     total ="))
    reveal(context)
    driver.act("debug-breakpoint", line=context)
    patch_header = next(i for i, row in enumerate(rows, 1) if row.startswith("+++ b/generated/coupon.py"))
    driver.act("workbook-debug", path="generated/coupon.py", at=reveal(patch_header))
    eventually(lambda: any("paused" in state for state in driver.act("debug-state")), True)
    driver.act("debug-continue")
    driver.act("workbook-output", text="Coupon receipt: 1880 cents")
    eventually(lambda: all("paused" not in state and "running" not in state for state in driver.act("debug-state")), True)
    literal = next(i for i, row in enumerate(rows, 1) if row.startswith('print("<script>'))
    reveal(literal)
    driver.act("debug-breakpoint", line=literal)
    fence = next(i for i in range(literal - 1, 0, -1) if rows[i - 1] == "```python")
    driver.act("workbook-debug", path="python fence", at=reveal(fence))
    eventually(lambda: any("paused" in state for state in driver.act("debug-state")), True)
    driver.act("debug-continue")
    driver.act("workbook-output", text="literal text</script>")
    driver.act("workbook-output", text="This line is stderr.")
    eventually(lambda: all("paused" not in state and "running" not in state for state in driver.act("debug-state")), True)
    driver.act("workbook-run", path=path, at=reveal(fence))
    driver.act("workbook-output", text="This line is stderr.")
    assert (demo / "workbook.md").read_text() == source
    assert not (demo / "generated").exists(), "Debug should leave generated paths in its scratch tree"


def debug_acceptance(driver):
    driver.act("open", path="debug.py")
    driver.act("debug-breakpoint", line=2)
    driver.act("debug-start", path="debug.py")
    eventually(lambda: any("paused" in state for state in driver.act("debug-state")), True)
    driver.act("split"); driver.act("move")
    assert any("paused" in state for state in driver.act("debug-state"))
    driver.act("debug-step")
    eventually(lambda: any("value =" in value for value in driver.act("debug-values")), True)
    driver.act("debug-stop")
    eventually(lambda: not any("paused" in state for state in driver.act("debug-state")), True)

def smart_enter_acceptance(driver, folder):
    # Contract: .hickory-source/frontends/contract.md. Visible settings, real keys and disk.
    path = "enter.py"; source = (folder / path).read_text()
    driver.act("open", path=path)
    size = driver.act("editor-size", path=path)
    for mode, indent in [("smart", "        "), ("copy", "    "), ("none", "")]:
        driver.act("enter-setting", mode=mode)
        assert driver.act("editor-size", path=path) == size
        driver.act("enter-at-end", path=path)
        driver.act("save")
        eventually(lambda: (folder / path).read_text(), source + "\n" + indent)
        driver.act("undo", path=path)
        eventually(lambda: driver.act("content", path=path), source)


def editor_navigation_acceptance(driver, folder):
    # Contract: .hickory-source/frontends/contract.md. Real keys, menus and reload.
    path = "words.txt"
    original = (folder / path).read_text()
    driver.act("open", path=path)
    size = driver.act("editor-size", path=path)
    driver.act("editor-setting", modifier="Alt/Option", mode="pieces")
    driver.act("editor-setting", modifier="Ctrl", mode="word")
    assert driver.act("editor-size", path=path) == size
    assert driver.act("editor-word-selection", path=path, keys=["Alt+Shift+ArrowRight"]) == "THIS"
    assert driver.act("editor-word-selection", path=path, keys=["Alt+Shift+ArrowRight", "Alt+Shift+ArrowRight"]) == "THIS_IS"
    assert driver.act("editor-word-selection", path=path, keys=["Alt+Shift+ArrowRight", "Alt+Shift+ArrowRight", "Alt+Shift+ArrowLeft"]) == "THIS_"
    assert driver.act("editor-word-selection", path=path, keys=["Control+Shift+ArrowRight"]) == "THIS_IS_ONE_WORD"
    assert driver.act("reload-editor-settings") == {"control": "word", "alt": "pieces"}
    assert (folder / path).read_text() == original


def file_editing_acceptance(driver, folder):
    # Contract: .hickory-source/frontends/contract.md. Real keyboard and disk I/O.
    commands = driver.act("file-menu-items")
    for command in ["Open File", "Open Directory", "New Tab", "New Window", "Save", "Save As", "Save All", "Refresh"]:
        assert command in commands
    for path, unit in [("four.py", "    "), ("two.py", "  "), ("tabs.py", "\t")]:
        original = (folder / path).read_text()
        driver.act("file-command", command="Open File", path=path)
        driver.act("indent-tabs", path=path)
        eventually(lambda: driver.act("content", path=path), original + "\n" + unit * 2)
    driver.act("file-command", command="Save All")
    for path, unit in [("four.py", "    "), ("two.py", "  "), ("tabs.py", "\t")]:
        eventually(lambda: (folder / path).read_text().split("\n")[-1], unit * 2)
    driver.act("file-command", command="Save As", path="copied.py")
    eventually(lambda: (folder / "copied.py").exists(), True)
    assert (folder / "copied.py").read_text() == (folder / "tabs.py").read_text()


def search_acceptance(driver, folder):
    path = "search.txt"
    source = (folder / path).read_text()
    driver.act("open", path=path)
    driver.act("append", path=path, text="unsaved motion witness")
    for reduced in (False, True):
        state = driver.act("motion-search", path=path, reduced=reduced)
        assert state == {"duration": "0s" if reduced else "0.12s", "sameEditor": True, "stable": True}, state
    driver.act("undo", path=path)
    eventually(lambda: driver.act("search-source", path=path), source)
    assert (folder / path).read_text() == source, "Opening search must not save the draft"
    size = driver.act("editor-size", path=path)
    driver.act("search-open")
    assert driver.act("editor-size", path=path) == size
    driver.act("search-query", query=r"name=(\d+)", replacement="$1", regex=True)
    driver.act("search-replace-all")
    driver.act("save")
    eventually(lambda: (folder / path).read_text(), "😀 12\n34\n")
    driver.act("search-close")
    driver.act("undo", path=path)
    eventually(lambda: driver.act("search-source", path=path), source)

def folding_acceptance(driver, folder):
    path = "fold.txt"
    source = (folder / path).read_text()
    driver.act("open", path=path)
    driver.act("append", path=path, text="tail")
    driver.act("fold", path=path, line=1)
    eventually(lambda: driver.act("fold-count", path=path), 1)
    driver.act("save")
    eventually(lambda: (folder / path).read_text(), source + "tail")
    driver.act("split"); driver.act("move")
    eventually(lambda: driver.act("fold-count", path=path), 1)
    driver.act("unfold", path=path, line=1)
    eventually(lambda: driver.act("fold-count", path=path), 0)
    assert driver.act("content", path=path) == source + "tail"
    driver.act("undo", path=path)
    eventually(lambda: driver.act("content", path=path), source)

def spreadsheet_acceptance(driver, folder, backend):
    # One UI flow over the browser grid and the exact locally bundled native grid.
    source = "1,2,3\n4,5,6\n7,8,9\n"
    driver.act("open", path="spreadsheet.csv")
    for native in [False, True]:
        if native:
            driver.act("native-table", url=(ROOT / "apps/apple-simple/Sources/Resources/math.html").as_uri() + "?table", backend=backend, source=source, format="csv")
        assert driver.act("spreadsheet-select", row=0, column=0) == "A1"
        assert driver.act("spreadsheet-select-header", axis="row", index=1) == "2:2"
        assert driver.act("spreadsheet-select-header", axis="column", index=1) == "B:B"
        assert driver.act("spreadsheet-select", row=0, column=0, endRow=1, endColumn=1) == "A1:B2"
        assert driver.act("spreadsheet-select", row=0, column=0, endRow=0, endColumn=1) == "A1:B1"
        touch_range = driver.act("spreadsheet-touch-range", row=0, column=0, endRow=1, endColumn=1)
        assert touch_range == "A1:B2", touch_range
        driver.act("spreadsheet-structure", button="+ Row")
        driver.act("spreadsheet-structure", button="+ Column")
        driver.act("spreadsheet-select", row=3, column=3)
        driver.act("spreadsheet-structure", button="− Row")
        driver.act("spreadsheet-select", row=0, column=3)
        driver.act("spreadsheet-structure", button="− Column")
        rectangle = [[0, 0], [0, 1], [1, 0], [1, 1]]
        for gesture in [{"touch": True}, {"continuing": True, "hoverRow": 0, "hoverColumn": 2},
                        {"continuing": True, "hoverRow": 2, "hoverColumn": 0, "releaseOutside": True}]:
            picked = driver.act("spreadsheet-formula-point", row=2, column=2, prefix="=sum(",
                                fromRow=1, fromColumn=1, toRow=0, toColumn=0, **gesture)
            assert picked == {"draft": "=sum(A1:B2", "preview": rectangle, "picked": rectangle}, picked
        driver.act("spreadsheet-formula", value=picked["draft"] + ")")
        eventually(lambda: driver.act("spreadsheet-value", row=2, column=2), "12")
        driver.act("spreadsheet-select", row=2, column=0)
        driver.act("spreadsheet-formula", value="0")
        eventually(lambda: driver.act("spreadsheet-value", row=2, column=0), "0")
        assert driver.act("spreadsheet-select", row=0, column=0, endRow=1, endColumn=0) == "A1:A2"
        driver.act("spreadsheet-fill", row=2, column=0)
        eventually(lambda: driver.act("spreadsheet-value", row=2, column=0), "7")
        assert driver.act("spreadsheet-height", key="Home") == 64
        assert driver.act("spreadsheet-height", key="ArrowDown") == 88
        if native:
            assert "=sum(A1:B2)" in driver.act("native-table-source")
            driver.act("native-table", url=(ROOT / "apps/apple-simple/Sources/Resources/math.html").as_uri() + "?table", backend=backend, source="", format="csv")
            driver.act("spreadsheet-select", row=0, column=0)
            driver.act("spreadsheet-formula", value="first")
            eventually(lambda: driver.act("native-table-source"), "first")
        else:
            driver.act("save")
            eventually(lambda: "=sum(A1:B2)" in (folder / "spreadsheet.csv").read_text(), True)
            driver.act("open", path="empty.csv")
            driver.act("spreadsheet-select", row=0, column=0)
            driver.act("spreadsheet-formula", value="first")
            driver.act("save")
            eventually(lambda: (folder / "empty.csv").read_text(), "first")


def table_acceptance(driver, folder):
    path = "table.md"; original = (folder / path).read_text()
    driver.act("open", path=path)
    driver.act("table-edit", before="120", value="121")
    driver.act("save")
    eventually(lambda: (folder / path).read_text(), original.replace("120", "121"))
    driver.act("undo", path=path)
    driver.act("save")
    eventually(lambda: (folder / path).read_text(), original)
    driver.act("table-source")
    assert "| café 😀 | 120 |" in driver.act("content", path=path)
    driver.act("table-grid")
    assert driver.act("table-keys", before="café 😀", keys=["ArrowRight"]) == "120"
    assert driver.act("resize-gutter") >= 64
    driver.act("open", path="stock.csv")
    assert driver.act("table-keys", before="Pen", keys=["ArrowRight"]) == "30"
    assert driver.act("table-caret") == {"start": 0, "end": 0}
    driver.act("table-edit", before="30", value="31")
    driver.act("save")
    eventually(lambda: (folder / "stock.csv").read_text(), 'Product,Stock\nPen,31\n')
    driver.act("table-source")
    assert "Pen,31" in driver.act("content", path="stock.csv")
    driver.act("table-grid")
    driver.act("open", path="caret.csv")
    driver.act("table-open-caret", before="abcdefghijk", offset=8)
    assert driver.act("table-caret") == {"start": 8, "end": 8}
    assert driver.act("table-keys", before="abcdefghijk", keys=["ArrowDown"]) == "x"
    assert driver.act("table-caret") == {"start": 1, "end": 1}
    assert driver.act("table-keys", before="x", keys=["ArrowDown"]) == "abcdefghijklm"
    assert driver.act("table-caret") == {"start": 8, "end": 8}
    driver.act("table-keys", before="abcdefghijklm", keys=["ArrowUp", "ArrowLeft", "ArrowDown"])
    assert driver.act("table-caret") == {"start": 0, "end": 0}
    assert (folder / "caret.csv").read_text() == "Label,Count\nabcdefghijk,1\nx,2\nabcdefghijklm,3\n"

def markdown_acceptance(driver):
    driver.act("open", path="source.md")
    formatting = driver.act("markdown-formatting", path="source.md")
    assert formatting["heading"]["size"] > formatting["size"]
    assert int(formatting["bold"]["weight"]) >= 650
    assert formatting["bold"]["text"] == "bold"
    assert formatting["italic"]["italic"] == "italic"

def acceptance(driver, folder):
    plain = folder / "notes.txt"
    driver.act("open", path="notes.txt")
    assert driver.act("minimap") == 40
    assert driver.act("content", path="notes.txt") == "before"
    driver.act("append", path="notes.txt", text=" saved")
    driver.act("save")
    eventually(plain.read_text, "before saved")
    driver.act("append", path="notes.txt", text=" later")
    driver.act("split"); driver.act("move")
    assert driver.act("content", path="notes.txt") == "before saved later"
    driver.act("undo", path="notes.txt")
    eventually(lambda: driver.act("content", path="notes.txt"), "before saved")
    driver.act("open", path="source.md")
    driver.act("reorder-tab", path="source.md", before="notes.txt")
    eventually(lambda: driver.act("tab-order", path="notes.txt"), ["source.md", "notes.txt"])
    driver.act("reorder-tab", path="notes.txt", before="source.md")
    eventually(lambda: driver.act("tab-order", path="notes.txt"), ["notes.txt", "source.md"])
    panes = driver.act("pane-count")
    for edge in ["left", "right", "top", "bottom"]:
        driver.act("dock-tab", path="notes.txt", target="source.md", edge=edge)
        if edge == "left": panes += 1
        eventually(lambda: driver.act("pane-count"), panes)
        assert driver.act("content", path="notes.txt") == "before saved"
    driver.act("dock-tab", path="notes.txt", target="source.md", edge="center")
    panes -= 1
    eventually(lambda: driver.act("pane-count"), panes)
    eventually(lambda: driver.act("tab-order", path="notes.txt"), ["source.md", "notes.txt"])
    driver.act("append", path="notes.txt", text=" local")
    driver.act("close-tab")
    assert any("notes.txt" in tab for tab in driver.act("tabs"))
    plain.write_text("external")
    driver.act("save")
    eventually(lambda: any("changed on disk" in alert for alert in driver.act("alerts")), True)
    assert plain.read_text() == "external"
    assert driver.act("content", path="notes.txt") == "before saved local"
    driver.act("open", path="source.md")
    driver.act("append", path="source.md", text="\nMore prose.\n")
    driver.act("save")
    eventually(lambda: (folder / "source.md").read_text().endswith("\nMore prose.\n"), True)
    driver.act("agent"); driver.act("draft", text="unsent message")
    driver.act("move")
    assert driver.act("read-draft") == "unsent message"
    terminal_acceptance(driver)
    driver.act("screenshot", path=str(ROOT / "target/simple-browser.png"))

def conversation_history_organization_acceptance(driver, folder):
    sessions = folder / "sessions"
    before = {path.name: path.read_bytes() for path in sessions.glob("*.md")}
    first, follow_up, second = "First conversation", "First follow-up", "Second conversation"
    branch = "Branch from agent · Independent branch"

    def show(view):
        driver.act("conversation-history-view", view=view)

    def observe(title, visible=True):
        assert driver.act("conversation-history-entry", title=title, visible=visible) is visible

    def change(title, label):
        driver.act("conversation-history-action", title=title, label=label)

    show("History")
    driver.act("conversation-history-entry", title=first, open=True)
    driver.act("draft", text="Keep my open draft")
    change(follow_up, "Favorite")
    show("Favorites")
    observe(first)
    observe(follow_up)
    observe(second, False)
    observe(branch, False)

    change(first, "Archive")
    observe(first, False)
    show("History")
    observe(branch)
    observe(second)
    observe(follow_up, False)
    assert driver.act("read-draft") == "Keep my open draft"

    driver.act("conversation-history-reload")
    show("Archived")
    observe(first)
    observe(follow_up)
    observe(branch, False)
    assert driver.act("conversation-history-is-favorite", title=first)
    assert "Follow-up answer" in driver.act(
        "conversation-history-entry", title=follow_up, open=True, answer="Follow-up answer")
    change(first, "Restore")
    observe(first, False)
    show("Favorites")
    observe(first)
    change(first, "Remove favorite")
    observe(first, False)
    show("History")
    observe(first)
    assert {path.name: path.read_bytes() for path in sessions.glob("*.md")} == before


def conversation_tabs_acceptance(driver, folder):
    # Contract: .hickory-source/frontends/contract.md. Real host history and browser gestures.
    before = {path.name: path.read_bytes() for path in (folder / "sessions").glob("*.md")}
    state = driver.act("conversation-tabs")
    assert state["firstDraft"] == state["restored"] == "Keep first\nnext line", state
    assert "10 tokens" in state["firstUsage"] and "25 tokens" not in state["firstUsage"], state
    assert "25 tokens" in state["secondUsage"] and "10 tokens" not in state["secondUsage"], state
    assert state["branches"] == 2 and state["below"] and state["empty"], state
    assert "First answer" in state["reading"] and "Second answer" not in state["reading"], state
    assert state["footerSettings"] == state["redundantHeading"] == 0, state
    assert len(state["splitUsage"]) == 2, state
    assert any("10 tokens" in usage for usage in state["splitUsage"]), state
    assert any("25 tokens" in usage for usage in state["splitUsage"]), state
    assert any("✦" in title for title in state["titles"]), state
    assert state["reused"] and state["originalPreserved"] and state["iconButton"], state
    assert state["assistantDraft"] == "First answer" and state["userDraft"] == "First conversation", state
    assert "First conversation" not in state["userReading"], state
    assert state["savedBranch"]["answer"] == "Edited first answer" and state["savedBranch"]["provider"] == "human-edited", state
    assert "0 tokens" in state["zeroTokens"] and state["openedVisible"] and state["reopenedVisible"], state
    assert all((folder / "sessions" / name).read_bytes() == data for name, data in before.items())

def agent_acceptance(driver, folder):
    # Hydrate a Python conversation before continuing through the real send route.
    # The unavailable model keeps this check independent of provider access.
    text = "# Explain this\n\n```python\nprint('café 😀')\n```"
    driver.act("open", path="source.md")
    state = driver.act("agent-documents", text=text)
    assert state["readonly"] and state["stable"] and state["quiet"] and state["closed"], state
    assert "sonnet" in state["pickedModel"], state
    assert "Ln 1, Col 2" in state["sourcePosition"], state
    assert state["retained"] == text, state
    context = state["payload"]["context"]
    assert context["live"] is True and len(context["buffers"]) == 1, context
    source = context["buffers"][0]
    assert source["path"] == "source.md" and source["content"] == (folder / "source.md").read_text(), source
    assert {key: value for key, value in state["payload"].items() if key != "context"} == {
        "backend": "python", "prompt": text, "parent_id": "agent-history",
        "provider": "claude-subscription", "model": "unavailable-model", "effort": "high",
    }, state
    assert "# Previous question" in state["history"], state
    driver.act("screenshot", path=str(ROOT / "target/simple-agent.png"))

    path = "conversation/draft.md"
    driver.act("startup-held", url=driver.url, gates={"stylesheet": "**/katex.min.css?inline*"})
    driver.act("agent")
    eventually(lambda: driver.act("capability-held", name="stylesheet"), True)
    driver.act("source-text-type", path=path, text=text)
    driver.act("remember-editor", path=path)
    draft = driver.act("source-text-state", path=path)
    assert draft["source"] == text
    geometry = driver.act("editor-size", path=path)
    driver.act("capability-release", name="stylesheet", fail=True)
    eventually(lambda: driver.act("agent-render-status").startswith("Inline rendering unavailable"), True)
    assert driver.act("editor-is-retained", path=path)
    assert driver.act("editor-size", path=path) == geometry
    assert driver.act("source-text-state", path=path) == draft

def recorded_patch_acceptance(driver, folder):
    paths = [("user", "author.md", "value = 41", "generated.py"),
             ("agent", "review.txt", "assistant draft", None)]
    saved = {path: (folder / path).read_bytes() for path in ["author.md", "review.txt", "generated.py", "sessions/recorded-patch.md"]}
    for side, target, replacement, consequence in paths:
        driver.act("open", path=target)
        driver.act("remember-editor", path=target)
        original = driver.act("source-text-state", path=target)
        driver.act("agent")
        message = f"conversation/recorded-patch-{side}.md"
        before = driver.act("source-text-state", path=message)
        assert "@@ -" in before["source"], before
        driver.act("recorded-patch-review", path=message)
        eventually(lambda: driver.act("proposal-state")["applyEnabled"], True)
        review = driver.act("proposal-state")["text"]
        assert replacement in review and f"recorded {'assistant' if side == 'agent' else side} fence" in review, review
        if consequence:
            assert f"Generated output: {consequence}" in review, review
        assert all((folder / path).read_bytes() == data for path, data in saved.items())
        applied = driver.act("proposal-apply")
        assert "Complete proposal applied to drafts" in applied, applied
        driver.act("proposal-close")
        assert driver.act("source-text-state", path=message) == before
        driver.act("open", path=target)
        assert driver.act("editor-is-retained", path=target)
        assert replacement in driver.act("source-text-state", path=target)["source"]
        driver.act("undo", path=target)
        assert driver.act("source-text-state", path=target)["source"] == original["source"]
        assert all((folder / path).read_bytes() == data for path, data in saved.items())


def prepare_recorded_construction(root, side="assistant"):
    # Use the same selected operation for either message. Keep its owning
    # document separate from the other archived message, which has no evidence.
    import hashlib
    from textwrap import dedent
    assert side in ("user", "assistant")
    root = root.resolve()
    session = root / "sessions" / "agent.md"
    session.parent.mkdir(parents=True)
    target = "materialized.txt"
    root.joinpath(target).write_text("Independent working file\n")
    source = dedent("""
    <hick:project/>

    <hick:patch id="selected">
    --- a/materialized.txt
    +++ b/materialized.txt
    @@ -1 +1 @@
    -old
    +middle
    </hick:patch>
    <hick:patch id="later" after="selected">
    --- a/materialized.txt
    +++ b/materialized.txt
    @@ -1 +1 @@
    -middle
    +final
    </hick:patch>
    """).strip("\n")
    if side == "user":
        source = source.replace("<hick:project/>", "The retained request supplies this patch.", 1)

    def snapshot(value):
        return {"entries": {target: {"bytes": list(value), "kind": "File", "executable": False, "origins": []}}}

    result = b"final\n"
    fingerprint = hashlib.sha256(json.dumps({target: [list(result), "File", False]}, separators=(",", ":")).encode()).hexdigest()
    response = '<hick:project sources="request.md"/>\nThe retained request owns this patch.' if side == "user" else source
    sources = {"response.md": response}
    user = "Inspect the retained construction"
    if side == "user":
        sources["request.md"] = source
        user = f"<hick:input>\n{source}\n</hick:input>"
    assistant = f"<hick:input>\n{response}\n</hick:input>"
    capture = {"root": str(root), "entry": "response.md", "documents": {},
        "inventory": {"documents": list(sources), "exclude": []}, "starting": snapshot(b"old\n"),
        "completed": snapshot(result), "ambiguous": [], "error": None, "response": response,
        "artifact": {"entry": "response.md", "sources": sources, "expected": fingerprint},
        "session": str(session)}
    evidence = json.dumps({"turn": "declared-construction", "capture": capture}).replace("<", "\\u003c")
    session.write_text('<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-11-18T10:00:00Z" doc=".hick-workspace-agent">\n' +
        '<hick:user id="u1" turn="declared-construction" provider="codex-subscription" model="test">' + user + '</hick:user>\n' +
        '<hick:assistant>' + assistant + '</hick:assistant>\n' +
        '<hick:context kind="acp-turn-status">{"turn":"declared-construction","status":"ok"}</hick:context>\n' +
        '<hick:context kind="literate-project-capture">' + evidence + '</hick:context>\n</hick:session>\n')
    return {path: (root/path).read_text() for path in (target, 'sessions/agent.md')}


def recorded_fence_debug_acceptance(driver, folder, backend):
    import hashlib
    from urllib.request import urlopen

    recording = folder / "sessions/agent.md"
    original = recording.read_bytes()
    with urlopen(backend + "/api/docs/workspace/agent/turns", timeout=5) as response:
        [turn] = json.load(response)["turns"]
    assert turn["id"] == "recorded-fence-host"
    driver.act("agent")
    for role, side, result in [("user", "prompt", 41), ("agent", "answer", 52)]:
        source = turn[side]
        opening = next(index for index, line in enumerate(source.splitlines(), 1) if line.endswith("python"))
        path = f"conversation/{turn['id']}-{role}.md"
        driver.act("remember-editor", path=path)
        run = driver.act("recorded-fence-run", path=path, opening=opening)
        assert run["address"] == {"side": "user" if role == "user" else "assistant",
                                  "source_hash": hashlib.sha256(source.encode()).hexdigest(),
                                  "opening_line": opening - 1}, run
        assert str(result + 1) in run["output"].splitlines() and "wrong" not in run["output"], run
        assert recording.read_bytes() == original
        driver.act("recorded-fence-debug", path=path, opening=opening, breakpoint=opening+1)
        eventually(lambda: driver.act("recorded-fence-position", path=path), f"value = {result}")
        launch = [item for item in driver.act("debug-wire") if item.get("op") == "start"][-1]
        assert re.fullmatch(r"hick-project:///[0-9a-f]{16}/message.md", launch["doc"]), launch
        assert launch["source_hash"] == hashlib.sha256(source.encode()).hexdigest(), launch
        driver.act("debug-step")
        eventually(lambda: driver.act("recorded-fence-position", path=path), "print(value + 1)")
        assert driver.act("debug-watch", path=path, expression="value", expected=str(result)) == str(result)
        driver.act("debug-continue")
        eventually(lambda: sum("exit code 0" in label for label in driver.act("debug-state")), 1 if role == "user" else 2)
        assert driver.act("editor-is-retained", path=path)
        assert recording.read_bytes() == original
        assert not (folder / "fence.py").exists()


def terminal_acceptance(driver):
    # Contract: .hickory-source/frontends/contract.md. Real PTY and real keys.
    driver.act("terminal")
    driver.act("terminal-draft", text="printf 'hickory-terminal-check\\n'")
    driver.act("terminal-send")
    eventually(lambda: "hickory-terminal-check" in driver.act("terminal-screen").splitlines(), True)
    # Cursor editing belongs to the shell, rather than a second input widget.
    driver.act("terminal-draft", text="printf 'cursor-X\\n'")
    for _ in range(4): driver.act("terminal-key", key="ArrowLeft")
    driver.act("terminal-key", key="Backspace")
    driver.act("terminal-draft", text="Y")
    driver.act("terminal-key", key="End")
    driver.act("terminal-send")
    eventually(lambda: "cursorYX" in driver.act("terminal-screen").splitlines(), True)
    driver.act("terminal-draft", text="printf 'retained-session\\n'")
    driver.act("move")
    driver.act("terminal-send")
    eventually(lambda: "retained-session" in driver.act("terminal-screen").splitlines(), True)
    # Raw mode receives one key without Enter; alternate-screen output must render.
    raw = "import os,tty,termios; old=termios.tcgetattr(0); tty.setraw(0); print('\\x1b[?1049h\\x1b[2J\\x1b[HRAW-READY',flush=True); b=os.read(0,1); termios.tcsetattr(0,termios.TCSADRAIN,old); print('\\x1b[?1049l\\r\\nRAW-'+b.hex(),flush=True)"
    import shlex
    driver.act("terminal-draft", text="python3 -c " + shlex.quote(raw)); driver.act("terminal-send")
    eventually(lambda: "RAW-READY" in driver.act("terminal-screen"), True)
    driver.act("terminal-key", key="Control+n")
    eventually(lambda: "RAW-0e" in driver.act("terminal-screen").splitlines(), True)
    before = driver.act("terminal-size")
    assert before["separateInputs"] == 0, before
    driver.act("terminal-resize", width=1350, height=900)
    eventually(lambda: driver.act("terminal-size")["width"] > before["width"], True)
    driver.act("terminal-draft", text="sleep 30")
    driver.act("terminal-send")
    driver.act("terminal-key", key="Control+c")
    driver.act("terminal-draft", text="printf 'interrupt-ok\\n'")
    driver.act("terminal-send")
    eventually(lambda: "interrupt-ok" in driver.act("terminal-screen").splitlines(), True)
    driver.act("terminal-draft", text="python3 -c \"import sys; print('EOF-READY',flush=True); print('EOF-'+str(len(sys.stdin.read(1))),flush=True)\"")
    driver.act("terminal-send")
    eventually(lambda: "EOF-READY" in driver.act("terminal-screen").splitlines(), True)
    driver.act("terminal-eof")
    eventually(lambda: "EOF-0" in driver.act("terminal-screen").splitlines(), True)
    protocol = driver.act("native-terminal-protocol", url=(ROOT / "apps/apple-simple/Sources/Resources/math.html").as_uri())
    assert protocol["input"] == "café 😀\x1b[A\x03", protocol
    assert protocol["sizes"][-1]["cols"] > protocol["sizes"][0]["cols"], protocol
    assert protocol["color"] == 1 and protocol["cursor"] is False, protocol
    assert "default-src 'none'" in protocol["network"], protocol


def embedded_acceptance(driver, folder):
    # Contract: .hickory-source/frontends/contract.md. Real controls, links and disk.
    driver.act("open", path="diagram.md")
    state = driver.act("embedded-controls", path="diagram.md")
    assert state["height"] < 280, state
    assert state["opened"].rstrip("\n") == (folder / "sample.txt").read_text().rstrip("\n"), state
    assert '"href":"sample.txt"' in (folder / "diagram.md").read_text()
    driver.act("open", path="mermaid.md")
    assert driver.act("mermaid-link").rstrip("\n") == (folder / "sample.txt").read_text().rstrip("\n")
    driver.act("open", path="scroll.md")
    state = driver.act("embedded-wheel", path="scroll.md")
    assert state["first"] > 0 and state["after"] > state["before"], state
    assert state["nested"] == 0 and state["moved"] > 0, state


def inline_features_acceptance(driver, folder):
    # These observations go through real DOM controls and exact workspace bytes.
    for path, kind in [("diagram.md", "diagram"), ("fenced.md", "syntax"), ("patch.md", "patch")]:
        original = (folder / path).read_text()
        driver.act("open", path=path)
        state = driver.act("inline-feature-state", path=path, kind=kind)
        if kind == "diagram":
            assert state == {"nodes": 3, "position": "absolute"}, state
        elif kind == "syntax":
            assert "def" in state["keywords"] and "return" in state["keywords"] and state["colored"], state
            driver.act("fold", path=path, line=1)
            assert driver.act("fold-count", path=path) == 1
            driver.act("unfold", path=path, line=1)
        else:
            assert state["editors"] == 2 and not state["alerts"], state
            assert any('print(2)' in text for text in state["code"]), state
        assert (folder / path).read_text() == original, "Presentation must not save source"


def table_dataflow_acceptance(driver, folder, backend):
    """One real dataflow exercised through browser and native shared surfaces."""
    driver.act("browser-surface", url=driver.url)
    path = folder / "tables.md"
    def record():
        result = subprocess.run([str(ROOT / "target/debug/hick"), "run", "tables.md", "--cache"], cwd=folder, env={**os.environ, "HICKORY_EXECUTOR": "local", "HICKORY_STATE_DIR": str(folder.parent / "state")}, capture_output=True, text=True)
        assert result.returncode == 0, result.stderr
    marker = folder / "table-producer-ran"
    driver.act("open", path="tables.md")
    state = driver.act("table-dataflow-state", tea="6")
    assert state["readonly"] == 0 and not marker.exists(), state
    record()
    assert marker.read_text() == "ran"
    driver.act("refresh-recorded")
    state = driver.act("table-dataflow-state", tea="6", total="11")
    assert state == {"tea": "6", "cake": "5", "total": "11", "readonly": 1}, state
    marker.unlink()
    driver.act("table-dataflow-edit", value="4")
    driver.act("save")
    eventually(lambda: "Tea,4" in path.read_text(), True)
    state = driver.act("table-dataflow-state", tea="8")
    assert state["tea"] == "8" and state["cake"] == "5" and not marker.exists(), state
    record()
    assert marker.read_text() == "ran"
    driver.act("refresh-recorded")
    state = driver.act("table-dataflow-state", tea="8", total="13")
    assert state["total"] == "13" and state["readonly"] == 1, state
    source = path.read_text()
    order = re.search(r'<hick:table id="Order"[^>]*>\n(.*?)</hick:table>', source, re.S).group(1)
    driver.act("native-table", url=(ROOT / "apps/apple-simple/Sources/Resources/math.html").as_uri() + "?table", backend=backend, source=order, format="csv", header=True, language="python", scope={"source": source, "table": "Order"})
    # The same real formula route receives the full document through the native bridge.
    eventually(lambda: driver.act("spreadsheet-value", row=1, column=2), "8")
    output = "\x1eHICKORY_DISPLAY " + json.dumps({"kind": "table", "schema": [{"name": "Value", "dtype": "text"}], "rows": [["=1+1"]], "offset": 0, "has_more": False})
    state = driver.act("native-output", url=(ROOT / "apps/apple-simple/Sources/Resources/math.html").as_uri(), source=output)
    assert "=1+1" in state["text"] and state["editable"] == 0, state

if __name__ == "__main__":
    processes = []; driver = None
    with tempfile.TemporaryDirectory(prefix="hickory-simple-") as temporary:
        folder = Path(temporary) / "workspace"; folder.mkdir()
        for name, unit in [("four.py", "    "), ("two.py", "  "), ("tabs.py", "\t")]:
            (folder / name).write_text("def orders():\n" + unit + "pass\n# end")
        (folder / "scroll.md").write_text("Before paragraph.\n\n" * 8 + "| A | B |\n| --- | --- |\n" + "| value | 1 |\n" * 50 + "\nAfter paragraph.\n\n" * 80)
        (folder / "mermaid.md").write_text('<hick:diagram renderer="mermaid">\nflowchart TD\n A[Source]\n click A "sample.txt"\n</hick:diagram>')
        (folder / "sample.txt").write_text("Linked source file\n")
        (folder / "diagram.md").write_text('<hick:diagram renderer="graph">{"topology":{"nodes":[{"id":"a","href":"sample.txt"},{"id":"b"},{"id":"c"}],"edges":[{"from":"a","to":"b"}]},"layout":{"a":{"x":0,"y":0},"b":{"x":180,"y":0},"c":{"x":360,"y":0}}}</hick:diagram>')
        (folder / "fenced.md").write_text('# Fenced section\n\n```python\ndef coupon():\n    return "café 😀"\n```\n')
        (folder / "patch.md").write_text('<hick:project/>\n<hick:file id="base" path="x.py">\nprint(1)\n</hick:file>\n```patch id="change" after="base"\n--- a/x.py\n+++ b/x.py\n@@ -1 +1 @@\n-print(1)\n+print(2)\n```\n')
        table_source = (ROOT / "examples/simple-ui/tables.md").read_text()
        table_source = table_source.replace('order = table("Order")', f'from pathlib import Path\nPath({str(folder / "table-producer-ran")!r}).write_text("ran")\norder = table("Order")')
        (folder / "tables.md").write_text(table_source)
        (folder / "notes.txt").write_text("before")
        (folder / "enter.py").write_text("class C:\n    def f():")
        (folder / "words.txt").write_text("THIS_IS_ONE_WORD HTTPServer42 café😀")
        (folder / "search.txt").write_text("😀 name=12\nname=34\n")
        (folder / "fold.txt").write_text("function main() {\n  first()\n  second()\n}\n")
        (folder / "source.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n# A literate document\n\n**bold** and *italic*\n</h:doc>\n')
        (folder / "table.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\nBefore\n\n| A | B |\n| --- | ---: |\n| café 😀 | 120 |\n\nAfter\n</h:doc>\n')
        (folder / "stock.csv").write_text('Product,Stock\nPen,30\n')
        (folder / "empty.csv").write_text("")
        (folder / "spreadsheet.csv").write_text("1,2,3\n4,5,6\n7,8,9\n")
        (folder / "caret.csv").write_text("Label,Count\nabcdefghijk,1\nx,2\nabcdefghijklm,3\n")
        (folder / "renamea.py").write_text("value = 42\n")
        (folder / "renameb.py").write_text("from renamea import value\nprint(value)\n")
        if "--workspace-changes-only" in sys.argv:
            (folder / "renamec.py").write_text("from renamea import value as kept\nprint(kept)\n")
        (folder / "symbols.py").write_text("class Widget:\n    def render(self):\n        return 1\n")
        (folder / "background.py").write_text('def background():\n    amount: int = "wrong"\n    return amount\n')
        (folder / "problem.py").write_text('amount: int = "wrong"\n')
        (folder / "navigation.py").write_text("from semantic_target import value\nprint(value)\n\ndef calculate(amount):\n    return adjust(amount)\n\ndef adjust(amount):\n    return amount + 1\n\ncalculate(value)\n")
        (folder / "semantic_target.py").write_text("prefix = 0; value = 42\n")
        if "--call-hierarchy-only" in sys.argv:
            subprocess.run(["git", "init", "-q", str(folder)], check=True, timeout=10)
            for path, source in CALL_HIERARCHY_SOURCE.items():
                (folder / path).write_text(source)
        if "--managed-formatting-only" in sys.argv:
            managed_formatting_fixture(folder)
        installed_provider = os.environ.get("HICKORY_SIMPLE_TEST_PYTHON_PROVIDER")
        if (any(flag in sys.argv for flag in ("--semantic-origin-only", "--call-hierarchy-only"))
                or "--workspace-changes-only" in sys.argv and installed_provider):
            provider = Path(os.environ["HICKORY_SIMPLE_TEST_PYTHON_PROVIDER"]).resolve(strict=True)
            if not provider.is_file():
                raise RuntimeError("The installed basedpyright language server script must be a file.")
            (folder / ".hick-lsp.json").write_text(json.dumps({"servers": {"python": {"command": ["node", str(provider), "--stdio"]}}}))
        elif "--semantic-navigation-only" in sys.argv or "--problems-only" in sys.argv or "--workspace-changes-only" in sys.argv or "--workspace-analysis-only" in sys.argv or "--search-everywhere-only" in sys.argv:
            subprocess.run([str(ROOT / "target/debug/hick"), "lsp", "install", "python", "--root", str(folder)], check=True, timeout=180)
        (folder / "cache.py").write_text("counter = 1\nprint(counter)\n")
        (folder / "debug.py").write_text("value = 1\nvalue += 1\nprint(value)\n")
        if "--delayed-capabilities-only" in sys.argv or "--early-source-only" in sys.argv:
            nested = folder / "nested"; nested.mkdir()
            (nested / "README #é.md").write_text('<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n' + "\n".join(
                f"# Section {i}" if i % 9 == 0 else f"Readable paragraph {i}" for i in range(160)) + "\n</hick:doc>\n")
            (nested / "peer.txt").write_text("Other text in the containing folder\n")
            (nested / "test_delay.py").write_text("def test_before():\n    assert True\n")
            (nested / "producer.md").write_text('<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n<hick:file path="generated.txt">Generated text\n</hick:file>\n</hick:doc>\n')
            (nested / "generated.txt").write_text("Generated text\n")
            (nested / "delay.csv").write_text("Product,Count\n" + "".join(f"product-{i},{i}\n" for i in range(100)))
        if "--debug-threads-only" in sys.argv:
            virtualenv=Path(os.environ.get("HICKORY_THREAD_TEST_VENV",str(ROOT/"target/thread-debug-env"))).resolve()
            if not (virtualenv/"bin/python").exists():raise RuntimeError("Prepare the explicit debugpy test virtualenv first: "+str(virtualenv))
            (folder/".venv").symlink_to(virtualenv,target_is_directory=True)
            debug_path="%2F λ?#.py" if "--debug-literal" in sys.argv else "thread_debug.py"
            (folder/".hick-launch.json").write_text(json.dumps({"version":1,"profiles":[{"id":"thread-test","name":"Thread test","program":debug_path,"default":True}]}))
            (folder/debug_path).write_text("import threading\nimport time\nready = threading.Event()\nfinish = threading.Event()\ndef worker():\n    worker_value = 'worker'\n    ready.set()\n    while not finish.is_set():\n        worker_value = 'worker'\nthread = threading.Thread(target=worker, name='Hickory worker')\nthread.start()\nready.wait()\nmain_value = 'main'\npause_marker = 1\nfinish.set()\nthread.join()\n")
        if {"--model-diagram-only", "--workbook-only", "--workbook-apple-only"}.intersection(sys.argv):
            demo = folder / "workbook-demo"
            demo.mkdir()
            for name in ("project.md", "workbook.md", "orders.py", "prepare.py", "test_orders.py", "architecture.md"):
                shutil.copyfile(ROOT / "examples/simple-ui" / name, demo / name)
            shutil.copytree(ROOT / "examples/simple-ui/shop", demo / "shop")
        if {"--workbook-only", "--workbook-apple-only"}.intersection(sys.argv):
            venv = Path(os.environ.get("HICKORY_THREAD_TEST_VENV", str(ROOT / "target/thread-debug-env"))).resolve()
            if not (venv / "bin/python").exists():
                raise RuntimeError("Prepare the explicit debugpy test virtualenv first: " + str(venv))
        if "--recorded-patch-only" in sys.argv:
            construction = '<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n'
            (folder / "project.md").write_text(construction + '<hick:project sources="author.md"/>\n</hick:doc>\n')
            (folder / "author.md").write_text(construction + '<hick:file path="generated.py">\nvalue = 1\n</hick:file>\n</hick:doc>\n')
            (folder / "generated.py").write_text("value = 1\n")
            (folder / "review.txt").write_text("saved note\n")
            (folder / "sessions").mkdir()
            (folder / "sessions/recorded-patch.md").write_text('''<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-11-13T10:00:00Z" doc=".hick-workspace-agent">
<hick:user id="u1" turn="recorded-patch" provider="codex-subscription" model="test">Review this construction.

```diff
--- a/author.md
+++ b/author.md
@@ -3 +3 @@
-value = 1
+value = 41
```
</hick:user>
<hick:assistant>Review this note.

\x60\x60\x60patch
--- a/review.txt
+++ b/review.txt
@@ -1 +1 @@
-saved note
+assistant draft
\x60\x60\x60
</hick:assistant>
<hick:context kind="acp-turn-status">{"turn":"recorded-patch","status":"ok"}</hick:context>
</hick:session>
''')
        if {"--conversation-tabs-only", "--conversation-history-only"}.intersection(sys.argv):
            (folder / "sessions").mkdir()
            for name, number, amount in [("First", 1, 10), ("Second", 2, 25)]:
                (folder / f"sessions/{number}.md").write_text(
                    '<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-11-13T10:00:00Z" doc=".hick-workspace-agent">\n'
                    + f'<hick:user id="u{number}" turn="conversation-{number}" provider="codex-subscription" model="test">{name} conversation</hick:user>\n'
                    + f'<hick:context kind="python-runtime">{{"backend":"python","turn":"conversation-{number}","context":null,"prior_turns":[]}}</hick:context>\n'
                    + f'<hick:assistant>{name} answer</hick:assistant>\n'
                    + f'<hick:usage turn="conversation-{number}" input="{amount}" output="0" cache-read="0" cache-write="0"/>\n'
                    + f'<hick:context kind="acp-turn-status">{{"turn":"conversation-{number}","status":"ok"}}</hick:context>\n'
                    + '</hick:session>\n')
        if "--conversation-history-only" in sys.argv:
            first = folder / "sessions/1.md"
            tail = (
                '<hick:user id="u3" turn="conversation-3" parent="conversation-1">First follow-up</hick:user>\n'
                '<hick:assistant>Follow-up answer</hick:assistant>\n'
            )
            first.write_text(first.read_text().replace('</hick:session>', tail + '</hick:session>'))
            (folder / "sessions/branch.md").write_text(
                '<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-11-13T10:01:00Z" doc=".hick-workspace-agent">\n'
                '<hick:user id="u4" turn="conversation-4" parent="conversation-1">Independent branch</hick:user>\n'
                '<hick:assistant>Branch answer</hick:assistant>\n'
                '</hick:session>\n'
            )
        if "--agent-only" in sys.argv:
            (folder / "sessions").mkdir()
            (folder / "sessions/agent.md").write_text("""<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-11-13T10:00:00Z" doc=".hick-workspace-agent">
<hick:user id="u1" turn="agent-history" provider="codex-subscription" model="test"># Previous question</hick:user>
<hick:context kind="python-runtime">{"backend":"python","turn":"agent-history","context":null,"prior_turns":[]}</hick:context>
<hick:assistant># Previous answer</hick:assistant>
<hick:context kind="acp-turn-status">{"turn":"agent-history","status":"ok"}</hick:context>
</hick:session>
""")
        environment = {**os.environ, "HICKORY_STATE_DIR": str(Path(temporary) / "state")}
        if {"--workbook-only", "--workbook-apple-only"}.intersection(sys.argv):
            environment["PATH"] = str(venv / "bin") + os.pathsep + environment["PATH"]
        if "--recorded-fence-debug-only" in sys.argv:
            (folder / "sessions").mkdir()
            (folder / "sessions/agent.md").write_text('''<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-11-18T10:00:00Z" doc=".hick-workspace-agent">
<hick:user id="u1" turn="recorded-fence-host" provider="codex-subscription" model="test">Question before code.

```python
value = 41
print(value + 1)
```

```python
raise RuntimeError('wrong user fence')
```
</hick:user>
<hick:assistant>Recorded answer.

~~~python
value = 52
print(value + 1)
~~~

~~~python
raise RuntimeError('wrong assistant fence')
~~~
</hick:assistant>
<hick:context kind="acp-turn-status">{"turn":"recorded-fence-host","status":"ok"}</hick:context>
</hick:session>
''')
            environment["PATH"] = str(ROOT / "target/thread-debug-env/bin") + os.pathsep + environment["PATH"]
        try:
            with socket.socket() as reservation:
                reservation.bind(("127.0.0.1", 0)); ui_port = reservation.getsockname()[1]
            ui_origin = f"http://127.0.0.1:{ui_port}"
            engine = start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE", str(ROOT / "target/debug/examples/engine_client")), str(folder), "window", ui_origin], env=environment); processes.append(engine)
            backend = origin(engine)
            if "--workbook-apple-only" in sys.argv:
                subprocess.run(["swift", "test", "--package-path", "apps/apple-simple", "--scratch-path",
                    "target/apple-simple-spm", "--filter", "ParityTests/testLiveWorkbookContextAndFenceDebug"],
                    cwd=ROOT, env={**environment, "HICKORY_WORKBOOK_TEST_ORIGIN": backend, "HICKORY_WORKBOOK_TEST_ROOT": str(folder)}, check=True)
                sys.exit(0)
            vite = start(["node", "node_modules/vite/bin/vite.js", "--config", "vite.simple.config.ts", "--host", "127.0.0.1", "--port", str(ui_port), "--strictPort"], cwd=ROOT / "apps/web", env={**environment, "HICKORY_API_ORIGIN": backend}); processes.append(vite)
            driver = BrowserDriver(origin(vite) + "/simple.html", navigate=not any(flag in sys.argv for flag in ["--delayed-capabilities-only", "--early-source-only", "--workspace-welcome-only"]))
            if "--workspace-welcome-only" in sys.argv:
                welcome_acceptance(driver)
                print("Browser: neutral welcome, explicit server address, failed retry and recovered file tree passed")
            elif "--new-window-only" in sys.argv:
                exclusive_new_window_acceptance(driver, folder)
                print("Browser: disconnected New Window, duplicate workspace refusal, preserved draft and release on close passed")
            elif "--early-source-only" in sys.argv:
                early_source_acceptance(driver, folder)
                print("Browser: readable first-open source survives delayed and failed ownership; guarded editing attaches to the same editor")
            elif "--delayed-capabilities-only" in sys.argv:
                delayed_capabilities_acceptance(driver, folder)
                print("Browser: requested file and containing tree, slow and failed capabilities, retained source, selection, scroll, Undo, drafts and generated-file protection passed")
            elif "--recorded-patch-only" in sys.argv:
                recorded_patch_acceptance(driver, folder)
                print("Browser: recorded user and assistant patches reached guarded review, draft Apply and Undo")
            elif "--conversation-history-only" in sys.argv:
                conversation_history_organization_acceptance(driver, folder)
                print("Browser: favorite, archive, reopen, restore and remove favorite through real Agent History, with independent branches and unchanged session files")
            elif "--conversation-tabs-only" in sys.argv:
                conversation_tabs_acceptance(driver, folder)
                print("Browser: independent conversation tabs, retained multiline drafts, message-bound branches, pane usage and immutable history passed")
            elif "--agent-only" in sys.argv:
                agent_acceptance(driver, folder)
                print("Browser: literate history, read-only messages, retained draft, subscription controls and Python-only HTTP request passed")
            elif "--recorded-fence-debug-only" in sys.argv:
                recorded_fence_debug_acceptance(driver, folder, backend)
                print("Browser: selected Run and real debugpy breakpoints, Step, Watch and Continue passed on both recorded message sides")
            elif "--terminal-only" in sys.argv:
                terminal_acceptance(driver)
                print("Browser: direct PTY input, cursor editing, raw/alternate screen, resize, pane reattachment, interrupt, EOF and native renderer protocol passed")
            elif "--table-dataflow-only" in sys.argv:
                table_dataflow_acceptance(driver, folder, backend)
                print("Browser and native bundle: named formulas, explicit Python inputs, published read-only tables and receipt refresh passed")
            elif "--spreadsheet-only" in sys.argv:
                spreadsheet_acceptance(driver, folder, backend)
                print("Browser and native bundle: range selection, structure, formula range picking, real Python evaluation and height controls passed")
            elif "--embedded-only" in sys.argv:
                embedded_acceptance(driver, folder)
                print("Browser: compact diagram, gutter source round trip, graph/Mermaid file links and stationary-pointer scrolling passed")
            elif "--inline-features-only" in sys.argv:
                inline_features_acceptance(driver, folder)
                print("Browser: real inline diagram, fenced syntax colors, heading fold and editable project patch passed")
            elif "--smart-enter-only" in sys.argv:
                smart_enter_acceptance(driver, folder)
                print("Browser: live Enter modes, real saved indentation, stationary settings and undo passed")
            elif "--editor-navigation-only" in sys.argv:
                editor_navigation_acceptance(driver, folder)
                print("Browser: independent persisted arrow settings, identifier pieces, Shift reversal and stable editor geometry passed")
            elif "--file-editing-only" in sys.argv:
                file_editing_acceptance(driver, folder)
                print("Browser: File menu, detected Tab alignment, Save All and Save As passed")
            elif "--table-only" in sys.argv:
                table_acceptance(driver, folder)
                print("Browser: inline table grid, real saved Markdown, source reveal and undo passed")
            elif "--workspace-changes-only" in sys.argv:
                workspace_changes_acceptance(driver, folder)
                print("Browser: genuine three-file rename, stale Apply and Undo refusal, retained editor, coordinated Undo/Redo and unchanged saved bytes passed")
            elif "--managed-formatting-only" in sys.argv:
                managed_formatting_acceptance(driver, folder)
                print("Browser: genuine clangd formatting preserved CRLF, the prose draft, editor identity and saved bytes through Apply, Undo and Redo")
            elif "--workspace-search-only" in sys.argv:
                workspace_search_acceptance(driver, folder)
                print("Browser: exhaustive workspace regex search, Unicode columns, stationary geometry, retained editor, stale-source refusal and unchanged disk passed")
            elif "--search-everywhere-only" in sys.argv:
                search_everywhere_acceptance(driver,folder)
                print("Browser: shared menu/action commands, disabled reasons, keyboard entry, real type/member search, source-bound reveal, stale-type retirement, retained editor/draft/geometry and unchanged disk passed")
            elif "--workspace-analysis-only" in sys.argv:
                workspace_analysis_acceptance(driver,folder)
                print("Browser: workspace acquisition, unopened-source diagnostics, unchanged active editor/draft/geometry, floating status/menu report and unchanged disk passed")
            elif "--problems-only" in sys.argv:
                problems_acceptance(driver, folder)
                print("Browser: source-hash-bound provider Problems, precise next-problem navigation, status/menu access, stationary geometry, retained draft and unchanged disk passed")
            elif "--call-hierarchy-only" in sys.argv:
                call_hierarchy_acceptance(driver, folder)
                print("Browser: real cross-file callers and callees, exact UTF-16 carets, Back, retained draft and renamed-target refusal passed")
            elif "--semantic-origin-only" in sys.argv:
                semantic_navigation_acceptance(driver, folder, held_origin=True)
                print("Browser: genuine definition retired after a same-tab caret move during HTTP, fresh definition and Back preserved exact authored positions and saved bytes")
            elif "--semantic-navigation-only" in sys.argv:
                semantic_navigation_acceptance(driver, folder)
                print("Browser: real symbols/calls, Back/Forward with retained editors, exact columns, stationary geometry and unchanged disk passed")
            elif "--language-cache-only" in sys.argv:
                language_cache_acceptance(driver, folder)
                print("Browser: real Python colors retained across typing and undo; disk unchanged")
            elif "--search-only" in sys.argv:
                search_acceptance(driver, folder)
                print("Browser: regex replacement, exact saved source, stationary search and undo passed")
            elif "--fold-only" in sys.argv:
                folding_acceptance(driver, folder)
                print("Browser: folding, exact saved source, moved folds and undo passed")
            elif "--debug-threads-only" in sys.argv:
                debug_thread_acceptance(driver,folder)
                print("Browser: actual debugpy thread read/selection, selected-frame watches, unchanged editor identity/geometry/source and explicit stop passed")
            elif "--model-diagram-only" in sys.argv:
                model_diagram_acceptance(driver, folder)
                print("Browser: repository diagram source, automatic saved dependency update and file navigation passed")
            elif "--workbook-only" in sys.argv:
                workbook_acceptance(driver, folder)
                print("Browser workbook: deliberate link, context and fence breakpoints, stdout and stderr passed")
            elif "--debug-only" in sys.argv:
                debug_acceptance(driver)
                print("Browser: inline DAP pause, moved session, step values and stop passed")
            elif "--markdown-only" in sys.argv:
                markdown_acceptance(driver)
                print("Browser: Markdown headings, bold and italic passed")
            else:
                markdown_acceptance(driver)
                acceptance(driver, folder)
                spreadsheet_acceptance(driver, folder, backend)
                table_dataflow_acceptance(driver, folder, backend)
                print("Browser: real saves, conflict refusal, retained undo/drafts, shared spreadsheet and table dataflow passed")
        finally:
            if driver: driver.close()
            for process in reversed(processes): stop(process)
