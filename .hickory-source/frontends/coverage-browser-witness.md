<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise runner results and imported coverage through the menu

For maintainers changing workspace services independently of the editor renderer.

This real-engine witness discovers a Cargo case whose body would panic if executed, then imports a labeled LCOV fixture with its explicit source identity. Adjacent test declarations place test, coverage and debug actions on the same gutter line. Opening the grid and using its actions preserves the editor and later controls. The witness also checks exact report navigation and stale refusal after a draft edit while disk stays unchanged.

<ui:copy id="simple-coverage-browser-witness">
"""Real engine discovery and report import through the compact browser menu."""
import hashlib
import importlib.util
import os
from pathlib import Path
import socket
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-coverage-ui-") as temporary:
        root=Path(temporary)/"workspace";root.mkdir()
        (root/"Cargo.toml").write_text('[package]\nname="coverage_ui_witness"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="lib.rs"\n')
        source = (
            '#[test] fn coverage_ui_case() { panic!("discovery must not execute this body"); }\n'
            '#[test] fn coverage_extra_case() { assert!(true); }\n'
        )
        path = "lib.rs"
        (root / path).write_text(source)
        environment={**os.environ,"HICKORY_STATE_DIR":str(Path(temporary)/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            driver.act("coverage-open");listing=driver.act("coverage-discover");assert "coverage_ui_case" in listing and "discovered" in listing,listing
            assert driver.act("editor-size",path=path)==size
            identity={"collector":"import-witness","version":"1","configuration":"fixture","scope":".","source_hashes":{path:hashlib.sha256(source.encode()).hexdigest()}}
            result=driver.act("coverage-import",report=f"SF:{path}\nDA:1,1\nDA:2,1\nend_of_record\n",identity=identity);assert "2 / 2 executable lines covered" in result,result
            driver.act("coverage-source", path=path)
            fixture.eventually(lambda: driver.act("caret-offset", path=path), len(source.splitlines(keepends=True)[0]))
            assert driver.act("editor-is-retained", path=path)
            fixture.eventually(lambda: driver.act("coverage-marks"), 2)
            coverage = driver.act("coverage-gutter")
            assert coverage["before"] == coverage["after"] and any(coverage["before"]["following"]), coverage
            icons = coverage["icons"]
            assert len({icon["x"] for icon in icons}) == 3 and len({icon["y"] for icon in icons}) == 1, icons
            labels = {icon["label"] for icon in icons}
            assert {"Inspect source tests: coverage_ui_case", "Debug lib.rs"} <= labels, labels
            assert any(label.startswith("Coverage ") for label in labels), labels
            driver.act("coverage-close")

            tests = driver.act("source-tests-gutter", name="coverage_ui_case")
            assert tests["before"] == tests["after"] and tests["icons"] == icons, tests
            assert "coverage_ui_case" in tests["text"], tests
            assert driver.act("editor-size", path=path) == size
            driver.act("source-tests-close")
            driver.act("append",path=path,text="// unsaved coverage draft\n");fixture.eventually(lambda:driver.act("coverage-marks"),0);draft=driver.act("search-source",path=path)
            driver.act("coverage-open");driver.act("coverage-source",path=path,stale=True);assert "Source changed" in driver.act("coverage-state")
            driver.act("coverage-close");assert driver.act("search-source",path=path)==draft;assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            print("PASS: actual runner discovery, report import, exact/stale navigation and retained editor geometry/draft")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-coverage-browser.py"><ui:paste select="#simple-coverage-browser-witness"/></ui:file>

## Collect actual failed tests through the shared surface

This fixture owns a tiny Python project and its engine. Discovery must not execute either test body. An actual pytest-cov run retains both a passed and failed result plus measured coverage; rerun-failed executes only the failed identity and preserves both records. The browser driver enters through the existing menu and waits for the selected running snapshot to refresh automatically, then checks real marks, current and stale source navigation, unchanged code geometry and an unrelated draft’s native Undo/Redo. No report is imported or fabricated.

The same fixture invokes an exact native acceptance with no skipped success: Apple drives the actual bundled WebKit surface above its retained native editor; Android drives the production fixed-message controller and native text/scroll state against the owned engine. Those controller and desktop witnesses do not certify physical device input, assistive technology or Android WebView rendering.

<ui:copy id="collector-host-fixture">
#!/usr/bin/env python3
"""Owned real pytest collector flows through the compact host adapters."""
import argparse,importlib.util,json,os,re,signal,socket,subprocess,sys,tempfile,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
APP='def classify(value):\n    if value < 0:\n        return -1\n    return 1\n'
TEST='from pathlib import Path\nfrom app import classify\ndef record(name):\n    with Path(".hick-cache/executions").open("a") as out: out.write(name+"\\n")\ndef test_positive():\n    record("positive")\n    assert classify(1) == 1\ndef test_negative():\n    record("negative")\n    assert classify(-1) == 1\n'
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['browser','apple','android']);parser.add_argument('--python-env',type=Path,default=ROOT/'target/thread-debug-env');parser.add_argument('--skip-build',action='store_true');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 with tempfile.TemporaryDirectory(prefix='hickory-collector-host-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.hickory-collector-fixture').write_text('owned-collector-v1\n');(root/'app.py').write_text(APP);(root/'test_app.py').write_text(TEST);(root/'notes.py').write_text('note = 1\n');(root/'pyproject.toml').write_text('[project]\nname="collector-fixture"\nversion="0.0.0"\n[tool.pytest.ini_options]\njunit_family="xunit1"\n');(root/'.venv').symlink_to(args.python_env.resolve(),target_is_directory=True)
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};processes=[]
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   ui=f'http://127.0.0.1:{port}';engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window',ui],env=env);processes.append(engine);origin=fixture.origin(engine);env.update(HICKORY_COLLECTOR_ORIGIN=origin,HICKORY_COLLECTOR_WORKSPACE=str(root))
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);url=fixture.origin(vite)+'/simple.html';command=['node',str(ROOT/'scripts/collector-browser-driver.mjs'),url]
   elif args.host=='apple':
    name='testLiveOwnedCollectorSurfaceResultsRerunAndSourceGuards';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','CoverageViewTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])
   else:
    name='liveOwnedCollectorBridgeResultsRerunAndSourceGuards';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.CoverageBindingTest.'+name,'--rerun']
   result=subprocess.run(command,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);print(result.stdout[-6000:],end='');assert result.returncode==0,result.returncode
   if args.host=='apple':assert re.search(re.escape(name)+r'.*passed',result.stdout) and 'Executed 1 test, with 0 failures' in result.stdout and 'Test skipped' not in result.stdout
   if args.host=='android':
    report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.CoverageBindingTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(k) is not None for k in ['skipped','failure','error'])
   assert (root/'app.py').read_text()==APP and (root/'test_app.py').read_text()==TEST and (root/'notes.py').read_text()=='note = 1\n';events=(root/'.hick-cache/executions').read_text().splitlines();assert sorted(events)==['negative','negative','positive'],events
   print(f'PASS {args.host}: actual pytest discovery/run/failed coverage/rerun-failed/history, unchanged source and retained UI state. Physical input/accessibility remains unverified.')
  finally:
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>

<ui:copy id="collector-browser-driver">
import {chromium} from '../apps/web/node_modules/@playwright/test/index.mjs';
import assert from 'node:assert/strict';
const browser=await chromium.launch({headless:true});const page=await browser.newPage({viewport:{width:1100,height:750}});
const errors=[];page.on('pageerror',error=>errors.push(String(error)));
const menu=async(name,action)=>{await page.locator('nav').getByText(name,{exact:true}).click();await page.getByRole('button',{name:action,exact:true}).click();};
const editor=path=>page.locator('.cm-content').and(page.locator(`[aria-label=${JSON.stringify(path)}]`));
const source=async path=>(await editor(path).locator('.cm-line').allTextContents()).join('\n');
const open=async path=>{await page.getByRole('button',{name:path,exact:true}).click();await editor(path).waitFor();};
const dialog=()=>page.getByRole('dialog',{name:'Tests and code coverage',exact:true});
const currentId=()=>dialog().locator('main>p').filter({hasText:'Run '}).locator('code').innerText();
const waitFinished=async()=>{await dialog().getByRole('heading',{name:/automated · failed · current source/}).waitFor({timeout:60000});await dialog().getByText(/executable lines covered/).waitFor();};
try{
 await page.goto(process.argv[2]);await open('notes.py');await editor('notes.py').click();await page.keyboard.press(process.platform==='darwin'?'Meta+ArrowDown':'Control+End');await page.keyboard.insertText('# retained unsaved note\n');const note=await source('notes.py');await open('app.py');const retained=await editor('app.py').elementHandle(),size=await editor('app.py').locator('..').evaluate(node=>({width:node.clientWidth,height:node.clientHeight}));
 await menu('Edit','Tests and coverage');await dialog().getByLabel('Project or source file',{exact:true}).fill('.');await dialog().getByRole('button',{name:'Discover runner cases',exact:true}).click();await dialog().getByText(/discovered · test_app.py::test_positive/).waitFor({timeout:30000});await dialog().getByText(/discovered · test_app.py::test_negative/).waitFor();
 await dialog().getByRole('button',{name:'Run saved project with coverage',exact:true}).click();await waitFinished();const first=await currentId();assert.match(first,/^[0-9a-f]{32}$/);await dialog().getByText(/passed · test_app.py::test_positive/).waitFor();const failure=dialog().getByText(/failed · test_app.py::test_negative/);await failure.click();await failure.locator('..').locator('pre').filter({hasText:/assert -1 == 1/}).waitFor();assert.equal(await editor('app.py').evaluate((node,old)=>node===old,retained),true);assert.deepEqual(await editor('app.py').locator('..').evaluate(node=>({width:node.clientWidth,height:node.clientHeight})),size);await page.waitForFunction(()=>document.querySelectorAll('.cm-debug-rail button[title^="Coverage "]').length>0);
 await dialog().getByRole('button',{name:'Rerun failed tests from selected run',exact:true}).click();await page.waitForFunction(first=>{const panel=document.querySelector('[aria-label="Tests and code coverage"]');return panel&&[...panel.querySelectorAll('main>p code')].some(node=>node.textContent!==first);},first);await waitFinished();const second=await currentId();assert.notEqual(first,second);await dialog().getByText(/failed · test_app.py::test_negative/).waitFor();assert.equal(await dialog().getByText(/passed · test_app.py::test_positive/).count(),0);assert.ok(await dialog().getByRole('button',{name:new RegExp(first)}).count());assert.ok(await dialog().getByRole('button',{name:new RegExp(second)}).count());
 await dialog().getByRole('button',{name:new RegExp(first)}).click();await dialog().getByText(/passed · test_app.py::test_positive/).waitFor();await dialog().getByRole('button',{name:'app.py',exact:true}).first().click();await dialog().waitFor({state:'detached'});assert.equal(await editor('app.py').evaluate((node,old)=>node===old,retained),true);
 await editor('app.py').click();await page.keyboard.press(process.platform==='darwin'?'Meta+ArrowDown':'Control+End');await page.keyboard.insertText('# unsaved after measurement\n');const draft=await source('app.py');await page.waitForFunction(()=>document.querySelectorAll('.cm-debug-rail button[title^="Coverage "]').length===0);await menu('Edit','Tests and coverage');await dialog().getByText(/executable lines covered/).waitFor();await dialog().getByRole('button',{name:'app.py',exact:true}).first().click();await dialog().getByRole('alert').filter({hasText:'Source changed'}).waitFor();await dialog().getByRole('button',{name:'Close coverage',exact:true}).click();assert.equal(await source('app.py'),draft);assert.equal(await editor('app.py').evaluate((node,old)=>node===old,retained),true);assert.deepEqual(await editor('app.py').locator('..').evaluate(node=>({width:node.clientWidth,height:node.clientHeight})),size);
 await open('notes.py');assert.equal(await source('notes.py'),note);await editor('notes.py').click();await page.keyboard.press(process.platform==='darwin'?'Meta+z':'Control+z');assert.equal(await source('notes.py'),'note = 1\n');await page.keyboard.press(process.platform==='darwin'?'Meta+Shift+z':'Control+Shift+z');assert.equal(await source('notes.py'),note);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({host:'browser',first,second,collector:'pytest-cov',discovery:true,failed_results:true,automatic_completion:true,rerun_failed:true,retained_history:true,current_navigation:true,stale_navigation_refused:true,coverage_marks_retired:true,retained_editor:true,unrelated_undo:true}));
}finally{await browser.close();}
</ui:copy>

<ui:file path="scripts/test-collector-hosts.py"><ui:paste select="#collector-host-fixture"/></ui:file>
<ui:file path="scripts/collector-browser-driver.mjs"><ui:paste select="#collector-browser-driver"/></ui:file>
</ui:doc>
