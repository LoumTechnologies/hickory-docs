"""Owned mounted UIKit parity witness. Requires an explicitly selected simulator."""
import argparse,hashlib,importlib.util,json,os,plistlib,shutil,socket,subprocess,sys,tempfile,urllib.error,urllib.parse,urllib.request,uuid
from contextlib import ExitStack, contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Event, Lock, Thread
ROOT=Path(__file__).resolve().parents[1]
TARGET = 'HickorySimpleMobileParityTests'
RECORDED_RUN_TARGET = 'HickorySimpleMobileRecordedRunTests'
SCHEMES = {TARGET: 'HickorySimpleMobileParity', RECORDED_RUN_TARGET: 'HickorySimpleMobileRecordedRun'}
PROPOSAL_DESTINATION = 'moved λ%2F?#.py'
MANAGED_MOVE_WORKFLOW = 'typescript-managed-move'
MANAGED_MOVE_DEFINITION_WORKFLOW = 'typescript-managed-move-definition'
MANAGED_MOVE_WORKFLOWS = {MANAGED_MOVE_WORKFLOW, MANAGED_MOVE_DEFINITION_WORKFLOW}
MANAGED_MOVE_DECLARATION = 'function relocate(name: string) { return "Hello " + name; }\n'
MANAGED_MOVE_LITERAL = MANAGED_MOVE_DECLARATION + 'console.log(relocate("Ada"));\n'
MANAGED_MOVE_IMPORT = 'import { relocate } from "./relocate";\n\n'
MANAGED_MOVE_MARKER = '// retained dirty\n'
TYPESCRIPT_PROVIDER_VERSIONS = {'typescript-language-server': '6.1.0', 'typescript': '6.0.3'}
STARTUP_PATH = 'nested/entry # %.py'
STARTUP_SOURCE = '# owned startup capability fixture\n' + ''.join(f'value_{i} = {i}\n' for i in range(180))
STARTUP_AUTHORITY_PATH = 'nested/entry # %.md'
GIT_REVIEW_BASELINE = 'print("committed baseline")\n'
GIT_REVIEW_SAVED = 'print("saved working tree")\n'
GIT_REVIEW_DRAFT_MARKER = '# retained local editor draft'
GIT_HISTORY_STABLE_LINE = '# stable committed source\n'
GIT_WORKFLOWS = {'git-review', 'git-history'}
PYTHON_CALLER = 'from helper import greeting\n\ngreeting("reader")\n'
PYTHON_DEFINITION = '# The provider should open the declaration below.\n\n\ndef greeting(name: str) -> str:\n    return name.upper()\n'
STARTUP_AUTHORITY_MODES = {
    'startup-authority': 'confirmed', 'startup-authority-unavailable': 'unavailable', 'startup-authority-retry': 'retry',
    'startup-authority-live': 'confirmed', 'startup-authority-live-retry': 'retry',
    'startup-authority-tree-first-retry': 'tree-first-retry',
}
LIVE_STARTUP_AUTHORITY = {'startup-authority-live', 'startup-authority-live-retry'}
RECORDED_CONSTRUCTION_SIDES = {'recorded-construction': 'assistant', 'recorded-construction-user': 'user'}
RECORDED_DEBUG_UI_SIDES = {'recorded-debug-ui-user': 'user', 'recorded-debug-ui-assistant': 'assistant'}
WORKFLOWS = {
    'editing': ('testMountedUIKitEditingAndSelectedRustDebug', ['native-paste-history-passed', 'coordinated-cleanup-history-passed', 'debug-paused', 'completed']),
    'workspace': ('testMountedSearchProblemsAndCollectedTests', ['workspace-navigation-problems-passed', 'workspace-tests-coverage-passed']),
    'quickfix': ('testMountedClangdQuickFixReviewsAndReplaysDraft', ['clangd-quickfix-preview-passed', 'clangd-quickfix-history-passed']),
    'typescript-extraction': ('testMountedTypeScriptExtractionReplaysExactDraft', ['typescript-extraction-exact-history-passed']),
    MANAGED_MOVE_WORKFLOW: ('testMountedManagedTypeScriptMovePublishesExactHistory', [
        'managed-typescript-move-preview-passed', 'managed-typescript-move-save-passed',
        'managed-typescript-move-inverse-passed', 'managed-typescript-move-redo-passed',
    ]),
    MANAGED_MOVE_DEFINITION_WORKFLOW: ('testMountedManagedTypeScriptDefinitionRestoresCarets', [
        'managed-typescript-move-definition-history-passed',
    ]),
    'recovery': ('testMountedConnectedReadingAndPrivateRecovery', ['connected-impact-passed', 'private-recovery-passed']),
    'proposal': ('testMountedHistoricalProposalRefusesApplication', ['historical-proposal-passed']),
    'proposal-apply': ('testMountedManagedProposalAppliesBothReviewedDrafts', ['managed-proposal-preview-passed', 'managed-proposal-history-passed']),
    'proposal-native-send': ('testNativeAgentSendAppliesBothDraftsAndReplaysHistory',
                             ['native-proposal-send-archived-exact-message', 'native-proposal-send-applied-and-replayed-two-drafts']),
    'proposal-save': ('testMountedManagedProposalPublishesReviewedResource', ['managed-resource-publication-passed', 'managed-resource-inverse-passed']),
    'git-review': ('testNativeGitReviewStagesOnlySavedBytesAndRetainsDraft', ['native-git-review-staged-saved-only-and-retained-draft']),
    'git-history': ('testNativeGitAttributionOpensImmutableCommitAndRetainsDraft',
                    ['native-git-history-read-immutable-commit-and-retained-draft']),
    'python-definition': ('testNativePythonDefinitionAndHistoryAcrossFiles', ['native-python-definition-and-back-forward']),
    'startup': ('testMountedRequestedFileRetainsEditingThroughDelayedDiagnostics', ['requested-source-before-provider', 'delayed-diagnostic-kept-editor-state']),
    'startup-authority': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'late-tree-authority-kept-editor-state']),
    'startup-authority-unavailable': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'late-tree-authority-kept-editor-state']),
    'startup-authority-retry': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'retried-tree-authority-in-existing-editor', 'late-tree-authority-kept-editor-state']),
    'startup-authority-live': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'late-tree-authority-kept-editor-state']),
    'startup-authority-live-retry': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'retried-tree-authority-in-existing-editor', 'late-tree-authority-kept-editor-state']),
    'startup-authority-tree-first-retry': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-guarded-authority', 'missing-startup-guard-kept-editor-read-only', 'retried-tree-authority-in-existing-editor', 'late-tree-authority-kept-editor-state']),
    'recorded-debug': ('testMountedRecordedCodeFencesPauseStepWatchAndContinue',
                       ['recorded-code-user-pause-step-watch-continue', 'recorded-code-assistant-pause-step-watch-continue']),
    'recorded-run': ('testRecordedUserAndAssistantRunsShowTerminalWithoutChangingSource',
                     ['recorded-run-user-terminal-kept-editor', 'recorded-run-assistant-terminal-kept-editor']),
    'recorded-construction': ('testRecordedConstructionShowsOnlyTheSelectedLocalTransition',
                              ['recorded-construction-assistant-read-immutable-local-transition']),
    'recorded-construction-user': ('testRecordedConstructionShowsOnlyTheSelectedLocalTransition',
                                   ['recorded-construction-user-read-immutable-local-transition']),
    'recorded-patch': ('testMountedRecordedPatchesReviewApplyAndUndo',
                       ['recorded-patch-user-review-apply-undo', 'recorded-patch-assistant-review-apply-undo']),
}

RECORDED_FILES = {
    'recorded-debug': ('sessions/agent.md',),
    'recorded-run': ('sessions/agent.md',),
    'recorded-patch': ('project.md', 'author.md', 'generated.py', 'review.txt', 'sessions/recorded-patch.md'),
}

for workflow, side in RECORDED_DEBUG_UI_SIDES.items():
    value = {'user': '42', 'assistant': '53'}[side]
    WORKFLOWS[workflow] = ('testRecordedFenceDebugThroughNativeGutter', [
        f'recorded-debug-ui-{side}-{phase}' for phase in
        ('breakpoint-tap', 'launch-tap', 'paused-4', 'stepped-5', f'watched-{value}', 'finished-immutable')
    ])
    RECORDED_FILES[workflow] = RECORDED_FILES['recorded-debug']

def recorded_fixture_path(folder, workflow, path):
    fixture = 'recorded-debug' if workflow == 'recorded-run' or workflow in RECORDED_DEBUG_UI_SIDES else workflow
    return folder/fixture/path

def workflow_target(workflow):
    ui_workflows = {'recorded-run', 'proposal-native-send', 'python-definition'} | RECORDED_CONSTRUCTION_SIDES.keys() | RECORDED_DEBUG_UI_SIDES.keys() | GIT_WORKFLOWS
    return RECORDED_RUN_TARGET if workflow in ui_workflows else TARGET

DEFINITION_TRACE = r'''
import json, signal, subprocess, sys
from pathlib import Path
from threading import Lock, Thread

event_file = Path(sys.argv[1])
provider = subprocess.Popen(sys.argv[2:], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
definitions = set()
guard = Lock()

def terminate(*_):
    raise SystemExit

signal.signal(signal.SIGTERM, terminate)

def forward(source, target, outgoing):
    try:
        while True:
            headers = []
            length = None
            while line := source.readline():
                headers.append(line)
                if not line.strip():
                    break
                key, value = line.split(b':', 1)
                if key.strip().lower() == b'content-length':
                    length = int(value.strip())
            if length is None:
                return
            body = source.read(length)
            if len(body) != length:
                return
            message = json.loads(body)
            with guard:
                identifier = message.get('id')
                if outgoing and message.get('method') == 'textDocument/definition':
                    definitions.add(identifier)
                    event = 'request'
                elif not outgoing and identifier in definitions and 'method' not in message:
                    definitions.remove(identifier)
                    event = 'response'
                else:
                    event = None
                if event:
                    with event_file.open('a') as receipt:
                        receipt.write(json.dumps({'event': event, 'message': message})+'\n')
            target.write(b''.join(headers)+body)
            target.flush()
    except BrokenPipeError:
        return

reader = Thread(target=forward, args=(provider.stdout, sys.stdout.buffer, False), daemon=True)
reader.start()
try:
    forward(sys.stdin.buffer, provider.stdin, True)
finally:
    provider.stdin.close()
    if provider.poll() is None:
        provider.terminate()
    try:
        provider.wait(timeout=4)
    except subprocess.TimeoutExpired:
        provider.kill()
        provider.wait()
'''

STARTUP_PROVIDER = r'''
import json
from pathlib import Path
import sys
import time

def receive():
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        if not line.strip():
            break
        key, value = line.decode().split(':', 1)
        headers[key.lower()] = value.strip()
    return json.loads(sys.stdin.buffer.read(int(headers['content-length'])))

def send(**message):
    body = json.dumps({'jsonrpc': '2.0', **message}).encode()
    sys.stdout.buffer.write(f'Content-Length: {len(body)}\r\n\r\n'.encode() + body)
    sys.stdout.buffer.flush()

def diagnostic(uri, version, report):
    send(method='textDocument/publishDiagnostics', params={
        'uri': uri, 'version': version, 'diagnostics': [{
            'message': report + ' startup diagnostic', 'severity': 2,
            'range': {'start': {'line': 1, 'character': 0}, 'end': {'line': 1, 'character': 4}},
        }],
    })

versions = {}
while message := receive():
    method, params = message.get('method'), message.get('params', {})
    if method == 'textDocument/didOpen':
        document = params['textDocument']
        versions[document['uri']] = document['version']
    if method == 'textDocument/didChange':
        document = params['textDocument']
        uri, version = document['uri'], document['version']
        if uri in versions and params['contentChanges'][-1]['text'].startswith('# sampled-draft'):
            old = versions.pop(uri)
            gate = Path(sys.argv[2])
            deadline = time.monotonic() + 45
            while not gate.exists():
                if time.monotonic() >= deadline:
                    raise TimeoutError('Startup test did not release diagnostics')
                time.sleep(0.025)
            diagnostic(uri, old, 'stale')
            diagnostic(uri, version, 'fresh')
            Path(sys.argv[1]).write_text(json.dumps({'old': old, 'current': version}))
    if method == 'exit':
        break
    if 'id' in message:
        send(id=message['id'], result={'capabilities': {'textDocumentSync': 1}} if method == 'initialize' else None)
'''

@contextmanager
def startup_gate(folder):
    release_path = '/' + uuid.uuid4().hex
    marker = folder/'startup-diagnostic-release'

    class Release(BaseHTTPRequestHandler):
        def do_POST(self):
            if self.path == release_path:
                marker.touch()
                self.send_response(204)
                self.end_headers()
            else:
                self.send_error(404)

        def log_message(self, *_detail):
            pass

    with ThreadingHTTPServer(('127.0.0.1', 0), Release) as server:
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            yield f'http://127.0.0.1:{server.server_port}{release_path}'
        finally:
            server.shutdown()
            thread.join()

def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT/'scripts'/f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def engine_command(workspace):
    with socket.socket() as reserve:
        reserve.bind(('127.0.0.1', 0))
        port = reserve.getsockname()[1]
    binary = os.environ.get('HICKORY_SIMPLE_TEST_ENGINE', str(ROOT/'target/source-edit-focus/debug/examples/engine_client'))
    return [binary, str(workspace), 'window', f'http://127.0.0.1:{port}']

def git_output(workspace, *arguments):
    return subprocess.check_output(['git', '-C', str(workspace), *arguments])

def observe_git_fixture(workspace, method, address, request, status, response):
    path = urllib.parse.urlsplit(address).path
    if status != 200:
        return {}
    if method == 'GET' and path == '/api/git/diff':
        return {'diff': json.loads(response)['diff']}
    if method == 'GET' and path in {'/api/blame', '/api/git/log', '/api/git/reading'}:
        return {'reading': json.loads(response)}
    if method == 'POST' and path in {'/api/git/stage', '/api/git/unstage'}:
        return {'paths': json.loads(request)['paths'],
                'index_sha256': hashlib.sha256(git_output(workspace, 'show', ':note.py')).hexdigest()}
    return {}

def check_git_review(workspace, trace, head):
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    mutations = [row for row in rows if row['method'] == 'POST' and row['path'] in {'/api/git/stage', '/api/git/unstage'}]
    assert [row['path'] for row in mutations] == ['/api/git/stage', '/api/git/unstage'], mutations
    for row, expected in zip(mutations, (GIT_REVIEW_SAVED, GIT_REVIEW_BASELINE)):
        assert row['status'] == 200 and row['paths'] == ['note.py'], row
        assert row['index_sha256'] == hashlib.sha256(expected.encode()).hexdigest(), row
    diffs = [row for row in rows if row['method'] == 'GET' and urllib.parse.urlsplit(row['path']).path == '/api/git/diff']
    sides = set()
    for row in diffs:
        query = urllib.parse.parse_qs(urllib.parse.urlsplit(row['path']).query)
        assert row['status'] == 200 and query['path'] == ['note.py'], row
        assert '-'+GIT_REVIEW_BASELINE in row['diff'] and '+'+GIT_REVIEW_SAVED in row['diff'], row
        assert GIT_REVIEW_DRAFT_MARKER not in row['diff'], row
        sides.update(query['staged'])
    assert sides == {'false', 'true'}, diffs
    assert git_output(workspace, 'rev-parse', 'HEAD') == head
    assert git_output(workspace, 'diff', '--cached', '--', 'note.py') == b''
    assert (workspace/'note.py').read_bytes() == GIT_REVIEW_SAVED.encode()
    assert not any(row['method'] in {'PUT', 'PATCH', 'DELETE'} for row in rows), rows
    return mutations

def check_git_history(workspace, trace, head, baseline, saved):
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    sha = head.decode().strip()
    def reads(path, field, value):
        selected = [row for row in rows if row['method'] == 'GET' and urllib.parse.urlsplit(row['path']).path == path]
        for row in selected:
            query = urllib.parse.parse_qs(urllib.parse.urlsplit(row['path']).query)
            assert row['status'] == 200 and query[field] == [value], row
        return [row['reading'] for row in selected]
    blame, = reads('/api/blame', 'path', 'note.py')
    assert blame['path'] == 'note.py'
    committed, uncommitted = blame['lines']
    first, second = saved.splitlines()
    assert committed['line'] == 1 and committed['text'] == first and not committed['uncommitted'] and committed['full_commit'] == sha, committed
    assert uncommitted['line'] == 2 and uncommitted['text'] == second and uncommitted['uncommitted'] and not uncommitted.get('full_commit'), uncommitted
    history, = reads('/api/git/log', 'path', 'note.py')
    assert history['repository'] and [commit['sha'] for commit in history['commits']] == [sha], history
    immutable = reads('/api/git/reading', 'sha', sha)
    assert len(immutable) == 2, immutable
    for reading in immutable:
        file, = reading['files']
        assert reading['sha'] == sha and file['path'] == 'note.py' and file['after'] == baseline and file['before'] == '' and not file['binary'], reading
    assert git_output(workspace, 'rev-parse', 'HEAD') == head
    assert git_output(workspace, 'diff', '--cached', '--', 'note.py') == b''
    assert (workspace/'note.py').read_bytes() == saved.encode()
    assert not any(row['method'] in {'PUT', 'PATCH', 'DELETE'} or row['path'] in {'/api/git/stage', '/api/git/unstage'} for row in rows), rows
    return sha

def traced_definition_command(folder, workflow, command):
    bridge = folder/f'{workflow}-trace.py'
    bridge.write_text(DEFINITION_TRACE)
    return [sys.executable, str(bridge), str(folder/f'{workflow}-provider.jsonl'), *command]

def provider_uri_path(uri):
    address = urllib.parse.urlsplit(uri)
    assert address.scheme == 'file' and not address.netloc, uri
    return Path(urllib.parse.unquote(address.path)).resolve()

def check_python_definition(workspace):
    trace = workspace.parent/'python-definition-provider.jsonl'
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    request, response = rows
    assert (request['event'], response['event']) == ('request', 'response'), rows
    sent, received = request['message'], response['message']
    assert sent['method'] == 'textDocument/definition' and sent['id'] == received['id'], rows
    params = sent['params']
    assert provider_uri_path(params['textDocument']['uri']) == workspace/'main.py', params
    position = params['position']
    assert position == {'line': 2, 'character': 0}, position
    result = received['result']
    location, = result if isinstance(result, list) else [result]
    uri = location.get('targetUri', location.get('uri'))
    target = location.get('targetSelectionRange', location.get('range'))['start']
    assert provider_uri_path(uri) == workspace/'helper.py' and target == {'line': 3, 'character': 4}, location
    return position, target

def check_managed_move_definition(folder):
    trace = folder/f'{MANAGED_MOVE_DEFINITION_WORKFLOW}-provider.jsonl'
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    requests = [row['message'] for row in rows if row['event'] == 'request']
    assert requests, rows
    for request in requests:
        params = request['params']
        assert request['method'] == 'textDocument/definition' and params['position'] == {'line': 2, 'character': 15}, request
        staged = provider_uri_path(params['textDocument']['uri'])
        assert staged.name == 'move.ts' and staged.parent.parent.name.startswith('hick-lsp-vfiles-'), request
    result, = [row['message']['result'] for row in rows
               if row['event'] == 'response' and row['message']['id'] == requests[-1]['id']]
    location, = result if isinstance(result, list) else [result]
    target = provider_uri_path(location.get('targetUri', location.get('uri')))
    position = location.get('targetSelectionRange', location.get('range'))['start']
    assert target == staged.with_name('relocate.ts') and position == {'line': 0, 'character': 16}, location
    return {'staged_source': staged.name, 'request_position': params['position'],
            'staged_definition': target.name, 'definition_position': position}

def managed_move_publication(files):
    saved = files['notes.md'].replace(MANAGED_MOVE_DECLARATION, MANAGED_MOVE_IMPORT, 1) + MANAGED_MOVE_MARKER
    return {**files, 'notes.md': saved, 'relocate.ts': 'export ' + MANAGED_MOVE_DECLARATION}

def prepare_workflow(workflow, workspace, python_env, slow_debug, recorded_fixtures=None):
    workspace.mkdir()
    if workflow in RECORDED_CONSTRUCTION_SIDES:
        side = RECORDED_CONSTRUCTION_SIDES[workflow]
        return load_script('test-simple-browser').prepare_recorded_construction(workspace, side), None
    if workflow == 'python-definition':
        installed = ROOT/'target/fenced-lsp-tools/basedpyright'
        provider = installed/'dist/pyright-langserver.js'
        node = shutil.which('node')
        assert node and provider.is_file(), 'The installed basedpyright fixture and local Node runtime are required'
        command = traced_definition_command(workspace.parent, workflow, [node, str(provider), '--stdio'])
        configuration = {'servers': {'python': {'command': command}}}
        files = {'main.py': PYTHON_CALLER, 'helper.py': PYTHON_DEFINITION, '.hick-lsp.json': json.dumps(configuration)}
        for path, source in files.items():
            (workspace/path).write_text(source)
        return files, None
    if workflow in GIT_WORKFLOWS:
        stable = GIT_HISTORY_STABLE_LINE if workflow == 'git-history' else ''
        note = workspace/'note.py'
        note.write_text(stable+GIT_REVIEW_BASELINE)
        git_output(workspace, 'init', '-q')
        git_output(workspace, 'add', '--', 'note.py')
        git_output(workspace, '-c', 'user.name=Owned fixture', '-c', 'user.email=fixture@example.invalid',
                   'commit', '-qm', 'Commit owned baseline')
        saved = stable+GIT_REVIEW_SAVED
        note.write_text(saved)
        return {'note.py': saved}, None
    if workflow == 'editing':
        (workspace/'src').mkdir()
        (workspace/'.git').mkdir()
        sentinel = workspace/'wrong-ios-sibling-ran'
        files = {
            'src/lib.rs': '// owned-ios-parity-source\n#[cfg(test)]\nmod cases {\n #[test]\n fn test_selected(){\n  let selected_value=41;\n  assert_eq!(selected_value,41); // owned-ios-parity-stop\n }\n #[test]\n fn test_sibling(){std::fs::write('+json.dumps(str(sentinel))+',b"wrong").unwrap();}\n}\n',
            'Cargo.toml': '[workspace]\n[package]\nname="owned-ios-parity"\nversion="0.1.0"\nedition="2024"\n',
        }
        if slow_debug:
            files['build.rs'] = 'fn main() { std::thread::sleep(std::time::Duration::from_secs(33)); }\n'
    elif workflow in RECORDED_FILES:
        files = {path: recorded_fixture_path(recorded_fixtures, workflow, path).read_bytes().decode('utf-8')
                 for path in RECORDED_FILES[workflow]}
        (workspace/'sessions').mkdir()
        sentinel = None
    elif workflow in STARTUP_AUTHORITY_MODES:
        (workspace/'nested').mkdir()
        files = {STARTUP_AUTHORITY_PATH: STARTUP_SOURCE, 'nested/sibling.txt': 'sibling source\n'}
        if workflow in LIVE_STARTUP_AUTHORITY:
            files['outside.txt'] = 'source outside the containing folder\n'
        sentinel = None
    elif workflow == 'startup':
        (workspace/'nested').mkdir()
        provider = workspace.parent/'startup-provider.py'
        provider.write_text(STARTUP_PROVIDER)
        events = workspace.parent/'startup-provider-events.json'
        files = {
            STARTUP_PATH: STARTUP_SOURCE,
            'nested/sibling.txt': 'sibling source\n',
            'outside.txt': 'outside the containing folder\n',
            '.hick-lsp.json': json.dumps({'servers': {'python': {'command': ['python3', str(provider), str(events), str(workspace.parent/'startup-diagnostic-release')]}}}),
        }
        sentinel = None
    elif workflow == 'workspace':
        collector = load_script('test-collector-hosts')
        (workspace/'.hick-cache').mkdir()
        (workspace/'.venv').symlink_to(python_env.resolve(), target_is_directory=True)
        sentinel = None
        files = {
            '.hickory-ios-workspace': 'owned-workspace-v1\n',
            'app.py': collector.APP,
            'test_app.py': collector.TEST,
            'diagnostics.c': 'int first(void) {return owned_problem_one;}\nint second(void) {return owned_problem_two;}\n',
            'pyproject.toml': '[project]\nname="ios-collector-fixture"\nversion="0.0.0"\n[tool.uv]\n[tool.pytest.ini_options]\njunit_family="xunit1"\n',
        }
    elif workflow == 'quickfix':
        files = {
            'main.cpp': 'int square(int value) { return value * value; }\nint main() { return square(3) }\n',
            '.hickory-ios-quickfix': 'owned-quickfix-v1\n',
        }
        sentinel = None
    elif workflow in {'typescript-extraction', *MANAGED_MOVE_WORKFLOWS}:
        tools = ROOT/'target/refactor-provider/node_modules'
        server = tools/'typescript-language-server/lib/cli.mjs'
        for name, version in TYPESCRIPT_PROVIDER_VERSIONS.items():
            assert json.loads((tools/name/'package.json').read_text())['version'] == version
        (workspace/'node_modules').mkdir()
        (workspace/'node_modules/typescript').symlink_to(tools/'typescript', target_is_directory=True)
        command = ['node', str(server), '--stdio']
        if workflow == MANAGED_MOVE_DEFINITION_WORKFLOW:
            command = traced_definition_command(workspace.parent, workflow, command)
        files = {
            '.hick-lsp.json': json.dumps({'servers': {'typescript': {'command': command}}}),
            'package.json': '{"type":"module"}',
            'tsconfig.json': '{"compilerOptions":{"strict":true,"target":"ES2022","ignoreDeprecations":"6.0"}}',
        }
        if workflow in MANAGED_MOVE_WORKFLOWS:
            namespace = '\x3chick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n'
            close = '\x3c/hick:doc>\n'
            files.update({
                '.hickory-managed-move-fixture': 'owned-managed-move-v1\n',
                'project.md': namespace + '\x3chick:project sources="notes.md"/>\n' + close,
                'notes.md': namespace + '\n# Original managed prose 🧪\n\n'
                            '\x3chick:file id="source" path="move.ts">\n' + MANAGED_MOVE_LITERAL
                            + '\x3c/hick:file>\n\nThe authored prose after the declaration remains.\n\n' + close,
                'move.ts': MANAGED_MOVE_LITERAL,
            })
            if workflow == MANAGED_MOVE_DEFINITION_WORKFLOW:
                files = managed_move_publication(files)
                files['.hickory-managed-move-fixture'] = 'owned-managed-move-definition-v1\n'
            subprocess.run(['git', '-C', str(workspace), 'init', '-q'], check=True)
        else:
            case = load_script('test-provider-refactor-host').CASES[0]
            files.update({case['path']: case['source'], '.hickory-refactor-fixture': 'owned-refactor-v1\n'})
        sentinel = None
    elif workflow == 'recovery':
        details = load_script('test-behavior-readings-browser').prepare(workspace)
        (workspace/'.venv').unlink()
        (workspace/'.venv').symlink_to(python_env.resolve(), target_is_directory=True)
        files = {name: (workspace/name).read_text() for name in ['project.md', 'source.md', 'behavior.md', 'a.py', 'test_a.py', 'test_undeclared.py', 'hickory-dogfood/behavior-map.json', 'hickory-dogfood/narrative-map.json']}
        sentinel = Path(details['sibling'])
    elif workflow in {'proposal-apply', 'proposal-save', 'proposal-native-send'}:
        load_script('test-agent-proposals-browser').prepare(workspace)
        files = {name: (workspace/name).read_text() for name in ['project.md', 'construction.md', 'note.py', 'generated.py', 'behavior.md']}
        if workflow != 'proposal-native-send':
            close = '\x3c/h:doc>\n'
            files['construction.md'] = files['construction.md'].removesuffix(close) + 'Retained author context for the mounted viewport.\n' * 100 + close
        sentinel = None
    else:
        archive = load_script('test-agent-proposal-persistence')
        files = {**archive.FILES, '.hickory-proposal-archive-fixture': 'owned-proposal-archive-v1\n'}
        sentinel = None
    for name, text in files.items():
        (workspace/name).write_text(text, encoding='utf-8')
    return files, sentinel

def run_workflow(workflow, device, seed, temporary_device, candidate, derived, destination,
                 python_env, slow_debug, receipt, recorded_fixtures=None):
    fixture = load_script('test-simple-browser')
    method, markers = WORKFLOWS[workflow]
    taps = workflow_target(workflow) == RECORDED_RUN_TARGET
    test_class = 'RecordedRunUITests' if taps else 'UIKitParityAcceptanceTests'
    name = workflow_target(workflow)+'/'+test_class+'/'+method
    with tempfile.TemporaryDirectory(prefix='ios-parity-') as temporary, ExitStack() as cleanup:
        folder = Path(temporary)
        workspace, state = folder/'workspace', folder/'state'
        files, sentinel = prepare_workflow(workflow, workspace, python_env, slow_debug, recorded_fixtures)
        environment = {**os.environ, 'HICKORY_STATE_DIR': str(state)}
        recorded = workflow in RECORDED_FILES or workflow in RECORDED_CONSTRUCTION_SIDES
        if workflow in ('recorded-debug', 'recorded-run') or workflow in RECORDED_DEBUG_UI_SIDES:
            environment['PATH'] = str(python_env.resolve()/'bin') + os.pathsep + environment['PATH']
        native_send = workflow == 'proposal-native-send'
        git_workflow = workflow in GIT_WORKFLOWS
        python_definition = workflow == 'python-definition'
        managed = load_script('test-agent-proposals-browser') if workflow in {'proposal-apply', 'proposal-save', 'proposal-native-send'} else None
        if managed:
            settings, calls, tool_result = managed.provider_fixture(folder)
            environment['HICKORY_AGENT_TEST_UI_SETTINGS'] = str(settings)
            if native_send:
                environment['HICKORY_EXECUTOR'] = 'local'
            if workflow == 'proposal-save':
                edit = {'documentChanges': [{
                    'kind': 'rename', 'oldUri': 'hick:///note.py',
                    'newUri': 'hick:///' + urllib.parse.quote(PROPOSAL_DESTINATION, safe=''),
                }]}
            else:
                def replacement(line, start, end):
                    return {'range': {'start': {'line': line, 'character': start}, 'end': {'line': line, 'character': end}}, 'newText': '41'}
                edit = {'changes': {
                    'hick:///note.py': [replacement(0, 6, 7)],
                    'hick:///construction.md': [replacement(2, 12, 13)],
                }}
            calls.write_text(json.dumps({'intention': managed.intention(edit)}))
        command = engine_command(workspace)
        archive = load_script('test-agent-proposal-persistence') if workflow == 'proposal' else None
        metadata = {}
        if recorded:
            metadata['HICKORY_IOS_RECORDED_ROOT'] = str(workspace.resolve())
        if workflow in RECORDED_DEBUG_UI_SIDES:
            metadata['HICKORY_IOS_RECORDED_DEBUG_SIDE'] = RECORDED_DEBUG_UI_SIDES[workflow]
        if workflow in RECORDED_CONSTRUCTION_SIDES:
            metadata['HICKORY_IOS_CONSTRUCTION_SIDE'] = RECORDED_CONSTRUCTION_SIDES[workflow]
        if workflow == 'startup':
            metadata['HICKORY_IOS_STARTUP_PATH'] = STARTUP_PATH
            metadata['HICKORY_STARTUP_DIAGNOSTIC_GATE'] = cleanup.enter_context(startup_gate(folder))
        if python_definition:
            metadata.update(HICKORY_IOS_PYTHON_CALLER=PYTHON_CALLER, HICKORY_IOS_PYTHON_DEFINITION=PYTHON_DEFINITION)
        if workflow == 'editing' and slow_debug:
            metadata['HICKORY_IOS_PARITY_SLOW_DEBUG'] = '1'
            markers = [*markers, 'slow-debug-admission-passed']
        if archive:
            first = fixture.start(command, env=environment)
            try:
                original = archive.call(fixture.origin(first), 'POST', '/api/agent-proposals/prepare', archive.intention())
                assert original['durability']['retained'] and not original['historical']
            finally:
                archive.stop_engine(fixture, first, state)
            metadata = {'HICKORY_IOS_PROPOSAL_TOKEN': original['token'], 'HICKORY_IOS_PROPOSAL_FINGERPRINT': original['fingerprint']}
        authority_mode = STARTUP_AUTHORITY_MODES.get(workflow)
        live_authority = workflow in LIVE_STARTUP_AUTHORITY
        engine = fixture.start(command, env=environment) if live_authority or not authority_mode else None
        try:
            upstream = fixture.origin(engine) if engine else None
            if authority_mode:
                origin, controls, authority_calls, authority_wire = cleanup.enter_context(
                    startup_authority_fixture(workspace.resolve(), authority_mode, upstream))
                metadata.update(controls)
            else:
                origin = upstream
            if native_send:
                managed.api(upstream, 'POST', '/api/docs/workspace/agent/acp', {'backend': 'owned-proposal'})
                send_wire = receipt/'proposal-native-send-http.jsonl'
                origin = cleanup.enter_context(managed.trace_appkit_send(upstream, send_wire))
                metadata['HICKORY_IOS_PROPOSAL_PROMPT'] = managed.PROMPT
            if git_workflow:
                git_head = git_output(workspace, 'rev-parse', 'HEAD')
                baseline = git_output(workspace, 'show', 'HEAD:note.py').decode()
                git_wire = receipt/f'{workflow}-http.jsonl'
                transport = load_script('test-agent-proposals-browser')
                origin = cleanup.enter_context(transport.trace_appkit_send(
                    upstream, git_wire, observe=lambda *event: observe_git_fixture(workspace, *event)))
                metadata.update(HICKORY_IOS_GIT_BASELINE=baseline, HICKORY_IOS_GIT_SAVED=files['note.py'],
                                HICKORY_IOS_GIT_DRAFT_MARKER=GIT_REVIEW_DRAFT_MARKER,
                                HICKORY_IOS_GIT_COMMIT=git_head.decode().strip())
            if archive:
                catalogue = archive.call(origin, 'GET', '/api/agent-proposals')
                assert catalogue['workspace'] != original['workspace'], 'Expected a new owned engine lifetime'
            value = plistlib.loads(candidate.read_bytes())
            target, = test_targets(value, workflow_target(workflow))
            # The controlled startup and recorded cases mount an explicit workspace.
            host_origin = '' if authority_mode or recorded else origin
            for key in ['EnvironmentVariables', 'TestHostEnvironmentVariables']:
                target.setdefault(key, {}).update(HICKORY_IOS_PARITY_ORIGIN=origin, HICKORY_API_ORIGIN=host_origin, **metadata)
            prefix = 'last' if workflow == 'editing' else workflow
            result = receipt/f'{prefix}.xcresult'
            if result.exists():
                shutil.rmtree(result)
            log = receipt/f'{prefix}.log'
            configured = derived/'Build/Products/owned-ios-parity.xctestrun'
            configured.write_bytes(plistlib.dumps(value))
            try:
                with log.open('w') as output:
                    run(['xcodebuild', 'test-without-building', '-xctestrun', str(configured), '-destination', destination, '-parallel-testing-enabled', 'NO', '-maximum-concurrent-test-simulator-destinations', '1', '-only-testing:'+name, '-resultBundlePath', str(result), '-test-timeouts-enabled', 'YES', '-maximum-test-execution-time-allowance', '180'], stdout=output, stderr=subprocess.STDOUT)
                summary = json.loads(subprocess.check_output(['xcrun', 'xcresulttool', 'get', 'test-results', 'summary', '--path', str(result)]))
                (receipt/f'{prefix}-summary.json').write_text(json.dumps(summary, indent=2))
                assert summary['result'] == 'Passed' and summary['totalTestCount'] == 1 and summary['passedTests'] == 1 and summary['failedTests'] == summary['skippedTests'] == 0, summary
                expected = managed_move_publication(files) if workflow == MANAGED_MOVE_WORKFLOW else files
                for path, text in expected.items():
                    assert (workspace/path).read_bytes() == text.encode(), path
                if sentinel:
                    assert not sentinel.exists(), sentinel.name
                if workflow == 'workspace':
                    events = (workspace/'.hick-cache/executions').read_text().splitlines()
                    assert sorted(events) == ['negative', 'positive', 'positive'], events
                if workflow == 'startup':
                    events = json.loads((folder/'startup-provider-events.json').read_text())
                    assert events['old'] < events['current'], events
                if authority_mode:
                    check_startup_authority_calls(authority_mode, authority_calls)
                if workflow == 'recorded-run':
                    sessions = load_script('test-agent-proposals-browser').api(origin, 'GET', '/api/terminals')['sessions']
                    assert len(sessions) == len({item['id'] for item in sessions}) == 2
                    assert all(item['state'] == 'finished' and item['exit_code'] == 0
                               and item['title'] == 'Run recorded fence' for item in sessions), sessions
                    assert sorted(item['preview'] for item in sessions) == ['42', '53'], sessions
                if managed:
                    response = json.loads(tool_result.read_text())['result']
                    assert not response['isError'], response
                    prepared = json.loads(response['content'][0]['text'])
                    assert prepared['pending_review'] and not any(prepared[key] for key in ['applied', 'saved', 'checks_executed'])
                    retained = managed.api(origin, 'GET', '/api/agent-proposals/'+prepared['token'])
                    assert retained['fingerprint'] == prepared['fingerprint']
                    report = retained['host_application']
                    assert report['status'] == 'host-reported draft application' and not report['saved'] and not report['checks_executed']
                    if workflow == 'proposal-save':
                        assert not (workspace/PROPOSAL_DESTINATION).exists()
                        assert report['after_hashes'] == {
                            'note.py': 'missing',
                            PROPOSAL_DESTINATION: hashlib.sha256(files['note.py'].encode()).hexdigest(),
                        }
                    else:
                        assert report['after_hashes'] == {
                            'note.py': hashlib.sha256(b'value=41\n').hexdigest(),
                            'construction.md': hashlib.sha256(files['construction.md'].replace('generated = 1', 'generated = 41').encode()).hexdigest(),
                        }
                if native_send:
                    turns = managed.api(upstream, 'GET', '/api/docs/workspace/agent/turns')['turns']
                    turn, = turns
                    assert turn['prompt'] == managed.PROMPT and turn['status'] == 'ok', turn
                    managed.verify_native_send(Path(command[0]).resolve(), {path: text.encode() for path, text in files.items()},
                                               workspace, upstream, tool_result, wire=send_wire, log=log,
                                               receipt=receipt/'proposal-native-send-wire-receipt.json',
                                               scope='iOS simulator native keyboard, Send, sealed review, draft Apply and coordinated Edit menu Undo/Redo')
                if workflow == 'git-review':
                    index_receipts = check_git_review(workspace, git_wire, git_head)
                if workflow == 'git-history':
                    commit = check_git_history(workspace, git_wire, git_head, baseline, files['note.py'])
                if python_definition:
                    call_site, declaration = check_python_definition(workspace.resolve())
                observed = log.read_text()
                if live_authority:
                    check_live_startup_authority(workspace.resolve(), authority_wire, observed)
                for marker in markers:
                    assert observed.count('HickoryUIKitParity '+marker) == 1, marker
                payload = {
                    'workflow': workflow, 'device': device, 'seed_device': seed, 'temporary_device': temporary_device,
                    'test': name, 'summary': summary, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
                    'asserted_milestones': markers, 'keyboard_input_verified': False,
                    'physical_input_verified': False, 'accessibility_verified': False,
                }
                if workflow == 'proposal-save':
                    payload['saved_namespace_restored'] = True
                elif workflow in MANAGED_MOVE_WORKFLOWS:
                    payload['provider_versions'] = TYPESCRIPT_PROVIDER_VERSIONS
                    if workflow == MANAGED_MOVE_DEFINITION_WORKFLOW:
                        payload['definition'] = check_managed_move_definition(folder)
                        payload['source_unchanged'] = True
                    payload['saved_source_sha256'] = {
                        path: hashlib.sha256((workspace/path).read_bytes()).hexdigest()
                        for path in ['notes.md', 'relocate.ts', 'move.ts']
                    }
                    payload['generated_output_unchanged'] = True
                else:
                    payload['source_unchanged'] = True
                if sentinel:
                    payload['sibling_excluded'] = True
                if workflow == 'editing':
                    payload.update(native_paste_undo_redo_verified=True, coordinated_cleanup_undo_redo_verified=True)
                if taps:
                    payload['simulator_ui_taps_verified'] = True
                if native_send:
                    payload['keyboard_input_verified'] = True
                    payload['native_send_wire_receipt'] = 'proposal-native-send-wire-receipt.json'
                if git_workflow:
                    payload['keyboard_input_verified'] = True
                    payload['fixture_sha256'] = {'note.py': hashlib.sha256(files['note.py'].encode()).hexdigest()}
                if workflow == 'git-review':
                    payload['git_index_receipts'] = index_receipts
                if workflow == 'git-history':
                    payload['git_immutable_commit'] = commit
                if python_definition:
                    payload['keyboard_input_verified'] = True
                    installed = ROOT/'target/fenced-lsp-tools/basedpyright'
                    payload['provider'] = {'name': 'basedpyright', 'version': json.loads((installed/'package.json').read_text())['version'],
                                           'request_position': call_site, 'definition_position': declaration}
                if workflow == 'recorded-run':
                    payload['rendered_terminal_accessibility_verified'] = True
                if workflow in RECORDED_DEBUG_UI_SIDES:
                    payload['simulator_keyboard_input_verified'] = True
                    payload['recorded_side'] = RECORDED_DEBUG_UI_SIDES[workflow]
                if authority_mode:
                    payload['authority_responses'] = authority_calls
                if live_authority:
                    payload['engine_authority'] = authority_wire
                if recorded:
                    payload['fixture_sha256'] = {path: hashlib.sha256(text.encode()).hexdigest() for path, text in files.items()}
                (receipt/f'{prefix}-receipt.json').write_text(json.dumps(payload, indent=2))
            finally:
                configured.unlink(missing_ok=True)
        finally:
            if python_definition or workflow == MANAGED_MOVE_DEFINITION_WORKFLOW:
                trace = folder/f'{workflow}-provider.jsonl'
                if trace.exists():
                    shutil.copyfile(trace, receipt/trace.name)
            if archive:
                archive.stop_engine(fixture, engine, state)
            elif recorded or live_authority or native_send or git_workflow or python_definition or workflow in MANAGED_MOVE_WORKFLOWS:
                load_script('test-agent-proposal-persistence').stop_engine(fixture, engine, state)
            elif engine:
                fixture.stop(engine)
    print(f'PASS mounted UIKit {workflow}', flush=True)

def appkit_startup(receipt):
    fixture = load_script('test-simple-browser')
    name = 'NativeSourceIdentityTests/testMountedRequestedFileRetainsEditingThroughDelayedDiagnostics'
    receipt.mkdir(parents=True)
    with tempfile.TemporaryDirectory(prefix='appkit-startup-') as temporary, ExitStack() as cleanup:
        folder = Path(temporary)
        workspace = folder/'workspace'
        files, _ = prepare_workflow('startup', workspace, None, False)
        environment = {**os.environ, 'HICKORY_STATE_DIR': str(folder/'state')}
        engine = fixture.start(engine_command(workspace), env=environment)
        try:
            environment.update(HICKORY_APPKIT_STARTUP_ORIGIN=fixture.origin(engine), HICKORY_OPEN_FILE=STARTUP_PATH)
            environment['HICKORY_STARTUP_DIAGNOSTIC_GATE'] = cleanup.enter_context(startup_gate(folder))
            log = receipt/'appkit-startup.log'
            with log.open('w') as output:
                run(['swift', 'test', '--package-path', 'apps/apple-simple', '--scratch-path',
                     'target/apple-simple-spm', '--jobs', '2', '--filter', name],
                    env=environment, stdout=output, stderr=subprocess.STDOUT, timeout=600)
            observed = log.read_text()
            assert " testMountedRequestedFileRetainsEditingThroughDelayedDiagnostics]' passed" in observed
            assert 'Executed 1 test, with 0 failures' in observed
            for path, text in files.items():
                assert (workspace/path).read_bytes() == text.encode(), path
            events = json.loads((folder/'startup-provider-events.json').read_text())
            assert events['old'] < events['current'], events
            (receipt/'appkit-startup-receipt.json').write_text(json.dumps({
                'test': name, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
                'provider_versions': events, 'source_unchanged': True,
            }, indent=2))
        finally:
            fixture.stop(engine)
    print('PASS mounted AppKit startup', flush=True)

@contextmanager
def startup_authority_fixture(root, mode, engine=None):
    path = STARTUP_AUTHORITY_PATH
    released, lock, calls, wire = Event(), Lock(), [], []
    tree_first = mode == 'tree-first-retry'
    guard_requested, guard_released = Event(), Event()
    gate = '/' + uuid.uuid4().hex
    class Authority(BaseHTTPRequestHandler):
        def bytes_reply(self, status, body, content_type='application/json'):
            self.send_response(status)
            self.send_header('Content-Type', content_type)
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            if body:
                self.wfile.write(body)

        def reply(self, status, value=None):
            self.bytes_reply(status, json.dumps(value).encode() if value is not None else b'')

        def forward(self, request_path):
            if self.headers.get('Upgrade'):
                self.reply(426)
                return
            try:
                incoming = urllib.request.urlopen(engine + self.path, timeout=28)
            except urllib.error.HTTPError as error:
                incoming = error
            except urllib.error.URLError:
                self.reply(502, {'error': 'Owned authority relay could not reach its engine'})
                return
            with incoming:
                body, status = incoming.read(), incoming.status
                content_type = incoming.headers.get('Content-Type', 'application/octet-stream')
            event = None
            if status == 200 and content_type.startswith('application/json'):
                reading = json.loads(body)
                if request_path == '/api/startup-source':
                    event = {'event': 'startup-source', 'path': reading['path'], 'root_path': reading['root_path'],
                             'workspace': reading['workspace_id'], 'source_sha256': hashlib.sha256(reading['content'].encode()).hexdigest()}
                elif request_path == '/api/files':
                    event = {'event': 'tree-confirmed', 'root_path': reading['root_path'], 'workspace': reading['workspace_id']}
                elif request_path.startswith('/api/docs/') and isinstance(reading, dict) and reading.get('path') == path and reading.get('save_guard'):
                    event = {'event': 'guarded-source', 'route': request_path, 'path': reading['path'],
                             'source_sha256': hashlib.sha256(reading['source'].encode()).hexdigest(),
                             'guard_sha256': hashlib.sha256(reading['save_guard'].encode()).hexdigest()}
            if event:
                with lock:
                    wire.append({**event, 'status': status, 'engine': True})
                    calls.append(event['event'])
            self.bytes_reply(status, body, content_type)

        def do_GET(self):
            request = urllib.parse.urlsplit(self.path)
            if request.path == '/api/files':
                if not tree_first and not released.wait(20):
                    self.reply(504, {'error': 'Owned authority fixture timed out'})
                    return
                with lock:
                    unavailable = mode == 'unavailable' or (mode == 'retry' and 'tree-unavailable' not in calls)
                    if unavailable:
                        calls.append('tree-unavailable')
                        if engine:
                            wire.append({'event': 'tree-unavailable', 'status': 503, 'engine': False})
                if unavailable:
                    self.reply(503, {'error': 'Owned file authority unavailable'})
                    return
            if engine:
                self.forward(request.path)
            elif request.path == '/api/startup-source' and urllib.parse.parse_qs(request.query) == {'path': [path]}:
                if tree_first and not released.wait(20):
                    self.reply(504, {'error': 'Owned startup text was not released'})
                    return
                calls.append('startup-source')
                self.reply(200, {'version': 1, 'path': path, 'content': STARTUP_SOURCE,
                                 'root_path': str(root), 'workspace_id': 'owned-workspace'})
            elif request.path == '/api/files':
                calls.append('tree-confirmed')
                self.reply(200, {'root': str(root), 'root_path': str(root), 'workspace_id': 'owned-workspace', 'tree': [
                    {'name': 'nested', 'path': 'nested', 'dir': True, 'children': [
                        {'name': path.split('/')[-1], 'path': path, 'dir': False, 'doc_id': 'owned-md'},
                        {'name': 'sibling.txt', 'path': 'nested/sibling.txt', 'dir': False},
                    ]},
                ]})
            elif request.path == '/api/docs/owned-md':
                if tree_first and 'guard-missing' not in calls:
                    guard_requested.set()
                    if not guard_released.wait(20):
                        self.reply(504, {'error': 'Owned guarded read was not released'})
                        return
                    calls.append('guard-missing')
                    self.reply(200, {'source': STARTUP_SOURCE, 'save_guard_reason': 'Source permission is unconfirmed'})
                else:
                    calls.append('guarded-source')
                    self.reply(200, {'source': STARTUP_SOURCE, 'save_guard': 'owned-startup-guard'})
            else:
                self.reply(404, {'error': 'Optional capabilities are unavailable in this fixture'})

        def do_POST(self):
            if self.path == gate:
                if tree_first and not guard_requested.wait(10):
                    self.reply(504, {'error': 'The guarded source was not requested before startup text'})
                    return
                released.set()
                self.reply(204)
            elif tree_first and self.path == gate + '/guard':
                guard_released.set()
                self.reply(204)
            else:
                self.reply(404, {'error': 'Unexpected mutation'})

        def log_message(self, *_detail):
            pass

    with ThreadingHTTPServer(('127.0.0.1', 0), Authority) as server:
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        origin = f'http://127.0.0.1:{server.server_port}'
        try:
            controls = {
                'HICKORY_STARTUP_AUTHORITY_GATE': origin + gate,
                'HICKORY_STARTUP_AUTHORITY_MODE': mode,
                'HICKORY_OPEN_FILE': path,
            }
            if engine:
                controls['HICKORY_IOS_STARTUP_REAL_ROOT'] = str(root)
            yield origin, controls, calls, wire
        finally:
            released.set()
            guard_released.set()
            server.shutdown()
            thread.join()

def appkit_startup_authority(receipt):
    receipt.mkdir(parents=True)
    name = 'NativeSourceIdentityTests/testMountedRequestedTextPrecedesTreeAuthority'
    with tempfile.TemporaryDirectory(prefix='appkit-authority-') as temporary:
        root = Path(temporary).resolve()
        source = root/STARTUP_AUTHORITY_PATH
        source.parent.mkdir()
        source.write_text(STARTUP_SOURCE)
        for mode in ('confirmed', 'unavailable', 'retry', 'tree-first-retry'):
            with startup_authority_fixture(root, mode) as (origin, controls, calls, _):
                env = {**os.environ, **controls, 'HICKORY_APPKIT_STARTUP_AUTHORITY_ORIGIN': origin}
                log = receipt/f'{mode}.log'
                with log.open('w') as output:
                    run(['swift', 'test', '--package-path', 'apps/apple-simple', '--scratch-path',
                         'target/apple-simple-spm', '--jobs', '2', '--filter', name],
                        env=env, stdout=output, stderr=subprocess.STDOUT, timeout=600)
                assert " testMountedRequestedTextPrecedesTreeAuthority]' passed" in log.read_text()
                check_startup_authority_calls(mode, calls)
                assert source.read_bytes() == STARTUP_SOURCE.encode()
                (receipt/f'{mode}-receipt.json').write_text(json.dumps({
                    'test': name, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(), 'responses': calls,
                    'source_unchanged': True,
                }, indent=2))
    print('PASS mounted AppKit provisional source and later authority', flush=True)

def check_startup_authority_calls(mode, calls):
    expected = {
        'confirmed': ['startup-source', 'tree-confirmed', 'guarded-source'],
        'unavailable': ['startup-source', 'tree-unavailable'],
        'retry': ['startup-source', 'tree-unavailable', 'tree-confirmed', 'guarded-source'],
        'tree-first-retry': ['tree-confirmed', 'startup-source', 'guard-missing', 'tree-confirmed', 'guarded-source'],
    }[mode]
    assert calls == expected, calls

def check_live_startup_authority(root, wire, log):
    raw, tree, guarded = [row for row in wire if row['engine']]
    source = hashlib.sha256(STARTUP_SOURCE.encode()).hexdigest()
    assert raw['path'] == guarded['path'] == STARTUP_AUTHORITY_PATH
    assert raw['root_path'] == tree['root_path'] == str(root)
    assert raw['workspace'] == tree['workspace'] and raw['workspace']
    assert raw['source_sha256'] == guarded['source_sha256'] == source
    assert guarded['route'].startswith('/api/docs/')
    mounted = f"live-startup-source-sha256={source} guard-sha256={guarded['guard_sha256']} workspace={raw['workspace']}"
    assert log.count(mounted) == 1, 'The mounted editor did not adopt the engine guard for these exact startup bytes'

def run(args,**kwargs):
 return subprocess.run(args,check=True,text=True,**kwargs)
def require_space(minimum):
 available=shutil.disk_usage(ROOT).free
 if available<minimum*1024*1024:raise RuntimeError(f'Need {minimum}MiB free before simulator/build work; available {available//(1024*1024)}MiB')
def test_targets(value, name=TARGET):
 if isinstance(value,dict):
  if value.get('BlueprintName')==name or value.get('TestableName')==name:yield value
  else:
   for child in value.values():yield from test_targets(child,name)
 elif isinstance(value,list):
  for child in value:yield from test_targets(child,name)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--device');parser.add_argument('--build',action='store_true');parser.add_argument('--boot',action='store_true');parser.add_argument('--temporary-device',action='store_true');parser.add_argument('--minimum-free-mib',type=int,default=2048)
 parser.add_argument('--appkit-startup', action='store_true', help='Run only the mounted macOS startup workflow')
 parser.add_argument('--appkit-startup-authority', action='store_true', help='Hold file tree authority while mounting macOS source')
 parser.add_argument('--workflow', choices=WORKFLOWS, action='append', help='Repeat to use one simulator for several focused cases')
 parser.add_argument('--python-env', type=Path, default=ROOT/'target/thread-debug-env')
 parser.add_argument('--recorded-fixtures', type=Path, help='Explicit shared fixture directory for recorded Agent journeys')
 parser.add_argument('--receipt-dir', type=Path, help='Create a new directory for this run and its lifecycle receipt')
 parser.add_argument('--slow-debug', action='store_true', help='Delay the owned Rust fixture to exercise staged admission')
 args=parser.parse_args()
 if args.appkit_startup or args.appkit_startup_authority:
  if args.device or args.build or args.boot or args.temporary_device or args.workflow or args.slow_debug or not args.receipt_dir:
   parser.error('AppKit startup cases require --receipt-dir and no simulator options')
  if args.appkit_startup and args.appkit_startup_authority:
   parser.error('Choose one AppKit startup case')
  (appkit_startup if args.appkit_startup else appkit_startup_authority)(args.receipt_dir)
  return
 if not args.device:parser.error('--device selects the simulator for the hosted iOS workflow')
 recorded_workflows = RECORDED_FILES.keys() & set(args.workflow or [])
 if RECORDED_CONSTRUCTION_SIDES.keys() & set(args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Recorded construction requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if 'proposal-native-send' in (args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Native proposal Send requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if GIT_WORKFLOWS.intersection(args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Native Git review requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if 'python-definition' in (args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Native Python definition requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if MANAGED_MOVE_WORKFLOWS.intersection(args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Managed TypeScript Move requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if LIVE_STARTUP_AUTHORITY.intersection(args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Live startup authority requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if recorded_workflows:
  if not args.recorded_fixtures or not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
   parser.error('Recorded workflows require --recorded-fixtures and an explicit HICKORY_SIMPLE_TEST_ENGINE')
  if any(not recorded_fixture_path(args.recorded_fixtures, workflow, path).is_file()
         for workflow in recorded_workflows for path in RECORDED_FILES[workflow]):
   parser.error('The recorded fixture directory is missing an expected source')
 if args.slow_debug and args.workflow and 'editing' not in args.workflow:
  parser.error('--slow-debug requires the editing workflow')
 if args.minimum_free_mib<128:parser.error('Minimum free space must remain at least128MiB')
 require_space(args.minimum_free_mib)
 devices=json.loads(subprocess.check_output(['xcrun','simctl','list','devices','-j']))
 existing_ids={d['udid'].lower()for group in devices['devices'].values()for d in group}
 selected=[(runtime,d)for runtime,group in devices['devices'].items()for d in group if d['udid']==args.device and d.get('isAvailable')is True]
 if len(selected)!=1:raise RuntimeError('Choose exactly one installed available simulator UDID')
 runtime,seed=selected[0]
 if not runtime.startswith('com.apple.CoreSimulator.SimRuntime.iOS-'):raise RuntimeError('Choose an installed iOS simulator')
 if args.temporary_device and not args.boot:parser.error('--temporary-device requires explicit --boot after resource coordination')
 receipt=args.receipt_dir or ROOT/'target/ios-parity'
 receipt.mkdir(parents=True,exist_ok=args.receipt_dir is None)
 device=args.device;owned_device=None;booted_here=False;failures=[]
 try:
  state=seed['state']
  if args.temporary_device:
   returned=subprocess.check_output(['xcrun','simctl','create','HickoryParity-'+uuid.uuid4().hex,seed['deviceTypeIdentifier'],runtime],text=True).strip()
   uuid.UUID(returned)
   if returned.lower()in existing_ids:raise RuntimeError('Create returned a preexisting simulator identity; no boot or deletion is authorized')
   device=returned;owned_device=device;state='Shutdown'
  if state!='Booted':
   if not args.boot:raise RuntimeError('Selected simulator is shutdown; coordinate resources before explicitly using --boot')
   run(['xcrun','simctl','boot',device]);booted_here=True
  run(['xcrun','simctl','bootstatus',device,'-b'])
  derived=ROOT/'target/apple-simple-derived';destination='platform=iOS Simulator,id='+device
  workflows=list(dict.fromkeys(args.workflow or ['editing']))
  names=list(dict.fromkeys(workflow_target(workflow) for workflow in workflows))
  if args.build:
   require_space(args.minimum_free_mib);run(['xcodegen','generate','--spec',str(ROOT/'apps/apple-simple/project.yml')])
   for name in names:
    run(['xcodebuild','-project',str(ROOT/'apps/apple-simple/HickorySimple.xcodeproj'),'-scheme',SCHEMES[name],'-configuration','Debug','-destination',destination,'-derivedDataPath',str(derived),'-jobs','2','build-for-testing','CODE_SIGNING_ALLOWED=NO'])
  candidates={name:[] for name in names}
  for path in (derived/'Build/Products').glob('*.xctestrun'):
   value=plistlib.loads(path.read_bytes())
   for name in names:
    if len(list(test_targets(value,name)))==1:candidates[name].append(path)
  if any(len(paths)!=1 for paths in candidates.values()):raise RuntimeError('Build one configuration per selected test scheme with --build')
  for workflow in workflows:
   try:
    require_space(args.minimum_free_mib)
    run_workflow(workflow, device, args.device, owned_device is not None, candidates[workflow_target(workflow)][0], derived,
                 destination, args.python_env, args.slow_debug, receipt, args.recorded_fixtures)
   except (AssertionError, OSError, RuntimeError, subprocess.CalledProcessError) as error:
    print(f'FAIL mounted UIKit {workflow}: {error}', flush=True)
    failures.append(workflow)
 finally:
  lifecycle={'seed_device':args.device,'device':device,'temporary_device':owned_device is not None,'temporary_device_requested':args.temporary_device,'booted_here':booted_here,'seed_initial_state':seed['state']}
  if owned_device is not None:
   shutdown=subprocess.run(['xcrun','simctl','shutdown',owned_device],capture_output=True,text=True)
   deleted=subprocess.run(['xcrun','simctl','delete',owned_device],capture_output=True,text=True)
   lifecycle.update({'shutdown_exit':shutdown.returncode,'delete_exit':deleted.returncode,'owned_device_deleted':deleted.returncode==0})
  elif booted_here:
   shutdown=subprocess.run(['xcrun','simctl','shutdown',device],capture_output=True,text=True)
   lifecycle.update({'shutdown_exit':shutdown.returncode})
  (receipt/'last-device-lifecycle.json').write_text(json.dumps(lifecycle,indent=2))
  if owned_device is not None and lifecycle['delete_exit']!=0:raise RuntimeError('Owned simulator cleanup failed; inspect last-device-lifecycle.json and delete only its recorded owned UDID')
  if owned_device is None and booted_here and lifecycle['shutdown_exit']!=0:raise RuntimeError('Selected simulator did not return to Shutdown')
 if failures:raise RuntimeError('Unaccepted iOS workflows: '+', '.join(failures))
if __name__=='__main__':main()
