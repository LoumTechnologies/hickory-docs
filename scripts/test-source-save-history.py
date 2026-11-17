#!/usr/bin/env python3
"""Owned-engine strict source-save and draft-history authority checks."""
import hashlib, importlib.util, json, os, tempfile, time, urllib.error, urllib.parse, urllib.request, uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def digest(value):return hashlib.sha256(value.encode()).hexdigest()
def request(origin,method,route,body=None):
    raw=None if body is None else json.dumps(body).encode()
    with urllib.request.urlopen(urllib.request.Request(origin+route,data=raw,method=method,headers={'Content-Type':'application/json'}),timeout=30) as response:return json.load(response)
def refused(action):
    try:action()
    except urllib.error.HTTPError as error:
        assert error.code in (400,403,404,409,422), (error.code,error.read());return
    raise AssertionError('A stale or foreign source act was admitted')
def get_file(origin,path):return request(origin,'GET','/api/file?path='+urllib.parse.quote(path,safe=''))
def guard_file(origin,path):
    deadline=time.monotonic()+20
    while True:
        value=get_file(origin,path)
        if value.get('save_guard'):return value
        if time.monotonic()>deadline:raise AssertionError(value)
        time.sleep(.05)
def save_body(path,loaded,text):return {'version':1,'path':path,'content':text,'base_hash':loaded['hash'],'save_guard':loaded['save_guard'],'request_id':uuid.uuid4().hex}
def check_saved(value,body,path,text):
    assert value['version']==1 and value['history_protocol']==1 and value['request_id']==body['request_id'] and value['path']==path and value['source_hash']==digest(text) and value['save_guard'] and value['history']['outcome']=='published',value
def find(nodes,path):
    for row in nodes:
        if row['path']==path:return row
        found=find(row.get('children',[]),path)
        if found:return found
def suite(origin,root):
    path='source λ%?#.txt';file=root/path;before=file.read_text();loaded=guard_file(origin,path)
    body=save_body(path,loaded,'saved 😀\n');saved=request(origin,'PUT','/api/source-history/file',body);check_saved(saved,body,path,body['content']);identity=file.stat().st_ino
    replay=request(origin,'PUT','/api/source-history/file',body);assert replay==saved and file.stat().st_ino==identity
    refused(lambda:request(origin,'PUT','/api/source-history/file',{**body,'content':'different intent'}));assert file.read_text()==body['content']
    history=request(origin,'GET','/api/source-history?path='+urllib.parse.quote(path,safe=''));assert history['namespace']==str(root) and history['path']==path and history['current']['text']==body['content']
    entry=next(row for row in history['entries'] if row['id']==body['request_id']);assert entry['outcome']=='published' and entry['available'] and entry['before']['text']==before
    restore={'version':1,'namespace':history['namespace'],'workspace':history['workspace'],'path':path,'id':entry['id'],'fingerprint':entry['fingerprint'],'current_identity':history['current_identity'],'current_hash':history['current']['sha256'],'draft_hash':digest('unsaved draft'),'side':'before'}
    draft=request(origin,'POST','/api/source-history/restore-draft',restore);assert draft['source']==before and draft['consequences']=='unknown' and draft['draft_hash']==restore['draft_hash'];assert file.read_text()==body['content']
    for key in ['namespace','workspace','path','fingerprint','current_identity','current_hash']:
        refused(lambda key=key:request(origin,'POST','/api/source-history/restore-draft',{**restore,key:'foreign'}))
    fresh=guard_file(origin,path);stale=save_body(path,fresh,'must not replace recreation');replacement=root/'replacement.tmp';replacement.write_text(file.read_text());os.replace(replacement,file)
    refused(lambda:request(origin,'PUT','/api/source-history/file',{**stale,'force':True}));assert file.read_text()==body['content']
    refused(lambda:request(origin,'PUT','/api/source-history/file',{**body,'request_id':uuid.uuid4().hex,'save_guard':None}));assert file.read_text()==body['content']
    sibling='source λ%2F?#.txt';other=guard_file(origin,sibling);refused(lambda:request(origin,'PUT','/api/source-history/file',{**save_body(sibling,other,'wrong alias'),'save_guard':fresh['save_guard']}));assert (root/sibling).read_text()=='distinct sibling\n'
    generated=get_file(origin,'generated.py');refused(lambda:request(origin,'PUT','/api/source-history/file',{'version':1,'path':'generated.py','content':'changed','base_hash':generated['hash'],'save_guard':generated.get('save_guard') or fresh['save_guard'],'request_id':uuid.uuid4().hex}));assert (root/'generated.py').read_text()=='value = 1\n'
    readonly_body=save_body(path,guard_file(origin,path),'read only')
    file.chmod(0o444)
    try:refused(lambda:request(origin,'PUT','/api/source-history/file',readonly_body))
    finally:file.chmod(0o644)
    files=request(origin,'GET','/api/files');doc=find(files['tree'],'note.md');assert doc and doc.get('doc_id'),files
    loaded_doc=request(origin,'GET','/api/docs/'+urllib.parse.quote(doc['doc_id'],safe=''));assert loaded_doc.get('save_guard')
    doc_body={'version':1,'source':'# New document 😀\n','expected':{'source':loaded_doc['source'],'disk':loaded_doc['source']},'save_guard':loaded_doc['save_guard'],'request_id':uuid.uuid4().hex}
    route='/api/source-history/docs/'+urllib.parse.quote(doc['doc_id'],safe='');result=request(origin,'PUT',route,doc_body);check_saved(result,doc_body,'note.md',doc_body['source']);identity=(root/'note.md').stat().st_ino;assert request(origin,'PUT',route,doc_body)==result and (root/'note.md').stat().st_ino==identity
    refused(lambda:request(origin,'PUT',route,{**doc_body,'source':'different intent'}));assert (root/'note.md').read_text()==doc_body['source']
    assert not (root/'.hick-cache/resource-changes').exists(),'Ordinary history must use its private workspace store'
    print('PASS strict source history: exact file/doc publication and retry, changed-intent/recreation/namespace/alias/generated/read-only refusal, private retention and draft-only restore',flush=True)
def creation_suite(origin,root):
    tree=request(origin,'GET','/api/files');workspace=tree['workspace_id'];assert workspace
    ownership={'context':'construction.md','project_entry':'project.md','edit':{'changes':{}},'ownership_sources':{}}
    body={'version':1,'workspace':workspace,'path':'created λ%?#.txt','source':'created 😀\n','request_id':uuid.uuid4().hex,'ownership':ownership,'saved_guards':{}}
    route='/api/source-history/create';saved=request(origin,'POST',route,body);check_saved(saved,body,body['path'],body['source']);assert saved['workspace']==workspace and saved['namespace']==str(root) and saved['source']==body['source']
    file=root/body['path'];inode=file.stat().st_ino;assert request(origin,'POST',route,body)==saved and file.stat().st_ino==inode
    for changed in [{'source':'changed'},{'workspace':'foreign'},{'ownership':{**ownership,'context':'foreign.md'}}]:
        refused(lambda changed=changed:request(origin,'POST',route,{**body,**changed}))
    construction=find(request(origin,'GET','/api/files')['tree'],'construction.md');assert construction and construction.get('doc_id')
    project=request(origin,'GET','/api/docs/'+urllib.parse.quote(construction['doc_id'],safe=''));assert project.get('save_guard')
    dirty='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="draft-owned.txt">generated</h:file></h:doc>'
    captured={**ownership,'ownership_sources':{'construction.md':dirty}}
    attempt={**body,'path':'draft-owned.txt','request_id':uuid.uuid4().hex,'ownership':captured,'saved_guards':{'construction.md':project['save_guard']}}
    refused(lambda:request(origin,'POST',route,attempt));assert not (root/'draft-owned.txt').exists()
    refused(lambda:request(origin,'POST',route,{**attempt,'path':'missing-guard.txt','request_id':uuid.uuid4().hex,'saved_guards':{}}));assert not (root/'missing-guard.txt').exists()
    refused(lambda:request(origin,'POST',route,{**body,'path':'generated.py','request_id':uuid.uuid4().hex}));assert (root/'generated.py').read_text()=='value = 1\n'
    refused(lambda:request(origin,'POST',route,{**body,'path':'unaddressed-owned.txt','request_id':uuid.uuid4().hex,'unaddressed_sources':{'retained-untitled':dirty}}));assert not (root/'unaddressed-owned.txt').exists()
    ordinary={**body,'path':'with-other-note.txt','request_id':uuid.uuid4().hex,'unaddressed_sources':{'retained-untitled':'Ordinary unassigned note'}}
    confirmed=request(origin,'POST',route,ordinary);check_saved(confirmed,ordinary,ordinary['path'],ordinary['source'])
    replacement=root/'replacement.tmp';replacement.write_text(body['source']);os.replace(replacement,file)
    refused(lambda:request(origin,'POST',route,body));assert file.read_text()==body['source']
    print('PASS strict source creation: direct guarded full-text confirmation, exact replay/no second inode, changed intent/workspace/ownership refusal, dirty generated ownership, missing construction guard, saved generated target, unaddressed construction refusal/plain-note acceptance and same-byte recreation refusal',flush=True)
def missing_suite(origin,root):
    path='source λ%?#.txt';file=root/path;source='confirmed saved text before deletion\n';saved=save_body(path,guard_file(origin,path),source);request(origin,'PUT','/api/source-history/file',saved);file.unlink()
    def history(name):return request(origin,'GET','/api/source-history?path='+urllib.parse.quote(name,safe=''))
    value=history(path);entry=next(row for row in value['entries'] if row['id']==saved['request_id']);assert value['current']['kind']=='missing' and value['current_identity'].startswith('missing:') and value['restore_target']=='missing-source-draft' and entry['available'] and not entry['retry_allowed']
    body={'version':1,'namespace':value['namespace'],'workspace':value['workspace'],'path':path,'id':entry['id'],'fingerprint':entry['fingerprint'],'current_identity':value['current_identity'],'current_hash':'missing','draft_hash':digest('retained unsaved text'),'side':'after'}
    draft=request(origin,'POST','/api/source-history/restore-draft',body);assert draft['source']==source and draft['restore_target']=='missing-source-draft' and draft['source_effects'] is False and not file.exists()
    file.write_text('');refused(lambda:request(origin,'POST','/api/source-history/restore-draft',body));assert file.exists() and file.read_bytes()==b'';file.unlink();refused(lambda:request(origin,'POST','/api/source-history/restore-draft',body));assert not file.exists()
    created='created λ%?#.txt';created_file=root/created;created_file.unlink();value=history(created);entry=next(row for row in value['entries'] if row['before']['kind']=='missing');assert entry['available'] and not entry['retry_allowed']
    absent={'version':1,'namespace':value['namespace'],'workspace':value['workspace'],'path':created,'id':entry['id'],'fingerprint':entry['fingerprint'],'current_identity':value['current_identity'],'current_hash':'missing','draft_hash':digest('retained created draft'),'side':'before'}
    refused(lambda:request(origin,'POST','/api/source-history/restore-draft',absent));assert not created_file.exists();restored=request(origin,'POST','/api/source-history/restore-draft',{**absent,'side':'after'});assert restored['source']==entry['after']['text'] and restored['source_effects'] is False and not created_file.exists()
    print('PASS deleted-source history: Missing distinct from empty File, confirmed text becomes only unmaterialized draft, no retry/identity promotion or file creation, independent replacement and absence-ABA refusal, Missing-before side never coerced to empty text',flush=True)
def main():
    spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix='hickory-history-service-') as folder:
        temporary=Path(folder);root=temporary/'workspace';root.mkdir();root=root.resolve()
        for path,text in {'source λ%?#.txt':'before\n','source λ%2F?#.txt':'distinct sibling\n','note.md':'# Original\n','project.md':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="construction.md"/></h:doc>\n','construction.md':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="generated.py">value = 1\n</h:file></h:doc>\n','generated.py':'value = 1\n'}.items():(root/path).write_text(text)
        process=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(temporary/'state')})
        try:
            origin=fixture.origin(process);suite(origin,root);creation_suite(origin,root);missing_suite(origin,root)
        finally:fixture.stop(process)
if __name__=='__main__':main()
