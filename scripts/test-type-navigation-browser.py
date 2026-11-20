"""Read type relationships through the Simple Browser menus and real engine."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import socket
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def prepare_workspace(root, folder, clangd):
    if clangd:
        if not shutil.which("clangd"):
            raise RuntimeError("The genuine hierarchy witness requires installed clangd.")
        (root / ".git").mkdir()
        return {
            "hierarchy.cpp": '#include "base.hpp"\nstruct Mid : Root {};\nstruct Leaf : Mid {};\n',
            "base.hpp": "struct Root {};\n",
        }

    events = folder / "events.jsonl"
    command = [sys.executable, str(ROOT / "scripts/controlled-action-provider.py"), "python", str(events)]
    (root / ".hick-lsp.json").write_text(json.dumps({"servers": {"python": {"command": command}}}))
    return {"a.py": "value = 1\n"}


def query_relationship(driver, path, offset, size, query, expected):
    deadline = time.monotonic() + 20
    while True:
        result = driver.act("semantic-query", path=path, offset=offset, query=query)
        if expected in result["text"]:
            assert result["size"] == size, result
            return
        driver.act("semantic-close")
        assert time.monotonic() < deadline, result
        time.sleep(.1)


def exercise_relationships(driver, fixture, clangd, sources):
    path = "hierarchy.cpp" if clangd else "a.py"
    source = sources[path]
    offset = source.index("Mid") if clangd else 1
    driver.act("open", path=path)
    change = "// retained draft\n" if clangd else "# retained draft\n"
    driver.act("source-tests-edit", path=path, text=change)
    draft = driver.act("search-source", path=path)
    assert draft == source + change, draft
    driver.act("remember-editor", path=path)
    size = driver.act("editor-size", path=path)

    base = "Root · base.hpp:1:8" if clangd else "pythonBase · a.py:1:1"
    query_relationship(driver, path, offset, size, "Find supertypes", base)
    base_path = "base.hpp" if clangd else path
    driver.act("semantic-open", path=base_path, label=base)
    base_offset = sources[base_path].index("Root") if clangd else 0
    fixture.eventually(lambda: driver.act("caret-offset", path=base_path), base_offset)
    if clangd:
        driver.act("navigate-history", direction="back", path=path)
        fixture.eventually(lambda: driver.act("caret-offset", path=path), offset)

    derived = "Leaf · hierarchy.cpp:3:8" if clangd else "pythonDerived"
    query_relationship(driver, path, offset, size, "Find subtypes", derived)
    if clangd:
        driver.act("semantic-open", path=path, label=derived)
        fixture.eventually(lambda: driver.act("caret-offset", path=path), source.index("Leaf"))
    else:
        driver.act("semantic-close")

    assert driver.act("editor-is-retained", path=path)
    assert driver.act("editor-size", path=path) == size
    assert driver.act("search-source", path=path) == draft


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--clangd", action="store_true", help="Use installed clangd for C++ relationships")
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location("simple_fixture", ROOT / "scripts/test-simple-browser.py")
    fixture = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-type-navigation-") as temporary:
        folder = Path(temporary)
        root = folder / "workspace"
        root.mkdir()
        sources = prepare_workspace(root, folder, args.clangd)
        for path, source in sources.items():
            (root / path).write_text(source)
        environment = {**os.environ, "HICKORY_STATE_DIR": str(folder / "state")}
        processes = []
        driver = None
        try:
            with socket.socket() as reservation:
                reservation.bind(("127.0.0.1", 0))
                port = reservation.getsockname()[1]
            ui = f"http://127.0.0.1:{port}"
            engine_path = os.environ.get("HICKORY_SIMPLE_TEST_ENGINE", str(ROOT / "target/source-edit-focus/debug/examples/engine_client"))
            engine = fixture.start([engine_path, str(root), "window", ui], env=environment)
            processes.append(engine)
            vite = fixture.start(
                ["node", "node_modules/vite/bin/vite.js", "--config", "vite.simple.config.ts",
                 "--host", "127.0.0.1", "--port", str(port), "--strictPort"],
                cwd=ROOT / "apps/web", env={**environment, "HICKORY_API_ORIGIN": fixture.origin(engine)},
            )
            processes.append(vite)
            driver = fixture.BrowserDriver(fixture.origin(vite) + "/simple.html")
            exercise_relationships(driver, fixture, args.clangd, sources)
            for path, source in sources.items():
                assert (root / path).read_bytes() == source.encode()
            if not args.clangd:
                events = [json.loads(line) for line in (folder / "events.jsonl").read_text().splitlines()]
                assert {item.get("type") for item in events} >= {
                    "typeHierarchy/supertypes", "typeHierarchy/subtypes",
                }, events
            provider = "genuine clangd" if args.clangd else "controlled child"
            print(f"PASS {provider} type navigation, exact name positions, retained draft/editor and unchanged disk")
        finally:
            if driver:
                driver.close()
            for process in reversed(processes):
                fixture.stop(process)


if __name__ == "__main__":
    main()
