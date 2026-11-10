<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Drive the new browser through real save and rearrangement flows

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The acceptance flow launches an isolated real engine on an ephemeral port, then a Vite process and a browser adapter. No agent prompt is sent. Computed browser styles verify Markdown heading size, bold weight and italics on an opened literate document. Run `python3 scripts/test-simple-browser.py --markdown-only` for the focused formatting witness. Chat formatting and HTML safety have component witnesses; this real-engine flow does not claim provider-message coverage. Disk assertions cover successful plain/literate saves and conflict refusal; UI assertions cover moved document undo and moved agent drafts.

The focused `--debug-only` path launches a real Python adapter, places a line breakpoint, moves the paused tab, steps and checks inline values before stopping. It requires an installed Python debugger and a working Chromium host; a startup failure is not a passing debugger witness.

The focused `--search-only` flow uses menu commands and visible search fields, checks editor dimensions, saves exact regex replacement bytes to the workspace, and undoes through the real editor. The browser driver owns platform selectors; this acceptance flow imports no frontend implementation.

The focused `--fold-only` flow clicks the visible right-hand line menu, saves while folded, moves the same editor to another pane, unfolds and undoes. Its assertions read actual workspace bytes and visible content through the driver; no framework state is imported. Native gesture adapters are not implemented by this browser witness.

The table-only flow opens a real literate document from disk, edits a grid cell, saves and compares exact Markdown, then undoes and reveals source. It exercises cold parser startup as well as the grid. The adapter owns DOM selectors; these assertions express the user-visible save and undo behavior. Native gesture drivers for the same flow remain unimplemented.

The file-editing acceptance path opens real files through the File menu, presses Return and two Tabs, saves all three whitespace styles and checks Save As against disk. It runs alone with `--file-editing-only`; browser startup failures are not passing UI evidence.

<ui:copy id="simple-0799b7d31b-0">
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

if __name__ == "__main__":
    processes = []; driver = None
    with tempfile.TemporaryDirectory(prefix="hickory-simple-") as temporary:
        folder = Path(temporary) / "workspace"; folder.mkdir()
        for name, unit in [("four.py", "    "), ("two.py", "  "), ("tabs.py", "\t")]:
            (folder / name).write_text("def orders():\n" + unit + "pass\n# end")
        (folder / "notes.txt").write_text("before")
        (folder / "search.txt").write_text("😀 name=12\nname=34\n")
        (folder / "fold.txt").write_text("function main() {\n  first()\n  second()\n}\n")
        (folder / "source.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n# A literate document\n\n**bold** and *italic*\n</h:doc>\n')
        (folder / "table.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\nBefore\n\n| A | B |\n| --- | ---: |\n| café 😀 | 120 |\n\nAfter\n</h:doc>\n')
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
            if "--file-editing-only" in sys.argv:
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
                print("Browser: real plain/literate saves, conflict refusal, moved undo and agent draft passed")
        finally:
            if driver: driver.close()
            for process in reversed(processes): stop(process)
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="scripts/test-simple-browser.py"><ui:paste select="#simple-0799b7d31b-0"/></ui:file>

</ui:doc>
