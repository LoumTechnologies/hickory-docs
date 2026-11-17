#!/usr/bin/env python3
"""Owned real Git/native editor conflict composition acceptance; use the Xcode toolchain."""
import importlib.util,os,tempfile,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[1]
def mod(name,path):
 s=importlib.util.spec_from_file_location(name,R/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
f=mod('fixture','scripts/test-simple-browser.py');g=mod('gitfixture','scripts/test-git-conflict-resolutions.py')
for kind in ['native','delete','rename','stale','index']:
 with tempfile.TemporaryDirectory(prefix='hickory-apple-git-') as t:
  root=Path(t)/'workspace';path=g.make(root,kind);path='left.py' if kind=='rename' else path
  (root/'.hickory-native-git-fixture').write_text('owned-native-git-v1\n')
  engine=f.start([str(R/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(Path(t)/'state')})
  try:
   env={**os.environ,'HICKORY_NATIVE_GIT_ORIGIN':f.origin(engine),'HICKORY_NATIVE_GIT_WORKSPACE':str(root),'HICKORY_NATIVE_GIT_INPUT':path,'HICKORY_NATIVE_GIT_CASE':kind}
   log=Path('/tmp/apple-git-'+kind+'.log')
   with log.open('w') as out:p=subprocess.run(['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--skip-build','--filter','GitConflictResolutionTests.testLiveNativeCompositionSaveStageAndEditorHistory$'],env=env,stdout=out,stderr=subprocess.STDOUT)
   print(kind,p.returncode,flush=True)
   if p.returncode: raise RuntimeError(str(log)+'\n'+log.read_text()[-7000:])
   assert 'Executed 1 test, with 0 failures' in log.read_text() and 'Test skipped' not in log.read_text()
  finally:f.stop(engine)
