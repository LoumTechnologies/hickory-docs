"""Run a native ordinary-save history witness against one owned plain source fixture."""
import argparse, importlib.util, json, os, signal, subprocess, tempfile, threading, xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['apple','android']);parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client');parser.add_argument('--serve-only',action='store_true');parser.add_argument('--create',action='store_true',help='Select Android strict Save As; Apple uses its same combined history/Save As case.');args=parser.parse_args()
    def interrupted(number,_frame):raise SystemExit(128+number)
    signal.signal(signal.SIGTERM,interrupted)
    spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    if not args.engine.is_file():parser.error('Build the existing engine_client example first; this witness installs no dependencies.')
    with tempfile.TemporaryDirectory(prefix='hickory-native-history-') as folder:
        temporary=Path(folder);root=temporary/'workspace';root.mkdir();(root/'.git').mkdir();(root/'history.txt').write_text('saved-before\n');(root/'.hickory-native-history-fixture').write_text('owned-native-history-v1\n')
        engine=fixture.start([str(args.engine.resolve()),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(temporary/'state')})
        try:
            origin=fixture.origin(engine);print(json.dumps({'origin':origin,'workspace':str(root),'input':'history.txt','acceptance':'not run'}),flush=True)
            if args.serve_only:threading.Event().wait();return
            env={**os.environ,'HICKORY_NATIVE_HISTORY_ORIGIN':origin,'HICKORY_NATIVE_HISTORY_WORKSPACE':str(root)}
            if args.host=='apple':
                name='testLiveNativeOrdinarySaveHistoryRestoreAndUndo';report=temporary/'history.xml';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','SourceHistoryTests.'+name+'$','--xunit-output',str(report)]
            else:
                name='liveNativeStrictSaveAsMenuRetainsEditorAndGuardsOwnership' if args.create else 'liveNativeOrdinarySaveHistoryRestoreAndUndo';case='SourceCreationTest' if args.create else 'SourceHistoryTest';report=ROOT/('apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.'+case+'.xml');command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.'+case+'.'+name,'--rerun-tasks']
            report.unlink(missing_ok=True);log=ROOT/'target/native-history-smoke'/f'{args.host}-last.log';log.parent.mkdir(parents=True,exist_ok=True);process=fixture.start(command,cwd=ROOT,env=env)
            try:output,_=process.communicate(timeout=600)
            except BaseException:
                fixture.stop(process);output,_=process.communicate(timeout=15);log.write_text(output or '');raise
            log.write_text(output or '')
            if process.returncode:raise RuntimeError(f'Native history failed; inspect {log}')
            rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ('skipped','failure','error')),str(report)
            print(f'PASS {args.host}: one actual {"guarded Save As" if args.create else "ordinary-save/history/draft restoration"} controller witness, no skips. Apple combines history and Save As in this same case. Physical input/accessibility remains unverified.')
        finally:fixture.stop(engine)
if __name__=='__main__':main()
