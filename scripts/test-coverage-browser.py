"""Real engine discovery and report import through the compact browser menu."""
import hashlib
import importlib.util
import os
from pathlib import Path
import socket
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-coverage-ui-") as temporary:
        root=Path(temporary)/"workspace";root.mkdir()
        (root/"Cargo.toml").write_text('[package]\nname="coverage_ui_witness"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="lib.rs"\n')
        source = (
            '#[test] fn coverage_ui_case() { panic!("discovery must not execute this body"); }\n'
            '#[test] fn coverage_extra_case() { assert!(true); }\n'
        )
        path = "lib.rs"
        (root / path).write_text(source)
        environment={**os.environ,"HICKORY_STATE_DIR":str(Path(temporary)/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            driver.act("coverage-open");listing=driver.act("coverage-discover");assert "coverage_ui_case" in listing and "discovered" in listing,listing
            assert driver.act("editor-size",path=path)==size
            identity={"collector":"import-witness","version":"1","configuration":"fixture","scope":".","source_hashes":{path:hashlib.sha256(source.encode()).hexdigest()}}
            result=driver.act("coverage-import",report=f"SF:{path}\nDA:1,1\nDA:2,1\nend_of_record\n",identity=identity);assert "2 / 2 executable lines covered" in result,result
            driver.act("coverage-source", path=path)
            fixture.eventually(lambda: driver.act("caret-offset", path=path), len(source.splitlines(keepends=True)[0]))
            assert driver.act("editor-is-retained", path=path)
            fixture.eventually(lambda: driver.act("coverage-marks"), 2)
            coverage = driver.act("coverage-gutter")
            assert coverage["before"] == coverage["after"] and any(coverage["before"]["following"]), coverage
            icons = coverage["icons"]
            assert len({icon["x"] for icon in icons}) == 3 and len({icon["y"] for icon in icons}) == 1, icons
            labels = {icon["label"] for icon in icons}
            assert {"Inspect source tests: coverage_ui_case", "Debug lib.rs"} <= labels, labels
            assert any(label.startswith("Coverage ") for label in labels), labels
            driver.act("coverage-close")

            tests = driver.act("source-tests-gutter", name="coverage_ui_case")
            assert tests["before"] == tests["after"] and tests["icons"] == icons, tests
            assert "coverage_ui_case" in tests["text"], tests
            assert driver.act("editor-size", path=path) == size
            driver.act("source-tests-close")
            driver.act("append",path=path,text="// unsaved coverage draft\n");fixture.eventually(lambda:driver.act("coverage-marks"),0);draft=driver.act("search-source",path=path)
            driver.act("coverage-open");driver.act("coverage-source",path=path,stale=True);assert "Source changed" in driver.act("coverage-state")
            driver.act("coverage-close");assert driver.act("search-source",path=path)==draft;assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            print("PASS: actual runner discovery, report import, exact/stale navigation and retained editor geometry/draft")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
