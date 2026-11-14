"""Isolated-engine protocol witness with controlled child providers."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-provider-actions-") as temporary:
        folder=Path(temporary);root=folder/"workspace";root.mkdir();events=folder/"events.jsonl";gate=folder/"release"
        sources={"a.py":"value = 1\n","b.ts":"value = 2;\n"}
        for path,text in sources.items():(root/path).write_text(text)
        provider=ROOT/"scripts/controlled-action-provider.py"
        (root/".hick-lsp.json").write_text(json.dumps({"servers":{language:{"command":[sys.executable,str(provider),language,str(events),str(gate)]} for language in ["python","typescript"]}}))
        engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window","http://127.0.0.1:1"],env={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")})
        try:
            origin=fixture.origin(engine);subprocess.run(["node",str(ROOT/"scripts/provider-actions-driver.mjs"),origin,str(events),str(gate)],check=True,timeout=90)
            observed=[json.loads(line) for line in events.read_text().splitlines()];assert [item.get("resolved") for item in observed]==["python","python"],observed
            for path,text in sources.items():assert (root/path).read_text()==text,"Resolution wrote source"
        finally:fixture.stop(engine)
if __name__=="__main__":main()
