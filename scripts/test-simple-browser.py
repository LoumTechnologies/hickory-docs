# Contract: .hickory-source/frontends/contract.md
"""Focused real-engine browser acceptance for the independent frontend."""
import json
import os
from pathlib import Path
import queue
import re
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time

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
    def __init__(self, url):
        self.url = url
        self.process = start(["node", "scripts/simple-browser-driver.mjs"])
        try: self.act("launch", url=url)
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
    size = driver.act("editor-size", path=path)
    driver.act("search-open")
    assert driver.act("editor-size", path=path) == size
    driver.act("search-query", query=r"name=(\d+)", replacement="$1", regex=True)
    driver.act("search-replace-all")
    driver.act("save")
    eventually(lambda: (folder / path).read_text(), "😀 12\n34\n")
    driver.act("search-close")
    driver.act("undo", path=path)
    eventually(lambda: driver.act("content", path=path), source)

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
    driver.act("spreadsheet-language", language="python")
    for native in [False, True]:
        if native:
            driver.act("native-table", url=(ROOT / "apps/apple-simple/Sources/Resources/math.html").as_uri() + "?table", backend=backend, source=source, format="csv")
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
        draft = driver.act("spreadsheet-formula-point", row=2, column=2, prefix="=sum(", fromRow=0, fromColumn=0, toRow=1, toColumn=1, touch=True)
        assert draft == "=sum(A1:B2", draft
        driver.act("spreadsheet-formula", value=draft + ")")
        eventually(lambda: driver.act("spreadsheet-value", row=2, column=2), "12")
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
    driver.act("terminal")
    driver.act("terminal-draft", text="printf 'hickory-terminal-check\\n'")
    driver.act("terminal-send")
    eventually(lambda: "hickory-terminal-check" in driver.act("terminal-screen").splitlines(), True)
    driver.act("terminal-draft", text="pending terminal input")
    driver.act("move")
    assert driver.act("read-terminal-draft") == "pending terminal input"
    driver.act("terminal-eof")
    driver.act("screenshot", path=str(ROOT / "target/simple-browser.png"))

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
    driver.act("native-table", url=(ROOT / "apps/apple-simple/Sources/Resources/math.html").as_uri() + "?table", backend=backend, source=order, format="csv", header=True, scope={"source": source, "table": "Order"})
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
        (folder / "diagram.md").write_text('<hick:diagram renderer="graph">{"topology":{"nodes":[{"id":"a"},{"id":"b"},{"id":"c"}],"edges":[{"from":"a","to":"b"}]}}</hick:diagram>')
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
        (folder / "cache.py").write_text("counter = 1\nprint(counter)\n")
        (folder / "debug.py").write_text("value = 1\nvalue += 1\nprint(value)\n")
        environment = {**os.environ, "HICKORY_STATE_DIR": str(Path(temporary) / "state")}
        try:
            with socket.socket() as reservation:
                reservation.bind(("127.0.0.1", 0)); ui_port = reservation.getsockname()[1]
            ui_origin = f"http://127.0.0.1:{ui_port}"
            engine = start([str(ROOT / "target/debug/examples/engine_client"), str(folder), "window", ui_origin], env=environment); processes.append(engine)
            backend = origin(engine)
            vite = start(["node", "node_modules/vite/bin/vite.js", "--config", "vite.simple.config.ts", "--host", "127.0.0.1", "--port", str(ui_port), "--strictPort"], cwd=ROOT / "apps/web", env={**environment, "HICKORY_API_ORIGIN": backend}); processes.append(vite)
            driver = BrowserDriver(origin(vite) + "/simple.html")
            if "--table-dataflow-only" in sys.argv:
                table_dataflow_acceptance(driver, folder, backend)
                print("Browser and native bundle: named formulas, explicit Python inputs, published read-only tables and receipt refresh passed")
            elif "--spreadsheet-only" in sys.argv:
                spreadsheet_acceptance(driver, folder, backend)
                print("Browser and native bundle: range selection, structure, formula range picking, real Python evaluation and height controls passed")
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
            elif "--language-cache-only" in sys.argv:
                language_cache_acceptance(driver, folder)
                print("Browser: real Python colors retained across typing and undo; disk unchanged")
            elif "--search-only" in sys.argv:
                search_acceptance(driver, folder)
                print("Browser: regex replacement, exact saved source, stationary search and undo passed")
            elif "--fold-only" in sys.argv:
                folding_acceptance(driver, folder)
                print("Browser: folding, exact saved source, moved folds and undo passed")
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
