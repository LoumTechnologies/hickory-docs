#!/usr/bin/env python3
"""Read account quotas through the native HTTP API using a disposable CLI fixture.

python3 scripts/test-account-usage.py target/debug/examples/python_agent_ui_fixture
No real credentials, sign-in, inference, reset redemption or credit purchases.
"""
import json
import os
from pathlib import Path
import select
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request

# Contract: .hickory-source/guarantees/shell/quiet-inline-workspace/contract.md
FAKE = r'''
import json,sys
from pathlib import Path
for line in sys.stdin:
    message=json.loads(line)
    method=message.get('method')
    with (Path(__file__).parent/'methods.jsonl').open('a') as log: log.write(json.dumps(method)+'\n')
    if 'id' not in message: continue
    if method == 'initialize': result={}
    elif method == 'account/rateLimits/read':
        if (Path(__file__).parent/'unsupported').exists():
            print(json.dumps({'id':message['id'],'error':{'code':-32601,'message':'sensitive upstream error'}}),flush=True)
            continue
        result={'rateLimits':{'primary':{'usedPercent':20,'windowDurationMins':300,'resetsAt':1999999999}},
                'rateLimitResetCredits':{'availableCount':2,'credits':[{'id':'opaque-reset-secret'}]},
                'email':'private@example.invalid','credentials':'credential-secret'}
    else: raise RuntimeError('Unexpected operation: '+str(method))
    print(json.dumps({'id':message['id'],'result':result}),flush=True)
'''


def main():
    engine = Path(sys.argv[1]).resolve()
    with tempfile.TemporaryDirectory(prefix="hickory-usage-") as folder:
        root = Path(folder)
        (root / "note.md").write_text("# Account usage witness\n")
        binaries = root / "agents/node_modules/.bin"
        binaries.mkdir(parents=True)
        cli = binaries / "codex"
        cli.write_text("#!" + sys.executable + "\n" + FAKE)
        cli.chmod(0o700)
        environment = {key: value for key, value in os.environ.items() if not key.endswith("_API_KEY")}
        process = subprocess.Popen([str(engine), str(root)], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, env=environment)
        try:
            assert select.select([process.stdout], [], [], 30)[0], "Fixture did not start"
            origin = process.stdout.readline().strip()
            assert origin.startswith("http://127.0.0.1:"), origin
            url = origin + "/api/agents/codex-subscription/usage"
            with urllib.request.urlopen(url, timeout=35) as response:
                data = json.load(response)
            assert data["rateLimits"]["primary"]["usedPercent"] == 20, data
            assert data["rateLimitResetCredits"] == {"availableCount": 2}, data
            assert not any(value in json.dumps(data) for value in ["private@example", "credential-secret", "opaque-reset-secret"]), data
            (binaries / "unsupported").touch()
            try:
                urllib.request.urlopen(url, timeout=35)
                raise AssertionError("Unsupported quota RPC must not look like zero usage")
            except urllib.error.HTTPError as error:
                assert error.code == 503, error.code
                message = error.read().decode()
                assert "usage is unavailable" in message and "sensitive upstream" not in message, message
            methods = [json.loads(line) for line in (binaries / "methods.jsonl").read_text().splitlines()]
            assert set(methods) <= {"initialize", "initialized", "account/rateLimits/read"}, methods
            assert methods.count("account/rateLimits/read") == 2, methods
            print("PASS native HTTP: quota/reset count, credential redaction, unsupported RPC, no login/inference/billing/reset side effects")
        finally:
            process.terminate()
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill(); process.wait()


if __name__ == "__main__":
    main()
