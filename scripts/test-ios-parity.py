"""Owned mounted UIKit parity witness. Requires an explicitly selected simulator."""
import argparse,hashlib,importlib.util,json,os,plistlib,re,shutil,socket,subprocess,tempfile,time,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
TARGET='HickorySimpleMobileParityTests'
CASE='UIKitParityAcceptanceTests/testMountedUIKitEditingAndSelectedRustDebug'
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
 parser=argparse.ArgumentParser();parser.add_argument('--device',required=True);parser.add_argument('--build',action='store_true');parser.add_argument('--boot',action='store_true');parser.add_argument('--temporary-device',action='store_true');parser.add_argument('--minimum-free-mib',type=int,default=2048);args=parser.parse_args()
 if args.minimum_free_mib<128:parser.error('Minimum free space must remain at least128MiB')
 require_space(args.minimum_free_mib)
 devices=json.loads(subprocess.check_output(['xcrun','simctl','list','devices','-j']))
 existing_ids={d['udid'].lower()for group in devices['devices'].values()for d in group}
 selected=[(runtime,d)for runtime,group in devices['devices'].items()for d in group if d['udid']==args.device and d.get('isAvailable')is True]
 if len(selected)!=1:raise RuntimeError('Choose exactly one installed available simulator UDID')
 runtime,seed=selected[0]
 if not runtime.startswith('com.apple.CoreSimulator.SimRuntime.iOS-'):raise RuntimeError('Choose an installed iOS simulator')
 if args.temporary_device and not args.boot:parser.error('--temporary-device requires explicit --boot after resource coordination')
 device=args.device;owned_device=None;booted_here=False;passed=False
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
   if len(targets)==1:candidates.append((path,value,targets[0]))
  if len(candidates)!=1:raise RuntimeError('Build exactly one hosted parity test configuration with --build; no launch-only acceptance is inferred')
  spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
  with tempfile.TemporaryDirectory(prefix='ios-parity-')as temporary:
   folder=Path(temporary);workspace=folder/'workspace';(workspace/'src').mkdir(parents=True);(workspace/'.git').mkdir();sentinel=workspace/'wrong-ios-sibling-ran';source='// owned-ios-parity-source\n#[cfg(test)]\nmod cases {\n #[test]\n fn test_selected(){\n  let selected_value=41;\n  assert_eq!(selected_value,41); // owned-ios-parity-stop\n }\n #[test]\n fn test_sibling(){std::fs::write('+json.dumps(str(sentinel))+',b"wrong").unwrap();}\n}\n';(workspace/'src/lib.rs').write_text(source);(workspace/'Cargo.toml').write_text('[workspace]\n[package]\nname="owned-ios-parity"\nversion="0.1.0"\nedition="2024"\n')
   with socket.socket()as reserve:reserve.bind(('127.0.0.1',0));port=reserve.getsockname()[1]
   engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(workspace),'window',f'http://127.0.0.1:{port}'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
   try:
    origin=fixture.origin(engine);original,value,target=candidates[0]
    for key in ['EnvironmentVariables','TestHostEnvironmentVariables']:
     target.setdefault(key,{})['HICKORY_IOS_PARITY_ORIGIN']=origin
     target[key]['HICKORY_API_ORIGIN']=origin
    configured=derived/'Build/Products/owned-ios-parity.xctestrun';configured.write_bytes(plistlib.dumps(value));receipt=ROOT/'target/ios-parity';receipt.mkdir(parents=True,exist_ok=True);result=receipt/'last.xcresult'
    if result.exists():shutil.rmtree(result)
    log=receipt/'last.log'
    try:
     with log.open('w')as output:run(['xcodebuild','test-without-building','-xctestrun',str(configured),'-destination',destination,'-parallel-testing-enabled','NO','-maximum-concurrent-test-simulator-destinations','1','-only-testing:'+TARGET+'/'+CASE,'-resultBundlePath',str(result),'-test-timeouts-enabled','YES','-maximum-test-execution-time-allowance','180'],stdout=output,stderr=subprocess.STDOUT)
     summary=json.loads(subprocess.check_output(['xcrun','xcresulttool','get','test-results','summary','--path',str(result)]));(receipt/'last-summary.json').write_text(json.dumps(summary,indent=2))
     assert summary['result']=='Passed' and summary['totalTestCount']==1 and summary['passedTests']==1 and summary['failedTests']==0 and summary['skippedTests']==0,summary
     assert(workspace/'src/lib.rs').read_text()==source and not sentinel.exists()
     observed=re.findall(r'HickoryUIKitParity capability-native-paste-undo=(true|false)',log.read_text());assert len(observed)==1,observed
     native_paste_undo=observed[0]=='true'
     assert 'HickoryUIKitParity coordinated-cleanup-history-passed'in log.read_text()
     (receipt/'last-receipt.json').write_text(json.dumps({'device':device,'seed_device':args.device,'temporary_device':owned_device is not None,'test':TARGET+'/'+CASE,'summary':summary,'log_sha256':hashlib.sha256(log.read_bytes()).hexdigest(),'source_unchanged':True,'sibling_excluded':True,'native_paste_undo_verified':native_paste_undo,'coordinated_cleanup_undo_redo_verified':True,'keyboard_input_verified':False,'physical_input_verified':False,'accessibility_verified':False},indent=2))
     passed=True
    finally:configured.unlink(missing_ok=True)
   finally:fixture.stop(engine)
 finally:
  lifecycle={'seed_device':args.device,'device':device,'temporary_device':owned_device is not None,'temporary_device_requested':args.temporary_device,'booted_here':booted_here,'seed_initial_state':seed['state']}
  if owned_device is not None:
   shutdown=subprocess.run(['xcrun','simctl','shutdown',owned_device],capture_output=True,text=True)
   deleted=subprocess.run(['xcrun','simctl','delete',owned_device],capture_output=True,text=True)
   lifecycle.update({'shutdown_exit':shutdown.returncode,'delete_exit':deleted.returncode,'owned_device_deleted':deleted.returncode==0})
  elif booted_here:
   shutdown=subprocess.run(['xcrun','simctl','shutdown',device],capture_output=True,text=True)
   lifecycle.update({'shutdown_exit':shutdown.returncode})
  receipt=ROOT/'target/ios-parity';receipt.mkdir(parents=True,exist_ok=True);(receipt/'last-device-lifecycle.json').write_text(json.dumps(lifecycle,indent=2))
  if owned_device is not None and lifecycle['delete_exit']!=0:raise RuntimeError('Owned simulator cleanup failed; inspect last-device-lifecycle.json and delete only its recorded owned UDID')
  if owned_device is None and booted_here and lifecycle['shutdown_exit']!=0:raise RuntimeError('Selected simulator did not return to Shutdown')
 if passed:print('PASS mounted UIKit Paste + guarded Cleanup Undo/Redo + genuine selected Rust Debug; native Paste Undo '+('verified'if native_paste_undo else'unverified')+'; physical input and accessibility unverified')
if __name__=='__main__':main()
