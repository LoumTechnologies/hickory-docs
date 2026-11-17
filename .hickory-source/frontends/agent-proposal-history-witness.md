<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read a historical proposal beside a current dirty editor

For maintainers checking restart retention through the usable simple editors.
The first engine seals a caller-provided intention without changing source. A
second engine opens the same physical workspace and private archive, then the
host reads that exact old seal while keeping a new unsaved edit. Historical
material must be visible without acquiring current editor authority or enabling
Apply. Closing the reading must leave the editor's own Undo usable.

The storage protocol witness separately checks retained host claims, corruption
and source-effect refusal. This host witness deliberately leaves the old proposal
unreported, so a previous application report cannot accidentally mask a missing
historical-application guard.

<ui:copy id="agent-proposal-history-host-driver">
#!/usr/bin/env python3
"""Owned restart-to-historical-reading acceptance on one simple host."""
import argparse,hashlib,importlib.util,json,os,re,socket,subprocess,tempfile,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['browser','apple','android']);parser.add_argument('--skip-build',action='store_true');args=parser.parse_args()
 fixture=module('fixture',ROOT/'scripts/test-simple-browser.py');protocol=module('proposal_archive',ROOT/'scripts/test-agent-proposal-persistence.py')
 with tempfile.TemporaryDirectory(prefix='hickory-proposal-history-host-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();state=folder/'private-state';processes=[];driver=None
  for path,text in protocol.FILES.items():(root/path).write_text(text)
  (root/'.hickory-proposal-archive-fixture').write_text('owned-proposal-archive-v1\n')
  env={**os.environ,'HICKORY_STATE_DIR':str(state)}
  with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
  def start():return fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',f'http://127.0.0.1:{port}'],env=env)
  first=start()
  try:original=protocol.call(fixture.origin(first),'POST','/api/agent-proposals/prepare',protocol.intention());assert original['durability']['retained'] is True,original.get('durability')
  finally:protocol.stop_engine(fixture,first,state)
  second=start();processes.append(second)
  try:
   origin=fixture.origin(second);catalogue=protocol.call(origin,'GET','/api/agent-proposals');assert catalogue['workspace']!=original['workspace'];historical=protocol.call(origin,'GET','/api/agent-proposals/'+original['token']);assert historical['historical'] is True and historical['material']==original['material'];assert hashlib.sha256(historical['material'].encode()).hexdigest()==original['fingerprint']
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');path='note.py';source=protocol.FILES[path];draft=source+'# retained archive draft\n';driver.act('open',path=path);driver.act('remember-editor',path=path);driver.act('construction-set-source',path=path,source=draft);size=driver.act('editor-size',path=path)
    driver.act('proposal-open');reading=driver.act('proposal-select',token=original['token']);assert 'Retain an exact historical intention' in reading and 'historical' in reading.lower(),reading;assert not driver.act('proposal-state')['applyEnabled'];assert driver.act('construction-source',path=path)==draft;driver.act('proposal-close');assert driver.act('editor-is-retained',path=path) and driver.act('editor-size',path=path)==size;driver.act('undo',path=path);assert driver.act('construction-source',path=path)==source;assert not driver.act('debug-diagnostics')['errors']
   else:
    env.update(HICKORY_PROPOSAL_HISTORY_ORIGIN=origin,HICKORY_PROPOSAL_HISTORY_WORKSPACE=str(root),HICKORY_PROPOSAL_HISTORY_TOKEN=original['token'],HICKORY_PROPOSAL_HISTORY_FINGERPRINT=original['fingerprint']);name='testLiveHistoricalProposalKeepsDirtyEditorAndHasNoApply' if args.host=='apple' else 'liveHistoricalProposalKeepsDirtyEditorAndHasNoApply';report=folder/'native.xml'
    if args.host=='apple':command=['swift','test','--jobs','2','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','AgentProposalTests.'+name+'$','--xunit-output',str(report)]+(['--skip-build'] if args.skip_build else [])
    else:report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.AgentProposalsTest.xml';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.AgentProposalsTest.'+name,'--rerun']
    report.unlink(missing_ok=True);log=ROOT/'target'/f'proposal-history-{args.host}-actual.log'
    with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT,timeout=600)
    assert result.returncode==0,str(log)
    if report.exists():
     rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['failure','error','skipped']),str(report)
    else:
     text=log.read_text();assert args.host=='apple' and re.search(re.escape(name)+r'.*passed',text) and 'Executed 1 test, with 0 failures' in text and 'Test skipped' not in text,str(log)
   for path,text in protocol.FILES.items():assert (root/path).read_text()==text,path
   assert not (root/'.hick-cache').exists();print('PASS '+args.host+': actual restart, original sealed historical reading, current dirty editor/identity/geometry or native state, unavailable Apply, preserved Undo and unchanged saved source. Native physical input/accessibility remains unverified.')
  finally:
   if driver:driver.close()
   for process in reversed(processes):
    if process is second:protocol.stop_engine(fixture,process,state)
    else:fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-agent-proposal-history-host.py"><ui:paste select="#agent-proposal-history-host-driver"/></ui:file>
</ui:doc>
