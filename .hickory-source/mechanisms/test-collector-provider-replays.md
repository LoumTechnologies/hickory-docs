<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Replay genuine standard collector outcomes in an owned project

For maintainers validating supported CLI collectors independently of the common rendered test surface.

Each invocation creates a new disposable project and starts a real engine with isolated state. Discovery must execute no test bodies. An intentional failing test must retain structured failure output and genuine coverage; a second retained run must execute only the failed identities. Reported skipped cases stay visible as skipped and never count as executed test bodies. A same-source failed-only run stays available; then an intentional product fix triggers fresh no-body discovery and a passing run of only the original failed identities. All three records remain in history, and the original evidence stays historical. Apart from that explicit fixture fix, every authored fixture byte stays unchanged. Rust discovery may create its lockfile: retain that historical observation and ask again for a stable listing instead of blessing the earlier bytes. `--collision` checks identical framework names in distinct authored files with crossed failures; a broad name regex is not an adequate rerun selector.

Tool dependencies live only under `target`: Jest29.7.0, Vitest3.2.7 and matching coverage-v8 3.2.7; cargo-llvm-cov0.9.1 plus the LLVM component identified by the installed Rust1.96.1 manifest. The component archive SHA256 is 3778f19dde00252cfd25e3ef32c1c79ed128a80e9f11df7c4c659df3a61bef55. `LLVM_COV` and `LLVM_PROFDATA` point inside the verified extracted component, and automatic global rustup setup is disabled. The Rust fixture explicitly pins the installed repository toolchain in its disposable project, since the global stable toolchain can use a different LLVM profile format. The driver fails on missing tools rather than counting a skip as collector success. This service acceptance does not certify UI or language semantics and does not replace native/browser user-flow receipts.

`--source-adjacent` drives the saved source-candidate admission route and its retained native terminal. A similar filename with identical declarations must execute no bodies; stale source refuses before creating a session. Empty literal names require discovery or an explicit containing-file choice, while whitespace filters preserve their literal bytes. `--stale-second` probes complete native preflight: a stale second file must prevent even the first group's bodies. `--retire-root` replaces the project while the first native group is held, then requires its observed failures to remain retained, no second-group body, historical authority, and no cache or source writes into the foreign replacement. `--isolated-dependencies` exposes only direct provider packages, so native merge helpers cannot accidentally depend on a flattened project installation.

<ui:copy id="collector-provider-live-replay">
#!/usr/bin/env python3
"""Genuine bounded collector discovery, failed evidence, failed-only rerun and history."""
import argparse,hashlib,importlib.util,json,os,signal,subprocess,tempfile,time,urllib.error,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('provider',choices=['jest','vitest','rust']);parser.add_argument('--missing-case',action='store_true');parser.add_argument('--ambiguous-case',action='store_true');parser.add_argument('--stale-second',action='store_true');parser.add_argument('--isolated-dependencies',action='store_true');parser.add_argument('--source-adjacent',action='store_true');parser.add_argument('--retire-root',action='store_true');parser.add_argument('--name-case',choices=['basic','metachar','case-only','empty','whitespace','separator'],default='basic');parser.add_argument('--collision',action='store_true',help='Distinct files with the same full test names and crossed failures');parser.add_argument('--node-modules',type=Path,default=ROOT/'target/collector-node-providers/node_modules');args=parser.parse_args()
 if args.source_adjacent or args.retire_root or args.stale_second:args.collision=True
 if args.provider=='rust' and (args.collision or args.name_case!='basic'):parser.error('Rust module identities already namespace its fixture cases')
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 with tempfile.TemporaryDirectory(prefix='hickory-real-collector-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.hickory-collector-provider-fixture').write_text('owned-provider-v1\n');events=root/'.hick-cache/executions';sources={};language='rust' if args.provider=='rust' else 'typescript' if args.provider=='vitest' else 'javascript'
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state'),'NPM_CONFIG_CACHE':str(ROOT/'target/collector-npm-cache'),'NPM_CONFIG_OFFLINE':'true','NPM_CONFIG_UPDATE_NOTIFIER':'false'}
  expected=['a-positive','a-negative']
  if args.provider=='rust':
   metadata=json.loads((ROOT/'target/collector-llvm-tools.json').read_text());tool=ROOT/'target/collector-toolchain/bin/cargo-llvm-cov';assert tool.is_file(),'Prepare pinned target-only cargo-llvm-cov first';env.update(PATH=str(tool.parent)+os.pathsep+env['PATH'],LLVM_COV=metadata['bins']['llvm-cov'],LLVM_PROFDATA=metadata['bins']['llvm-profdata'],RUSTUP_TOOLCHAIN='1.96.1-aarch64-apple-darwin',CARGO_LLVM_COV_SETUP='no',CARGO_HOME=str(ROOT/'target/collector-cargo-home'),CARGO_TARGET_DIR=str(folder/'cargo-target'),CARGO_LLVM_COV_TARGET_DIR=str(folder/'llvm-cov-target'))
   sources['Cargo.toml']='[package]\nname="owned_collector"\nversion="0.0.0"\nedition="2024"\n';sources['src/lib.rs']='pub fn classify(n:i32)->i32{if n<0{-1}else{1}}\n#[cfg(test)]mod tests{use super::*;fn record(name:&str){use std::io::Write;std::fs::create_dir_all(".hick-cache").unwrap();writeln!(std::fs::OpenOptions::new().create(true).append(true).open(".hick-cache/executions").unwrap(),"{name}").unwrap();}#[test]fn positive(){record("a-positive");assert_eq!(classify(1),1);}#[test]fn negative(){record("a-negative");assert_eq!(classify(-1),1);}}\n'
  else:
   modules=args.node_modules.resolve();pins={'jest':'29.7.0','vitest':'3.2.7','@vitest/coverage-v8':'3.2.7'}
   for package in ([args.provider,'@vitest/coverage-v8'] if args.provider=='vitest' else [args.provider]):assert json.loads((modules/package/'package.json').read_text())['version']==pins[package],package
   if args.isolated_dependencies:
    direct=folder/'direct-modules';direct.mkdir();(direct/'.bin').mkdir()
    for name in ([args.provider,'@vitest/coverage-v8'] if args.provider=='vitest' else [args.provider]):destination=direct/name;destination.parent.mkdir(parents=True,exist_ok=True);destination.symlink_to(modules/name,target_is_directory=True)
    (direct/'.bin'/args.provider).symlink_to(modules/args.provider/('bin/jest.js' if args.provider=='jest' else 'vitest.mjs'));modules=direct
   (root/'node_modules').symlink_to(modules,target_is_directory=True);extension='ts' if args.provider=='vitest' else 'js';package={'name':'owned-collector','private':True,'devDependencies':{args.provider:pins[args.provider]}}
   if args.provider=='vitest':package.update(type='module');package['devDependencies']['@vitest/coverage-v8']='3.2.7'
   else:package['jest']={'testEnvironment':'node','maxWorkers':1}
   sources['package.json']=json.dumps(package,indent=2)+'\n';sources['app.'+extension]=('export function classify(n:number){if(n<0)return -1;return 1;}\n' if args.provider=='vitest' else 'exports.classify=(n)=>{if(n<0)return -1;return 1;};\n')
   for tag in (['a','b'] if args.collision else ['a']):
    if tag=='b':expected+=['b-positive','b-negative']
    imports=('import {describe,test,expect} from "vitest";import {classify} from "./app";import {appendFileSync,mkdirSync,existsSync,writeFileSync} from "node:fs";\n' if args.provider=='vitest' else 'const {classify}=require("./app");const {appendFileSync,mkdirSync,existsSync,writeFileSync}=require("node:fs");\n')
    retire='if(name==="a-negative"&&existsSync('+json.dumps(str(folder/'retire-now'))+')){writeFileSync('+json.dumps(str(folder/'retire-ready'))+',"ready");while(!existsSync('+json.dumps(str(folder/'retire-release'))+'))Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0,20);}'
    record='const record=(name)=>{mkdirSync('+json.dumps(str(events.parent))+',{recursive:true});appendFileSync('+json.dumps(str(events))+',name+"\\n");'+retire+'};\n'
    positive,negative={'basic':('positive','negative'),'metachar':('value literal','value [x].* (literal)+? ^$ |'),'case-only':('case','CASE'),'empty':('nonempty',''),'whitespace':('nonblank','   '),'separator':('positive > literal','negative > literal')}[args.name_case]
    cases='describe("cohort",()=>{test('+json.dumps(positive)+',()=>{record("'+tag+'-positive");expect(classify(1)).toBe('+('-1' if tag=='b' else '1')+');});test('+json.dumps(negative)+',()=>{record("'+tag+'-negative");expect(classify(-1)).toBe('+('-1' if tag=='b' else '1')+');});});\n'
    if args.name_case in ['empty','whitespace']:cases=cases[len('describe("cohort",()=>{'):-len('});\n')]+'\n'
    sources[('not-a' if tag=='b' and args.source_adjacent else tag)+'.test.'+extension]=imports+record+cases
  for path,text in sources.items():destination=root/path;destination.parent.mkdir(parents=True,exist_ok=True);destination.write_text(text)
  if args.stale_second:
   project=root.resolve();artifacts=folder/'private';artifacts.mkdir();reports=artifacts/'reports';reports.mkdir();groups=[]
   for tag in ['a','b']:
    file=(root/(tag+'.test.'+extension)).resolve();groups.append({'file':str(file),'hash':hashlib.sha256(file.read_bytes()).hexdigest(),'names':['cohort negative' if tag=='a' else 'cohort positive'],'known_names':['cohort positive','cohort negative']})
   second=root/('b.test.'+extension);second.write_text(second.read_text()+'\n// changed before the first native group\n');stat=project.stat();spec={'provider':args.provider,'project':str(project),'project_dev':str(stat.st_dev),'project_ino':str(stat.st_ino),'groups':groups,'artifacts':str(artifacts),'reports':str(reports),'result':str(artifacts/'results')};driver=(ROOT/'crates/hick-plugins/src/testing/javascript.mjs').read_text();run=subprocess.run(['node','--input-type=module','--eval',driver,json.dumps(spec)],cwd=project,capture_output=True,text=True);assert run.returncode and 'selected test source changed' in run.stderr and not events.exists(),(run.stdout,run.stderr);print('PASS native second-source preflight refuses before any first-group bodies');return
  engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env=env)
  try:
   origin=fixture.origin(engine)
   def api(body=None,endpoint='tests/coverage'):
    request=urllib.request.Request(origin+'/api/'+endpoint,data=None if body is None else json.dumps(body).encode(),headers={'Content-Type':'application/json'})
    try:
     with urllib.request.urlopen(request,timeout=180) as response:return response.status,json.load(response)
    except urllib.error.HTTPError as error:return error.code,json.load(error)
   if args.source_adjacent:
    path='a.test.'+extension;analysis={'path':path,'source':sources[path]};status,facts=api(analysis,'tests/source-candidates');assert status==200,facts;candidate=next(item for item in facts['candidates'] if item['name']==('(empty test name)' if args.name_case=='empty' else negative));request={'version':1,'input':{'kind':'saved'},'analysis':analysis,'fingerprint':facts['fingerprint'],'candidate_id':candidate['id'],'selection':{'kind':'name_filter'}}
    if args.name_case=='empty':request['selection']={'kind':'containing_scope'}
    (root/path).write_text(sources[path]+'\n// changed after inspection\n');status,refused=api(request,'tests/run-source');assert status==409,refused;assert not events.exists();(root/path).write_text(sources[path])
    if args.name_case=='empty':
     request['selection']={'kind':'name_filter'}
     assert candidate['name_filter'] is None,candidate;status,refused=api(request,'tests/run-source');assert status==422,refused;assert not events.exists();request['selection']={'kind':'containing_scope'}
    status,admitted=api(request,'tests/run-source');assert status==200,admitted;assert admitted['source_checked_at_admission'] and admitted['checked_sources']==facts['source_hashes'],admitted;deadline=time.monotonic()+120
    while time.monotonic()<deadline:
     status,terminals=api(None,'terminals');assert status==200,terminals;terminal=next((item for item in terminals['sessions'] if item['id']==admitted['id']),None)
     if terminal and terminal['exit_code'] is not None:break
     time.sleep(.05)
    else:raise AssertionError('Source-adjacent native process did not terminate')
    assert terminal['exit_code']==1,terminal;actual=events.read_text().splitlines();assert sorted(actual)==sorted(['a-positive','a-negative'] if args.name_case=='empty' else ['a-negative']),actual
    for path,text in sources.items():assert (root/path).read_text()==text,path
    print(json.dumps({'provider':args.provider,'source_adjacent':True,'name_case':args.name_case,'stale_refused':True,'empty_filter_requires_explicit_file':args.name_case=='empty','bodies':actual,'other_similar_filename_bodies':0,'process_exit':1,'test_verdict_inferred':False,'unchanged_source':True},indent=2));return
   listing=None
   for _ in range(3):
    status,listing=api({'action':'discover','path':'.','language':language});assert status==200,listing;assert len(listing['tests'])==len(expected),listing;assert not events.exists(),'Discovery executed a test body'
    if listing['source_state']=='current':break
   assert listing['source_state']=='current',listing
   def finished(id):
    deadline=time.monotonic()+180
    while time.monotonic()<deadline:
     status,value=api({'action':'get','id':id});assert status==200,value
     if value['status']!='running':return value
     time.sleep(.05)
    raise AssertionError('Owned collector timed out')
   status,start=api({'action':'run','path':'.','language':language});assert status==200,start;first=finished(start['id']);assert first['status']=='failed',first;assert len(first['tests'])==len(expected),first;assert first['evidence'] is not None,first;failures=[test for test in first['tests'] if test['status']=='failed'];assert len(failures)==(2 if args.collision else 1),first;assert all(test['output'] for test in failures),failures
   guarded_source=next(path for path in sources if path.startswith('app.') or path=='src/lib.rs')
   if args.retire_root:(folder/'retire-now').write_text('retire during first native rerun group')
   expected_failed=['a-negative']+(['b-positive'] if args.collision else []);status,start=api({'action':'rerun-failed','id':first['id']});assert status==200,start
   if args.retire_root:
    deadline=time.monotonic()+60
    while time.monotonic()<deadline and not (folder/'retire-ready').exists():time.sleep(.02)
    assert (folder/'retire-ready').exists(),'First native rerun body never reached retirement gate';moved=folder/'original-workspace';root.rename(moved);root.mkdir();(root/'foreign-sentinel').write_text('foreign')
    for path,text in sources.items():destination=root/path;destination.parent.mkdir(parents=True,exist_ok=True);destination.write_text(text)
    (root/'node_modules').symlink_to(modules,target_is_directory=True);(folder/'retire-release').write_text('release original native body');rerun=finished(start['id']);assert rerun['status']=='unavailable' and rerun['authority']=='historical-unverified',rerun;assert rerun['tests'] and any(test['status']=='failed' for test in rerun['tests']),rerun;assert sorted((moved/'.hick-cache/executions').read_text().splitlines())==sorted(expected+['a-negative']);assert not (root/'.hick-cache').exists() and not (root/'.vite').exists();assert (root/'foreign-sentinel').read_text()=='foreign';status,history=api({'action':'list'});assert status==200 and len(history['runs'])==2,history
    for path,text in sources.items():assert (root/path).read_text()==text and (moved/path).read_text()==text,path
    print(json.dumps({'provider':args.provider,'root_retired':True,'first_native_group_results_retained':len(rerun['tests']),'second_group_bodies':0,'history':2,'authority':rerun['authority'],'foreign_cache_created':False,'foreign_authored_bytes_unchanged':True},indent=2));return
   rerun=finished(start['id'])
   executed=[test for test in rerun['tests'] if test['status'] in ['passed','failed']];assert rerun['status']=='failed' and len(executed)==len(failures),rerun;assert {test['id'] for test in executed}=={test['id'] for test in failures},rerun;assert all(test['status'] in ['passed','failed','skipped'] for test in rerun['tests']),rerun;assert rerun['evidence'] is not None,rerun;assert sorted(events.read_text().splitlines())==sorted(expected+expected_failed),events.read_text();assert len({test['id'] for test in first['tests']})==len(expected),'Distinct authored test cases share one retained identity'
   if args.missing_case or args.ambiguous_case:
    path='src/lib.rs' if args.provider=='rust' else 'a.test.'+extension;text=sources[path]
    if args.missing_case:text=text.replace('fn negative()', 'fn removed_negative()') if args.provider=='rust' else text.replace(json.dumps(negative),json.dumps('removed negative'),1)
    else:
     duplicate='test('+json.dumps(negative)+',()=>{record("unexpected-duplicate");});';text+='\n'+(duplicate if args.name_case in ['empty','whitespace'] else 'describe("cohort",()=>{'+duplicate+'});')+'\n'
    sources[path]=text;(root/path).write_text(text);status,refused=api({'action':'rerun-failed','id':first['id']});assert status==422,refused;assert sorted(events.read_text().splitlines())==sorted(expected+expected_failed),'Fresh discovery ran bodies or launched an ambiguous selection';_,history=api({'action':'list'});assert len(history['runs'])==2,history;print('PASS changed-source missing/ambiguous failed identity refuses before bodies; historical outcomes preserved');return
   original=sources[guarded_source]
   if args.provider=='rust':fixed=original.replace('if n<0{-1}', 'if n<0{1}')
   else:fixed=original.replace('return -1','return 1')
   if args.collision and args.provider!='rust':fixed=original.replace('return -1','return 1').replace(';return 1;', ';return -1;')
   assert fixed!=original,(args.provider,original);sources[guarded_source]=fixed;(root/guarded_source).write_text(fixed);status,start=api({'action':'rerun-failed','id':first['id']});assert status==200,start;after_fix=finished(start['id']);fixed_cases=[case for case in after_fix['tests'] if case['status'] in ['passed','failed']];assert after_fix['status']=='completed' and len(fixed_cases)==len(failures),after_fix;assert all(case['status']=='passed' for case in fixed_cases),after_fix;assert {case['id'] for case in fixed_cases}=={case['id'] for case in failures},after_fix;assert after_fix['evidence'] is not None and 'runner discovery' in after_fix['notes'],after_fix;assert sorted(events.read_text().splitlines())==sorted(expected+expected_failed+expected_failed),'Fresh discovery executed bodies or failed-only selection widened';_,original_run=api({'action':'get','id':first['id']});assert original_run['source_state']=='historical' and original_run['identity']==first['identity'],original_run
   _,history=api({'action':'list'});assert len(history['runs'])==3 and {row['id'] for row in history['runs']}=={first['id'],rerun['id'],after_fix['id']},history
   for path,text in sources.items():assert (root/path).read_text()==text,path
   print(json.dumps({'provider':args.provider,'name_case':args.name_case,'collision':args.collision,'discovery':len(expected),'discovery_bodies':0,'failed':len(failures),'rerun_executed':len(executed),'rerun_reported_skipped':sum(test['status']=='skipped' for test in rerun['tests']),'ids':[test['id'] for test in rerun['tests']],'history':3,'after_fix_fresh_discovery':True,'after_fix_passed_only_original_failures':True,'coverage_files':list(rerun['evidence']['report']['files']),'source_state':rerun['source_state'],'unchanged_authored_bytes':True},indent=2));print('PASS genuine collector discovery/run/failing coverage/failed-only rerun/history. No UI, language-semantic, physical-input or accessibility acceptance is inferred.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-collector-providers.py"><ui:paste select="#collector-provider-live-replay"/></ui:file>
</ui:doc>
