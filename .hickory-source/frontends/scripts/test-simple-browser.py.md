<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Drive the new browser through real save and rearrangement flows

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The acceptance flow launches an isolated real engine on an ephemeral port, then a Vite process and a browser adapter. No agent prompt is sent. Computed browser styles verify Markdown heading size, bold weight and italics on an opened literate document. Run `python3 scripts/test-simple-browser.py --markdown-only` for the focused formatting witness. Chat formatting and HTML safety have component witnesses; this real-engine flow does not claim provider-message coverage. Disk assertions cover successful plain/literate saves and conflict refusal; UI assertions cover moved document undo and moved agent drafts.

The focused `--debug-only` path launches a real Python adapter, places a line breakpoint, moves the paused tab, steps and checks inline values before stopping. It requires an installed Python debugger and a working Chromium host; a startup failure is not a passing debugger witness.

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
        (folder / "notes.txt").write_text("before")
        (folder / "source.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n# A literate document\n\n**bold** and *italic*\n</h:doc>\n')
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
            if "--debug-only" in sys.argv:
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
