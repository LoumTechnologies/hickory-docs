#!/usr/bin/env python3
"""Exercise reviewed conflict publication against isolated real Git repositories and an owned engine."""
import argparse, importlib.util, json, os, subprocess, tempfile, urllib.error, urllib.parse, urllib.request, uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PREFIX='/api/git/conflict-resolution/'
def git(root,*args,ok=True):
    p=subprocess.run(['git','-C',str(root),'--literal-pathspecs',*args],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    if ok and p.returncode: raise RuntimeError(p.stderr.decode())
    return p.stdout

def make(root,kind):
    root.mkdir();git(root,'init','-q','-b','main');git(root,'config','user.name','Conflict fixture');git(root,'config','user.email','fixture@example.invalid')
    path='literal[1] λ%?#.py' if kind in ('text','noop','stale','index','lock','native') else 'case.py'
    (root/path).write_text('base\n');(root/'clean.py').write_text('clean\n');git(root,'add','--',path,'clean.py');git(root,'commit','-qm','base');git(root,'checkout','-qb','incoming')
    if kind=='rename':git(root,'mv','--',path,'right.py')
    else:(root/path).write_text('incoming\n')
    git(root,'add','-A');git(root,'commit','-qm','incoming');git(root,'checkout','-q','main')
    if kind=='rename':git(root,'mv','--',path,'left.py')
    elif kind=='delete':(root/path).unlink()
    else:(root/path).write_text('local\n')
    git(root,'add','-A');git(root,'commit','-qm','local');git(root,'merge','--no-edit','incoming',ok=False)
    assert git(root,'ls-files','-u','-z')
    return path

def request(origin,route,body=None):
    req=urllib.request.Request(origin+route,data=None if body is None else json.dumps(body).encode(),headers={'Content-Type':'application/json'})
    with urllib.request.urlopen(req,timeout=30) as response:return json.load(response)
def refused(action):
    try:action()
    except urllib.error.HTTPError as error:
        assert error.code in (400,409,422,503),error.read();return
    raise AssertionError('Unsafe action was admitted')
def prepare(origin,members,outputs):
    reading=request(origin,PREFIX+'read',{'version':1,'members':members})
    paths=sorted(set(members+[v['path'] for v in outputs]))
    capture={'context':uuid.uuid4().hex,'edit':{'changes':{'hick:///'+urllib.parse.quote(path,safe='/'):[] for path in paths}},'drafts':{},'project_entry':None,'ownership_sources':{}}
    inputs=request(origin,'/api/changes/resource-inputs',capture)
    return request(origin,PREFIX+'prepare',{'version':1,'reading':reading,'members':members,'outputs':outputs,'inputs':inputs,'capture':capture})
def output(path,identity,text):return {'path':path,'identity_from':identity,'mode':'100644','content':{'kind':'text','text':text}}
def act(origin,review,kind,identity=None):return request(origin,PREFIX+kind,{'version':1,'token':review['token'],'fingerprint':review['fingerprint'],'request_id':identity or uuid.uuid4().hex})
def suite(origin,root,kind,path):
    read=request(origin,PREFIX+'read',{'version':1,'members':[]});members=[row['path'] for row in read['paths']]
    if kind=='rename':outputs=[output('resolved.py','left.py','resolved rename\n')]
    elif kind=='delete':outputs=[]
    else:outputs=[output(path,path,(root/path).read_text() if kind=='noop' else 'composed = 41\n')]
    review=prepare(origin,members,outputs);assert review['resolution_protocol']==1
    before=(root/'.git/index').read_bytes();working={v:(root/v).read_bytes() if (root/v).exists() else None for v in members}
    if kind in ('stale','index','lock'):
        if kind=='stale':(root/path).write_text('foreign working edit\n')
        if kind=='index':(root/'clean.py').write_text('changed index\n');git(root,'add','--','clean.py');before=(root/'.git/index').read_bytes()
        if kind=='lock':(root/'.git/index.lock').write_text('foreign lock')
        identity=uuid.uuid4().hex;refused(lambda:act(origin,review,'save',identity));assert (root/'.git/index').read_bytes()==before
        status=request(origin,PREFIX+'status?request_id='+identity);assert status['state']=='completed' and status['outcome'] is None and status['error']
        if kind=='lock':
            assert (root/'.git/index.lock').read_text()=='foreign lock';(root/'.git/index.lock').unlink();assert request(origin,PREFIX+'validate',{'version':1,'token':review['token'],'fingerprint':review['fingerprint']})['current']
            refused(lambda:act(origin,review,'save'))
        else:refused(lambda:request(origin,PREFIX+'validate',{'version':1,'token':review['token'],'fingerprint':review['fingerprint']}))
        if kind!='stale':assert all(((root/v).read_bytes() if (root/v).exists() else None)==body for v,body in working.items())
        return
    if kind=='noop':assert not review['needs_save']
    else:
        identity=uuid.uuid4().hex;saved=act(origin,review,'save',identity);assert saved['request_id']==identity and saved['outcome']['complete'],saved
        assert saved['saved_sources']==review['saved_sources'];assert (root/'.git/index').read_bytes()==before
        assert request(origin,PREFIX+'status?request_id='+identity)['outcome']==saved
    if kind=='stage-lock':
        (root/'.git/index.lock').write_text('foreign stage lock');identity=uuid.uuid4().hex
        refused(lambda:act(origin,review,'stage',identity));assert (root/'.git/index').read_bytes()==before
        status=request(origin,PREFIX+'status?request_id='+identity);assert status['state']=='completed' and status['outcome'] is None
        assert (root/'.git/index.lock').read_text()=='foreign stage lock';(root/'.git/index.lock').unlink()
        proof={'version':1,'token':review['token'],'fingerprint':review['fingerprint'],'stage':True}
        assert request(origin,PREFIX+'validate',proof)['stage']
        refused(lambda:act(origin,review,'stage'))
        review=prepare(origin,members,outputs);assert not review['needs_save']
    identity=uuid.uuid4().hex;staged=act(origin,review,'stage',identity);assert staged['complete'] and not staged['index_uncertain'];assert staged['token']==review['token'] and staged['request_id']==identity and staged['review_fingerprint']==review['fingerprint']
    assert not git(root,'ls-files','-u','-z');assert git(root,'show',':clean.py')==b'clean\n'
    for value in outputs:assert git(root,'show',':'+value['path'])==value['content']['text'].encode()
    if kind=='delete':assert not (root/path).exists()
    if kind=='rename':assert not (root/'left.py').exists() and not (root/'right.py').exists() and not (root/'case.py').exists()

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client');parser.add_argument('--android',action='store_true');args=parser.parse_args()
    spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix='hickory-git-resolutions-') as folder:
        temporary=Path(folder)
        for kind in ['text','noop','delete','rename','stale','index','lock','stage-lock']+(['native'] if args.android else []):
            root=temporary/kind;path=make(root,kind);process=fixture.start([str(args.engine.resolve()),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(temporary/(kind+'-state'))})
            try:
                origin=fixture.origin(process)
                if kind=='native':
                    env={**os.environ,'HICKORY_NATIVE_GIT_ORIGIN':origin,'HICKORY_NATIVE_GIT_WORKSPACE':str(root),'HICKORY_NATIVE_GIT_INPUT':path}
                    subprocess.run([str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.GitConflictResolutionTest.liveNativeCompositionSaveStageAndEditorHistory','--rerun-tasks'],cwd=ROOT,env=env,check=True)
                else:suite(origin,root,kind,path)
                print('PASS actual Git conflict '+kind,flush=True)
            finally:fixture.stop(process)
if __name__=='__main__':main()
