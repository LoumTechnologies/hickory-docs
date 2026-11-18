"""Owned mounted UIKit parity witness. Requires an explicitly selected simulator."""
import argparse,hashlib,importlib.util,json,os,plistlib,shutil,socket,subprocess,tempfile,urllib.parse,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
TARGET = 'HickorySimpleMobileParityTests'
PROPOSAL_DESTINATION = 'moved λ%2F?#.py'
WORKFLOWS = {
    'editing': ('testMountedUIKitEditingAndSelectedRustDebug', ['native-paste-history-passed', 'coordinated-cleanup-history-passed', 'debug-paused', 'completed']),
    'workspace': ('testMountedSearchProblemsAndCollectedTests', ['workspace-navigation-problems-passed', 'workspace-tests-coverage-passed']),
    'quickfix': ('testMountedClangdQuickFixReviewsAndReplaysDraft', ['clangd-quickfix-preview-passed', 'clangd-quickfix-history-passed']),
    'typescript-extraction': ('testMountedTypeScriptExtractionReplaysExactDraft', ['typescript-extraction-exact-history-passed']),
    'recovery': ('testMountedConnectedReadingAndPrivateRecovery', ['connected-impact-passed', 'private-recovery-passed']),
    'proposal': ('testMountedHistoricalProposalRefusesApplication', ['historical-proposal-passed']),
    'proposal-apply': ('testMountedManagedProposalAppliesBothReviewedDrafts', ['managed-proposal-preview-passed', 'managed-proposal-history-passed']),
    'proposal-save': ('testMountedManagedProposalPublishesReviewedResource', ['managed-resource-publication-passed', 'managed-resource-inverse-passed']),
}

def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT/'scripts'/f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def prepare_workflow(workflow, workspace, python_env, slow_debug):
    workspace.mkdir()
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
    elif workflow == 'typescript-extraction':
        tools = ROOT/'target/refactor-provider/node_modules'
        server = tools/'typescript-language-server/lib/cli.mjs'
        for name, version in [('typescript-language-server', '6.1.0'), ('typescript', '6.0.3')]:
            assert json.loads((tools/name/'package.json').read_text())['version'] == version
        (workspace/'node_modules').mkdir()
        (workspace/'node_modules/typescript').symlink_to(tools/'typescript', target_is_directory=True)
        case = load_script('test-provider-refactor-host').CASES[0]
        files = {
            case['path']: case['source'],
            '.hickory-refactor-fixture': 'owned-refactor-v1\n',
            '.hick-lsp.json': json.dumps({'servers': {'typescript': {'command': ['node', str(server), '--stdio']}}}),
            'package.json': '{"type":"module"}',
            'tsconfig.json': '{"compilerOptions":{"strict":true,"target":"ES2022","ignoreDeprecations":"6.0"}}',
        }
        sentinel = None
    elif workflow == 'recovery':
        details = load_script('test-behavior-readings-browser').prepare(workspace)
        (workspace/'.venv').unlink()
        (workspace/'.venv').symlink_to(python_env.resolve(), target_is_directory=True)
        files = {name: (workspace/name).read_text() for name in ['project.md', 'source.md', 'behavior.md', 'a.py', 'test_a.py', 'test_undeclared.py', 'hickory-dogfood/behavior-map.json', 'hickory-dogfood/narrative-map.json']}
        sentinel = Path(details['sibling'])
    elif workflow in {'proposal-apply', 'proposal-save'}:
        load_script('test-agent-proposals-browser').prepare(workspace)
        files = {name: (workspace/name).read_text() for name in ['project.md', 'construction.md', 'note.py', 'generated.py', 'behavior.md']}
        close = '\x3c/h:doc>\n'
        files['construction.md'] = files['construction.md'].removesuffix(close) + 'Retained author context for the mounted viewport.\n' * 100 + close
        sentinel = None
    else:
        archive = load_script('test-agent-proposal-persistence')
        files = {**archive.FILES, '.hickory-proposal-archive-fixture': 'owned-proposal-archive-v1\n'}
        sentinel = None
    for name, text in files.items():
        (workspace/name).write_text(text)
    return files, sentinel

def run_workflow(workflow, device, seed, temporary_device, candidate, derived, destination, python_env, slow_debug, receipt):
    fixture = load_script('test-simple-browser')
    method, markers = WORKFLOWS[workflow]
    name = TARGET+'/UIKitParityAcceptanceTests/'+method
    with tempfile.TemporaryDirectory(prefix='ios-parity-') as temporary:
        folder = Path(temporary)
        workspace, state = folder/'workspace', folder/'state'
        files, sentinel = prepare_workflow(workflow, workspace, python_env, slow_debug)
        environment = {**os.environ, 'HICKORY_STATE_DIR': str(state)}
        managed = load_script('test-agent-proposals-browser') if workflow in {'proposal-apply', 'proposal-save'} else None
        if managed:
            settings, calls, tool_result = managed.provider_fixture(folder)
            environment['HICKORY_AGENT_TEST_UI_SETTINGS'] = str(settings)
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
        with socket.socket() as reserve:
            reserve.bind(('127.0.0.1', 0))
            port = reserve.getsockname()[1]
        command = [os.environ.get('HICKORY_SIMPLE_TEST_ENGINE', str(ROOT/'target/source-edit-focus/debug/examples/engine_client')), str(workspace), 'window', f'http://127.0.0.1:{port}']
        archive = load_script('test-agent-proposal-persistence') if workflow == 'proposal' else None
        metadata = {}
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
        engine = fixture.start(command, env=environment)
        try:
            origin = fixture.origin(engine)
            if archive:
                catalogue = archive.call(origin, 'GET', '/api/agent-proposals')
                assert catalogue['workspace'] != original['workspace'], 'Expected a new owned engine lifetime'
            value = plistlib.loads(candidate.read_bytes())
            target, = test_targets(value)
            for key in ['EnvironmentVariables', 'TestHostEnvironmentVariables']:
                target.setdefault(key, {}).update(HICKORY_IOS_PARITY_ORIGIN=origin, HICKORY_API_ORIGIN=origin, **metadata)
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
                for path, text in files.items():
                    assert (workspace/path).read_bytes() == text.encode(), path
                if sentinel:
                    assert not sentinel.exists(), sentinel.name
                if workflow == 'workspace':
                    events = (workspace/'.hick-cache/executions').read_text().splitlines()
                    assert sorted(events) == ['negative', 'positive', 'positive'], events
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
                observed = log.read_text()
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
                else:
                    payload['source_unchanged'] = True
                if sentinel:
                    payload['sibling_excluded'] = True
                if workflow == 'editing':
                    payload.update(native_paste_undo_redo_verified=True, coordinated_cleanup_undo_redo_verified=True)
                (receipt/f'{prefix}-receipt.json').write_text(json.dumps(payload, indent=2))
            finally:
                configured.unlink(missing_ok=True)
        finally:
            if archive:
                archive.stop_engine(fixture, engine, state)
            else:
                fixture.stop(engine)
    print(f'PASS mounted UIKit {workflow}', flush=True)

def run(args,**kwargs):
 return subprocess.run(args,check=True,text=True,**kwargs)
def require_space(minimum):
 available=shutil.disk_usage(ROOT).free
 if available<minimum*1024*1024:raise RuntimeError(f'Need {minimum}MiB free before simulator/build work; available {available//(1024*1024)}MiB')
def test_targets(value):
 if isinstance(value,dict):
  if value.get('BlueprintName')==TARGET or value.get('TestableName')==TARGET:yield value
  else:
   for child in value.values():yield from test_targets(child)
 elif isinstance(value,list):
  for child in value:yield from test_targets(child)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--device',required=True);parser.add_argument('--build',action='store_true');parser.add_argument('--boot',action='store_true');parser.add_argument('--temporary-device',action='store_true');parser.add_argument('--minimum-free-mib',type=int,default=2048)
 parser.add_argument('--workflow', choices=WORKFLOWS, action='append', help='Repeat to use one simulator for several focused cases')
 parser.add_argument('--python-env', type=Path, default=ROOT/'target/thread-debug-env')
 parser.add_argument('--receipt-dir', type=Path, help='Create a new directory for this run and its lifecycle receipt')
 parser.add_argument('--slow-debug', action='store_true', help='Delay the owned Rust fixture to exercise staged admission')
 args=parser.parse_args()
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
  if args.build:
   require_space(args.minimum_free_mib);run(['xcodegen','generate','--spec',str(ROOT/'apps/apple-simple/project.yml')])
   run(['xcodebuild','-project',str(ROOT/'apps/apple-simple/HickorySimple.xcodeproj'),'-scheme','HickorySimpleMobileParity','-configuration','Debug','-destination',destination,'-derivedDataPath',str(derived),'-jobs','2','build-for-testing','CODE_SIGNING_ALLOWED=NO'])
  candidates=[]
  for path in (derived/'Build/Products').glob('*.xctestrun'):
   value=plistlib.loads(path.read_bytes());targets=list(test_targets(value))
   if len(targets)==1:candidates.append(path)
  if len(candidates)!=1:raise RuntimeError('Build exactly one hosted parity test configuration with --build; no launch-only acceptance is inferred')
  for workflow in dict.fromkeys(args.workflow or ['editing']):
   try:
    require_space(args.minimum_free_mib)
    run_workflow(workflow, device, args.device, owned_device is not None, candidates[0], derived, destination, args.python_env, args.slow_debug, receipt)
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
