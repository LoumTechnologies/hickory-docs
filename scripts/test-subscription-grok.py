#!/usr/bin/env python3
"""Drive the native engine's subscription API with a fake native Grok executable.

Build: cargo build --offline --locked -p hickory-cli --example python_agent_ui_fixture
Run: python3 scripts/test-subscription-grok.py --engine target/debug/examples/python_agent_ui_fixture --python-runtime apps/agent/.venv/bin/python
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
FAKE_GROK = r'''
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc,char **argv) {
 int inspect=0,agent=0,login=0;
 if (getenv("XAI_API_KEY") || getenv("GROK_MODELS_BASE_URL") || !getenv("GROK_DISABLE_API_KEY_AUTH") || strcmp(getenv("GROK_DISABLE_API_KEY_AUTH"),"true")) return 90;
 for(int i=1;i<argc;i++) {
  if(!strcmp(argv[i],"inspect")) inspect=1;
  if(!strcmp(argv[i],"stdio")) agent=1;
  if(!strcmp(argv[i],"login")) login=1;
  if(!strcmp(argv[i],"--prompt-file")) return 99; /* No inference after signed-out auth. */
 }
 if(inspect) { puts("{\"loginPolicy\":{\"apiKeyAuthDisabled\":true},\"projectInstructions\":[],\"hooks\":[],\"skills\":[],\"agents\":[],\"plugins\":[],\"mcpServers\":[],\"lspServers\":[]}");return 0; }
 if(login) return 1; /* Browser/network failure fixture. */
 if(!agent) return 91;
 char line[16384];
 while(fgets(line,sizeof(line),stdin)) {
  char *id=strstr(line,"\"id\":");
  if(!id) continue;
  if(!strstr(line,"\"initialize\"")) return 92; /* No API, session, tool or purchase RPC. */
  printf("{\"id\":%ld,\"result\":{\"authMethods\":[{\"id\":\"grok.com\"}]}}\n",strtol(id+5,0,10)); fflush(stdout);
 }
 return 0;
}
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
        grok = bin_path/'grok'
        fixture_source = root/'grok.c'
        fixture_source.write_text(FAKE_GROK)
        subprocess.run(['/usr/bin/cc', str(fixture_source), '-o', str(grok)], check=True)
        # A fixture credential home contains no real credentials and is never
        # populated from the user's Grok files. Canonicalization still runs.
        credentials = Path.home()/('.hickory-subscription-fixture-'+root.name)
        credentials.mkdir(mode=0o700)
        environment = {k:v for k,v in os.environ.items() if not k.endswith('_API_KEY')}
        environment['GROK_AUTH_PATH'] = str(credentials/'auth.json')
        environment['XAI_API_KEY'] = 'fixture-key-must-never-be-inherited'
        environment['GROK_MODELS_BASE_URL'] = 'https://invalid.example/never-called'
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

            status, account = request('GET','/api/agents/grok-subscription')
            assert status == 200 and account['ready'] is False, account
            assert account['balance'] is None and account['credit_cost'] is None
            status, login = request('POST','/api/agents/grok-subscription',{})
            assert status == 200 and login['login_pending'] is True, login
            deadline = time.monotonic()+5
            while time.monotonic() < deadline:
                status, account = request('GET','/api/agents/grok-subscription')
                if not account.get('login_pending'): break
                time.sleep(0.05)
            assert status == 200 and 'failed or timed out' in account['error'], account
            status, settings = request('PUT','/api/agents/python',{'python':str(runtime)})
            assert status == 200, settings
            # The native fixture starts with exactly one document.
            status, projects = request('GET','/api/projects')
            project = projects[0]['id']
            status, docs = request('GET','/api/projects/'+project+'/docs')
            entries = docs if isinstance(docs,list) else docs['docs']
            document = next(d['id'] for d in entries if d.get('name') == 'note.md' or d.get('path','').endswith('note.md'))
            path = '/api/docs/'+document+'/agent'
            status, run = request('POST',path,{'prompt':'Use subscription Grok','backend':'python','provider':'grok-subscription','model':''})
            assert status == 202, run
            deadline = time.monotonic()+30
            while time.monotonic() < deadline:
                _, listing = request('GET',path+'/turns')
                if listing['turns'] and listing['turns'][-1]['status'] != 'running': break
                time.sleep(0.05)
            turn = listing['turns'][-1]
            assert turn['provider'] == 'grok-subscription' and turn['backend'] == 'python', turn
            assert turn['model'] == 'grok-build', turn
            assert turn['status'] == 'error' and 'Sign in with Grok' in turn['error'], turn
            assert turn['usage'] is None and listing['totals']['usd'] is None, listing
            assert (root/'note.md').read_text() == 'Original human input\n'
            process.terminate(); process.wait(timeout=5)
            process = subprocess.Popen([str(engine),str(root)], stdout=subprocess.PIPE,
                                       stderr=subprocess.DEVNULL, text=True, env=environment)
            assert select.select([process.stdout],[],[],30)[0], 'restarted engine address'
            base = process.stdout.readline().strip()
            _, restored = request('GET',path+'/turns')
            assert restored['provider'] == 'grok-subscription', restored
            assert restored['model'] == 'grok-build', restored
            # An omitted or empty provider must retain subscription billing.
            status, rerun = request('POST',path,{'prompt':'Still subscription','backend':'python','provider':''})
            assert status == 202, rerun
            deadline = time.monotonic()+30
            while time.monotonic() < deadline:
                _, listing = request('GET',path+'/turns')
                retried = next((turn for turn in listing['turns'] if turn['id'] == rerun['session_id']),None)
                if retried and retried['status'] != 'running': break
                time.sleep(0.05)
            assert retried['provider'] == 'grok-subscription' and 'Sign in with Grok' in retried['error'], retried
            print('PASS native HTTP: subscription selection, signed-out state, native login failure, authentication failure, unknown charges, retained selection after restart and no billing fallback.')
        finally:
            process.terminate()
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill(); process.wait()
            credentials.rmdir()


if __name__ == '__main__':
    main()
