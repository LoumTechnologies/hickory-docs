<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compose a complete conflict group without conflating drafts, saved sources and Git

For maintainers exercising the existing floating repository review against an
owned Git repository and real engine. Reuse the service fixture’s literal path
and actual unmerged index; never touch this checkout’s Git index or post-commit
state. Each destructive failure case owns a separate repository and process.
The successful path explicitly composes a destination with retained identity,
previews consequences, applies draft-only text, undoes/redoes that coordinated
change, then separately saves and stages. Source changes, index changes and a
replacement root must refuse the captured publication while retaining editors.

<ui:copy id="git-resolution-browser-real-engine">
"""Owned actual Git group composition: draft, saved source and index are separate."""
import importlib.util,os,socket,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def main():
 fixture=module('fixture',ROOT/'scripts/test-simple-browser.py');shared=module('resolution_fixture',ROOT/'scripts/test-git-conflict-resolutions.py')
 with tempfile.TemporaryDirectory(prefix='hickory-git-resolution-browser-') as temporary:
  folder=Path(temporary)
  for case in ['complete','index','source','namespace']:
   root=folder/case;path=shared.make(root,'text');original=(root/path).read_text();index=(root/'.git/index').read_bytes();head=shared.git(root,'rev-parse','HEAD');resolved='composed = 41\n';draft=original+'\n# retained unrelated draft\n';processes=[];driver=None
   environment={**os.environ,'HICKORY_STATE_DIR':str(folder/(case+'-state'))}
   try:
    with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
    engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',f'http://127.0.0.1:{port}'],env=environment);processes.append(engine);backend=fixture.origin(engine)
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**environment,'HICKORY_API_ORIGIN':backend});processes.append(vite)
    driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path);driver.act('construction-set-source',path=path,source=draft);position=driver.act('editor-position',path=path,anchor=3,head=7,y=0)
    driver.act('review-open');driver.act('review-select',path=path,staged=False);text=driver.act('resolution-compose');assert 'explicitly retain every destination' in text,text
    driver.act('resolution-output',path=path,text=resolved);preview=driver.act('resolution-preview');assert 'Complete immutable source/resource' in preview and 'composed = 41' in preview,preview
    assert (root/path).read_text()==original and (root/'.git/index').read_bytes()==index;assert driver.act('construction-source',path=path)==draft
    assert driver.act('editor-position',path=path)==position
    if case=='complete':
     applied=driver.act('resolution-act',kind='draft');assert 'Applied coordinated source drafts' in applied,applied;assert driver.act('construction-source',path=path)==resolved;assert (root/path).read_text()==original and (root/'.git/index').read_bytes()==index
     driver.act('workspace-history',redo=False);driver.act('change-close');assert driver.act('construction-source',path=path)==draft
     driver.act('workspace-history',redo=True);driver.act('change-close');assert driver.act('construction-source',path=path)==resolved
     driver.act('resolution-preview');notice=driver.act('resolution-act',kind='save');assert 'Complete saved group confirmed' in notice,notice
     assert (root/path).read_text()==resolved and (root/'.git/index').read_bytes()==index
     state=driver.act('resolution-state');assert state['stage'],state
     assert driver.act('editor-is-retained',path=path) and driver.act('editor-size',path=path)==size
     notice=driver.act('resolution-act',kind='stage');assert 'Complete saved conflict group staged' in notice,notice
     assert shared.git(root,'ls-files','--unmerged','--',path)==b'';assert shared.git(root,'show',':0:'+path)==resolved.encode();assert shared.git(root,'rev-parse','HEAD')==head
     driver.act('review-close');assert driver.act('editor-is-retained',path=path);driver.act('undo',path=path);assert driver.act('construction-source',path=path)==draft;driver.act('redo',path=path);assert driver.act('construction-source',path=path)==resolved
     assert (root/path).read_text()==resolved and shared.git(root,'show',':0:'+path)==resolved.encode()
    elif case=='source':
     driver.act('source-tests-edit',path=path,text='\n# later source edit\n');changed=driver.act('construction-source',path=path);notice=driver.act('resolution-act',kind='save');assert 'changed' in notice.lower(),notice;assert driver.act('construction-source',path=path)==changed;assert (root/path).read_text()==original and (root/'.git/index').read_bytes()==index
    elif case=='index':
     # A saved index generation is checked independently of editor/source revisions.
     (root/'clean.py').write_text('changed unrelated index\n');shared.git(root,'add','--','clean.py');changed=(root/'.git/index').read_bytes();notice=driver.act('resolution-act',kind='save');assert 'changed' in notice.lower() or 'unconfirmed' in notice.lower(),notice;assert (root/path).read_text()==original and (root/'.git/index').read_bytes()==changed;assert driver.act('construction-source',path=path)==draft
    else:
     # Reusing the same root spelling cannot borrow the original workspace authority.
     moved=root.with_name(case+'-retained');root.rename(moved);root.mkdir();(root/path).write_text('independent replacement\n');notice=driver.act('resolution-act',kind='save');assert 'changed' in notice.lower() or 'unconfirmed' in notice.lower(),notice;assert (root/path).read_text()=='independent replacement\n' and (moved/path).read_text()==original and (moved/'.git/index').read_bytes()==index;assert driver.act('construction-source',path=path)==draft
    assert not driver.act('debug-diagnostics')['errors']
    print('PASS actual browser conflict '+case,flush=True)
   finally:
    if driver:driver.close()
    for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>

This controlled browser flow is not physical-input or accessibility acceptance.
Complex rename/delete groups additionally rely on the shared service witnesses;
this browser witness does not infer such relationships from names.

<ui:file path="scripts/test-git-resolution-browser.py"><ui:paste select="#git-resolution-browser-real-engine"/></ui:file>
</ui:doc>
