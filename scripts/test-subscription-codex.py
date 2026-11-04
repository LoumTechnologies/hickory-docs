#!/usr/bin/env python3
"""Drive the native engine's subscription API with a fake Codex executable.

Build: cargo build --offline --locked -p hickory-cli --example python_agent_ui_fixture
Run: python3 scripts/test-subscription-codex.py --engine target/debug/examples/python_agent_ui_fixture --python-runtime apps/agent/.venv/bin/python
No account sign-in or paid provider calls are made.
"""
import argparse
import json
import os
from pathlib import Path
import select
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

# Contract: .hickory-source/guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model/contract.md
FAKE_CODEX = r'''
import json,sys
from pathlib import Path
with (Path(__file__).parent/'calls.jsonl').open('a') as calls:
    calls.write(json.dumps(sys.argv[1:])+'\n')
if sys.argv[1:3] == ['login','status']:
    print('Not logged in',file=sys.stderr)
    sys.exit(1)
if sys.argv[1] != 'app-server':
    raise RuntimeError('Unexpected Codex inference after failed authentication')
for line in sys.stdin:
    m=json.loads(line)
    if 'id' not in m: continue
    method=m['method']
    if method == 'initialize': result={}
    elif method == 'account/read': result={'account':None}
    elif method == 'account/login/start':
        assert m['params'] == {'type':'chatgpt'}
        result={'type':'chatgpt','loginId':'fixture','authUrl':'https://auth.openai.com/fixture'}
    else: raise RuntimeError('Unexpected account/billing operation: '+method)
    print(json.dumps({'id':m['id'],'result':result}),flush=True)
    if method == 'account/login/start':
        print(json.dumps({'method':'account/login/completed','params':{'success':False,'error':'fixture network failure'}}),flush=True)
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--engine', type=Path, required=True)
    parser.add_argument('--python-runtime', type=Path, required=True)
    args = parser.parse_args()
    engine = args.engine.absolute()
    runtime = args.python_runtime.absolute()
    if sys.platform != 'darwin':
        raise RuntimeError('This subscription transport currently requires macOS')
    with tempfile.TemporaryDirectory(prefix='hickory-subscription-system-') as temporary:
        root = Path(temporary)
        (root/'note.md').write_text('Original human input\n')
        bin_path = root/'agents/node_modules/.bin'
        bin_path.mkdir(parents=True)
        codex = bin_path/'codex'
        codex.write_text('#!'+sys.executable+'\n'+FAKE_CODEX)
        codex.chmod(0o700)
        # A fixture credential home contains no real credentials and is never
        # populated from the user's Codex files. Canonicalization still runs.
        credentials = Path.home()/('.hickory-subscription-fixture-'+root.name)
        credentials.mkdir(mode=0o700)
        environment = {k:v for k,v in os.environ.items() if not k.endswith('_API_KEY')}
        environment['CODEX_HOME'] = str(credentials)
        process = subprocess.Popen([str(engine),str(root)], stdout=subprocess.PIPE,
                                   stderr=subprocess.DEVNULL, text=True, env=environment)
        try:
            if not select.select([process.stdout],[],[],30)[0]:
                raise RuntimeError('Native fixture did not report its HTTP address')
            base = process.stdout.readline().strip()
            assert base.startswith('http://127.0.0.1:'), base

            def request(method, path, body=None):
                encoded = None if body is None else json.dumps(body).encode()
                req = urllib.request.Request(base+path, encoded, {'Content-Type':'application/json'}, method=method)
                try:
                    with urllib.request.urlopen(req,timeout=30) as response:
                        return response.status,json.load(response)
                except urllib.error.HTTPError as error:
                    return error.code,json.load(error)

            status, account = request('GET','/api/agents/codex-subscription')
            assert status == 200 and account['ready'] is False, account
            assert account['balance'] is None and account['credit_cost'] is None
            status, login = request('POST','/api/agents/codex-subscription',{})
            assert status == 200 and login['auth_url'] == 'https://auth.openai.com/fixture', login
            status, account = request('GET','/api/agents/codex-subscription')
            assert status == 200 and 'did not complete' in account['error'], account
            status, settings = request('PUT','/api/agents/python',{'python':str(runtime)})
            assert status == 200, settings
            # The native fixture starts with exactly one document.
            status, projects = request('GET','/api/projects')
            project = projects[0]['id']
            status, docs = request('GET','/api/projects/'+project+'/docs')
            entries = docs if isinstance(docs,list) else docs['docs']
            document = next(d['id'] for d in entries if d.get('name') == 'note.md' or d.get('path','').endswith('note.md'))
            path = '/api/docs/'+document+'/agent'
            status, run = request('POST',path,{'prompt':'Use subscription Codex','backend':'python','provider':'codex-subscription','model':''})
            assert status == 202, run
            deadline = time.monotonic()+30
            while time.monotonic() < deadline:
                _, listing = request('GET',path+'/turns')
                if listing['turns'] and listing['turns'][-1]['status'] != 'running': break
                time.sleep(0.05)
            turn = listing['turns'][-1]
            assert turn['provider'] == 'codex-subscription' and turn['backend'] == 'python', turn
            assert turn['model'] == 'gpt-6.1-sol', turn
            assert turn['status'] == 'error' and 'Sign in with ChatGPT' in turn['error'], turn
            assert turn['usage'] is None and listing['totals']['usd'] is None, listing
            calls = [json.loads(line) for line in (bin_path/'calls.jsonl').read_text().splitlines()]
            assert sum(call[:2] == ['login','status'] for call in calls) == 1, calls
            assert not any('exec' in call for call in calls), calls
            assert (root/'note.md').read_text() == 'Original human input\n'
            process.terminate(); process.wait(timeout=5)
            process = subprocess.Popen([str(engine),str(root)], stdout=subprocess.PIPE,
                                       stderr=subprocess.DEVNULL, text=True, env=environment)
            assert select.select([process.stdout],[],[],30)[0], 'restarted engine address'
            base = process.stdout.readline().strip()
            _, restored = request('GET',path+'/turns')
            assert restored['provider'] == 'codex-subscription', restored
            assert restored['model'] == 'gpt-6.1-sol', restored
            # An omitted or empty provider must retain subscription billing.
            status, rerun = request('POST',path,{'prompt':'Still subscription','backend':'python','provider':''})
            assert status == 202, rerun
            deadline = time.monotonic()+30
            while time.monotonic() < deadline:
                _, listing = request('GET',path+'/turns')
                retried = next((turn for turn in listing['turns'] if turn['id'] == rerun['session_id']),None)
                if retried and retried['status'] != 'running': break
                time.sleep(0.05)
            assert retried['provider'] == 'codex-subscription' and 'Sign in with ChatGPT' in retried['error'], retried
            print('PASS native HTTP: subscription selection, signed-out state, supported sign-in request, authentication failure, unknown charges, retained selection after restart and no billing fallback.')
        finally:
            process.terminate()
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill(); process.wait()
            credentials.rmdir()


if __name__ == '__main__':
    main()
