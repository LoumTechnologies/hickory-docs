<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/tools/refine_layout.py

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="hickory-dogfood/tools/refine_layout.py">
#!/usr/bin/env python3
"""One-time reading-layout pass; leave every construction payload unchanged."""
import collections
import json
import os
from pathlib import Path
import re

BASE=Path(__file__).resolve().parents[1]
ROOT=BASE/'integrated'
MAP=BASE/'behavior-map.json'
m=json.loads(MAP.read_text())
if m.get('layout_refined'):raise SystemExit('Reading-layout migration already applied.')
OPEN='<book:doc xmlns:book="http://www.hickorydocs.com/1.0">\n'
CLOSE='\n</book:doc>\n'

def emit(path,body):
 p=ROOT/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(OPEN+body.rstrip()+'\n'+CLOSE)

def link(origin,target,label):
 return f'[{label}]({os.path.relpath(target,Path(origin).parent).replace(os.sep,"/")})'

def rebase(text,origin,destination):
 def change(match):
  url=match[2]
  if url.startswith(('#','http:','https:','mailto:','/')):return match[0]
  path,sep,anchor=url.partition('#');target=ROOT/Path(origin).parent/path
  if not target.is_file():return match[0]
  target=target.resolve().relative_to(ROOT.resolve()).as_posix()
  return link(destination,target,match[1])[:-1]+sep+anchor+')'
 return re.sub(r'\[([^\]\n]+)\]\(([^)\s]+)\)',change,text)

# Each domain index both explains the task and declares its contributing pages.
domains=[]
for domain in sorted({r['domain'] for r in m['contracts'].values()}):
 old=f'.hickory-source/read/{domain}.md';page=f'.hickory-source/guarantees/{domain}/index.md'
 body=rebase((ROOT/old).read_text().removeprefix(OPEN).removesuffix(CLOSE),old,page)
 for key,r in m['contracts'].items():
  if r['domain']==domain:body+=f'\n<book:include file="{Path(key).stem}/contract.md" />'
 emit(page,body);(ROOT/old).unlink();domains.append(page)
(ROOT/'.hickory-source/read').rmdir()
root=(ROOT/'project.md').read_text()
for page in domains:
 domain=Path(page).parent.name
 root=root.replace(f'.hickory-source/read/{domain}.md',page)
root=re.sub(r'<book:project sources="[^"]*"', '<book:project sources="'+' '.join(domains+['.hickory-source/shared/catalog.md','.hickory-source/support/catalog.md'])+'"',root)
(ROOT/'project.md').write_text(root)
m['domain_pages']=domains

# Break large catalogs into named component batches. Paths remain inside each parent.
new_catalogs=[]
for category in ('shared','support'):
 page=f'.hickory-source/{category}/catalog.md'
 groups=collections.defaultdict(list)
 for path,c in m['constructors'].items():
  if c['owner']!=category:continue
  parts=path.split('/')
  component='/'.join(parts[:2]) if parts[0] in ('crates','apps','docs','code-models','experiments') and len(parts)>1 else parts[0]
  groups[component].append((path,c))
 title='Shared implementation' if category=='shared' else 'Build inputs, unmapped code and historical guidance'
 index=f'# {title}\n\nFor: maintainers tracing affected behaviors and required project inputs.\n\nChoose the component you are changing. Each payload is constructed once. Contract references are navigation candidates; the review ledger records their limits.\n\n'
 for group,items in sorted(groups.items()):
  slug=re.sub(r'[^A-Za-z0-9_-]','-',group)
  for start in range(0,len(items),50):
   batch=items[start:start+50];child=f'.hickory-source/{category}/group-{slug}-{start//50+1:02}.md'
   body=f'# {group}: files {start+1}–{start+len(batch)}\n\nFor: maintainers editing this component or tracing its affected guarantees.\n\nOpen a construction below to edit its bytes and inspect the related contracts.\n\n'
   for path,c in batch:body+='- '+link(child,c['source'],path)+'\n'
   body+='\n'
   for path,c in batch:
    local=os.path.relpath(c['source'],Path(child).parent).replace(os.sep,'/')
    body+=f'<book:include file="{local}" />\n'
   emit(child,body);new_catalogs.append(child)
   index+='- '+link(page,child,f'{group}, {start+1}–{start+len(batch)}')+'\n'
   index+=f'<book:include file="{Path(child).name}" />\n'
 emit(page,index)
m['catalog_pages']=new_catalogs

# Retain every requirement clause while moving long details out of the overview.
for key,r in m['contracts'].items():
 if r['disposition']=='historical':continue
 page=r['page'];p=ROOT/page;text=p.read_text()
 start_marker='## Contract imported for review\n\n'
 if start_marker not in text:continue
 start=text.index(start_marker)+len(start_marker)
 end=text.index('\n\nThis is the earlier contract',start)
 normative=text[start:end]
 if len(normative.splitlines())<=85:continue
 lines=normative.splitlines(keepends=True)
 # A paragraph or a closed code fence is the smallest movable unit.
 units=[];current=[];fence=None
 for line in lines:
  marker=re.match(r'^\s*(`{3,}|~{3,})',line)
  if marker:
   token=marker[1][0]
   if fence is None:fence=token
   elif fence==token:fence=None
  current.append(line)
  if not line.strip() and fence is None:
   units.append(''.join(current));current=[]
 if current:units.append(''.join(current))
 chunks=[];chunk=[];count=0
 for unit in units:
  length=len(unit.splitlines())
  if chunk and count+length>65:chunks.append(''.join(chunk));chunk=[];count=0
  chunk.append(unit);count+=length
 if chunk:chunks.append(''.join(chunk))
 assert ''.join(chunks)==normative
 clause_pages=[];navigation='Read the requirements in order. These pages preserve the earlier clauses; their review status remains pending.\n\n'
 for number,chunk in enumerate(chunks,1):
  child=Path(page).parent.as_posix()+f'/clauses/{number:02}.md'
  heading=next((line.lstrip('# ').strip() for line in chunk.splitlines() if line.startswith('## ')),f'Requirements {number}')
  body=f'# {r["title"]}: {heading}\n\nFor: maintainers reconciling the detailed behavior of this guarantee.\n\n'
  body+='Return to '+link(child,page,'the contract and its implementation')+'.\n\n'+rebase(chunk,page,child)
  emit(child,body);clause_pages.append(child)
  navigation+='- '+link(page,child,heading)+'\n'
  navigation+=f'<book:include file="clauses/{number:02}.md" />\n'
 p.write_text(text[:start]+navigation.rstrip()+text[end:])
 r['clause_pages']=clause_pages
m['layout_refined']=True
MAP.write_text(json.dumps(m,indent=2)+'\n')
print(f'Reorganized {len(domains)} stakeholder-domain indexes and {len(new_catalogs)} component batches.')
print('Detailed requirement pages:',sum(len(r.get('clause_pages',[])) for r in m['contracts'].values()))
</maint:entry>
</maint:doc>
