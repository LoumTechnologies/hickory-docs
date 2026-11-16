"""Focused real-engine read-only template planning and authority refusals."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import urllib.error
import urllib.request
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location('template_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-template-service-') as temporary:
  root=Path(temporary)/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.hickory').mkdir()
  source='😀 old\r\n';(root/'source.py').write_bytes(source.encode())
  generated='print(1)\n';(root/'generated.py').write_text(generated)
  author='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="generated.py">\n'+generated+'</h:file>\n</h:doc>\n';(root/'author.md').write_text(author)
  manifest=root/'.hickory/templates.json';definition={'version':1,'templates':[{'id':'workspace.selected','label':'Selection','description':'Captured selection and line','language':'python','syntax':'${TM_SELECTED_TEXT}:${TM_LINE_NUMBER}$0'}]};manifest.write_text(json.dumps(definition))
  engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(Path(temporary)/'state')})
  try:
   origin=fixture.origin(engine)
   def ask(route,body,rejected=False):
    request=urllib.request.Request(origin+'/api/templates/'+route,data=json.dumps(body).encode(),headers={'Content-Type':'application/json'})
    try:
     with urllib.request.urlopen(request,timeout=30) as response:value=json.load(response)
    except urllib.error.HTTPError as error:
     assert rejected,(error.code,error.read().decode());return error.read().decode()
    assert not rejected,value
    return value
   context={'version':1,'path':'source.py','source':source,'selection':{'from':3,'to':6}}
   catalog=ask('catalog',context);assert catalog['template_protocol']==1 and catalog['available'];assert catalog['source_hash']==hashlib.sha256(source.encode()).hexdigest();assert any(entry['id']=='workspace.selected' and entry['origin']=='workspace' for entry in catalog['entries'])
   request={**context,'catalog_fingerprint':catalog['catalog_fingerprint'],'template_id':'workspace.selected'}
   plan=ask('plan',request);assert plan['snippet']['text']=='old:1' and plan['replacement']==context['selection'];assert ask('plan',request)==plan
   # Revoking a definition after preview cannot silently accept a different plan.
   definition['templates'][0]['syntax']='changed $0';manifest.write_text(json.dumps(definition));assert 'catalog changed' in ask('plan',request,True)
   definition['templates'].append({'id':'workspace.empty_choices','label':'Bounded empty choices','description':'Must refuse mirrored metadata amplification','language':'python','syntax':'${1|'+','*4095+'|}'+'$1'*8})
   manifest.write_text(json.dumps(definition));bounded=ask('catalog',context)
   excessive=next(entry for entry in bounded['entries'] if entry['id']=='workspace.empty_choices');assert not excessive['available'] and 'metadata' in excessive['reason']
   ask('plan',{**context,'catalog_fingerprint':bounded['catalog_fingerprint'],'template_id':'workspace.empty_choices'},True)
   for target in ['generated.py','../outside.py']:
    ask('catalog',{**context,'path':target},True)
   if os.name!='nt':
    (root/'alias.py').symlink_to(root/'generated.py');assert 'aliases' in ask('catalog',{**context,'path':'alias.py'},True)
    foreign=Path(temporary)/'foreign';foreign.mkdir();(root/'escaped').symlink_to(foreign,target_is_directory=True);assert 'escapes' in ask('catalog',{**context,'path':'escaped/new.py'},True)
   ask('plan',{**request,'command':'no executable template commands'},True)
   unknown=ask('catalog',{**context,'path':'plain.unknown'});assert not unknown['available'] and unknown['entries']==[]
   assert (root/'source.py').read_bytes()==source.encode() and (root/'author.md').read_text()==author and (root/'generated.py').read_text()==generated
   print('PASS actual template catalog/plan, unchanged repeat, revoked catalog, generated/alias/escape refusals and unchanged authors')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
