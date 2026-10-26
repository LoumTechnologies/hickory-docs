<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/tools/reconcile.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-441e9d3e9134"></a>
## `hickory-dogfood/tools/reconcile.py`

Construct and verify the maintained literate project, preserving migration evidence separately from current verification.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="hickory-dogfood/tools/reconcile.py">
#!/usr/bin/env python3
"""Apply an editorial reset without changing any generated program bytes.

Authored decisions are explicit. Unchanged requirements are retained, not newly
verified. Every original byte is addressable in the paragraph disposition ledger.
"""
import collections
import gzip
import hashlib
import json
import os
from pathlib import Path
import posixpath
import re

BASE=Path(__file__).resolve().parents[1]
ROOT=BASE/'integrated'
FROZEN=BASE/'workspace'
MAP=BASE/'behavior-map.json'
m=json.loads(MAP.read_text())
if m.get('reconciliation_version'):raise SystemExit('Reconciliation already applied; maintain current source directly.')
decisions=json.loads((BASE/'reconciliation/decisions.json').read_text())
entries={e['path']:e for e in json.loads((BASE/'baseline.json').read_text())['entries']}
OPEN='<book:doc xmlns:book="http://www.hickorydocs.com/1.0">\n'
CLOSE='\n</book:doc>\n'

# Retain the previous agent-authored drafts before correcting their authority model.
prior=dict(map=m,pages={})
for r in m['contracts'].values():
 for page in [r['page'],*r.get('clause_pages',[])]:
  prior['pages'][page]=(ROOT/page).read_text()
with gzip.open(BASE/'reconciliation/prior-drafts.json.gz','wt') as output:json.dump(prior,output)


def emit(page,body):
 p=ROOT/page;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(OPEN+body.rstrip()+'\n'+CLOSE)


def link(page,target,label):
 return f'[{label}]({os.path.relpath(target,Path(page).parent).replace(os.sep,"/")})'


def relocate(text,original,destination):
 def change(match):
  url=match[2]
  if url.startswith(('#','http:','https:','mailto:','/')):return match[0]
  path,sep,anchor=url.partition('#')
  candidate=posixpath.normpath(posixpath.join(posixpath.dirname(original),path))
  key=candidate.removeprefix('docs/guarantees/')
  target=m['contracts'][key]['page'] if candidate.startswith('docs/guarantees/') and key in m['contracts'] else m['constructors'].get(candidate,{}).get('source')
  if target:return link(destination,target,match[1])[:-1]+sep+anchor+')'
  # Existing Markdown examples are not links to project artifacts.
  return match[0]
 return re.sub(r'\[([^\]\n]+)\]\(([^)\s]+)\)',change,text).replace('<!--','&lt;!--')


def normative(text):
 section=text.split('\n---\n',1)[0]
 section=re.split(r'(?im)^Last LLM (?:implementation review|verification):',section,maxsplit=1)[0]
 section=re.split(r'(?im)^## (?:Historical checkpoint|Verification record|Assurance boundary)',section,maxsplit=1)[0]
 section=re.sub(r'(?m)^# [^\n]*\n','',section).strip()
 return section


def chunks(text):
 units=[];current=[];fence=None
 for line in text.splitlines(keepends=True):
  marker=re.match(r'^\s*(`{3,}|~{3,})',line)
  if marker:
   token=marker[1][0]
   if fence is None:fence=token
   elif fence==token:fence=None
  current.append(line)
  if not line.strip() and fence is None:units.append(''.join(current));current=[]
 if current:units.append(''.join(current))
 result=[];current=[];count=0
 for unit in units:
  size=len(unit.splitlines())
  if current and count+size>60:result.append(''.join(current));current=[];count=0
  current.append(unit);count+=size
 if current:result.append(''.join(current))
 assert ''.join(result)==text
 return result

# Resolve unique abbreviated citations. These remain navigation candidates.
by_name=collections.defaultdict(list)
for path in entries:
 if m['constructors'][path]['role'] in ('implementation','test','fixture'):by_name[Path(path).name].append(path)
for key,r in m['contracts'].items():
 text=(FROZEN/r['origin']).read_text()
 r['reference_resolution']=[]
 for token in sorted(set(re.findall(r'(?<![\w./-])(?:[\w.-]+/)*[\w.-]+\.(?:rs|tsx?|py|sh|mjs|cs|go)(?![\w.-])',text))):
  candidates=[p for p in by_name[Path(token).name] if p==token or p.endswith('/'+token)]
  if len(candidates)==1 and candidates[0] not in r['references']:
   r['references'].append(candidates[0]);r['reference_resolution'].append(dict(citation=token,path=candidates[0],basis='unique captured suffix; not semantic coverage'))
 d=decisions.get(key,{})
 for path in d.get('implementation',[])+[t['file'] for t in d.get('tests',[])]:
  if path not in entries:raise ValueError('Authored reference absent: '+path)
  if path not in r['references']:r['references'].append(path)
 r['references']=sorted(set(r['references']))

# Archive only explicit retired-page directives or the independently absent hosted endpoint.
for key,r in m['contracts'].items():
 original=(FROZEN/r['origin']).read_text()
 if 'not a guarantee of the current page' in original[:1200]:
  r['disposition']='historical';r['reason']='The original document explicitly retires this homepage design. Its current successor is landing/homepage-opens-paused.md. Retaining old implementation bytes does not reinstate that page contract.'
 if key=='analytics/landing-beacon-is-narrow.md':
  r['disposition']='historical';r['reason']='The promised apps/server analytics route is absent from the captured program. The browser sink is not evidence that this removed server allowlist still runs. Retain the old endpoint contract as history.'

# Rebuild reference backlinks without relocating or duplicating constructors.
users=collections.defaultdict(set)
for key,r in m['contracts'].items():
 if r['disposition']=='historical':continue
 for path in r['references']:
  if m['constructors'][path]['role'] in ('implementation','test','fixture'):users[path].add(key)
for path,c in m['constructors'].items():
 c['used_by']=sorted(users[path])
 p=ROOT/c['source'];text=p.read_text();start=text.index('<book:entry ')
 payload=text[start:]
 intro=f'# {path}\n\nFor: maintainers changing this file and reviewers tracing the related behavioral requirements.\n\n'
 if c['role']=='history-and-guidance':intro+='This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.\n\n'
 else:intro+='Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.\n\n'
 for key in c['used_by']:intro+='- '+link(c['source'],m['contracts'][key]['page'],m['contracts'][key]['title'])+'\n'
 intro+=f'\nCaptured SHA-256: `{c["sha256"]}`.\n\n'
 # The payload, closer and outer wrapper are retained byte for byte.
 p.write_text(OPEN+intro+payload)

ledger=[]
for key,r in m['contracts'].items():
 original=(FROZEN/r['origin']).read_text();raw=original.encode();page=r['page'];directory=Path(page).parent.as_posix()
 old=prior['pages'][page]
 owned=[]
 for path in r['owned']:
  local=os.path.relpath(m['constructors'][path]['source'],directory).replace(os.sep,'/')
  owned.append((path,local))
 for previous in r.get('clause_pages',[]):(ROOT/previous).unlink()
 r['clause_pages']=[]
 d=decisions.get(key)
 if d:
  r['disposition']='reconciled-requirement';r['title']=d['title'];r['decision']=d['decision']
  r['tests']=d['tests'];r['review']='not verified'
 elif r['disposition']=='migrated-for-review':
  r['disposition']='retained-requirement';r['decision']='Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.'
 r['requirement_status']=r['disposition']
 r['implementation_status']=r['review']
 r['human_judgment']='Stakeholder priorities confirmed; individual behavioral requirements not separately approved.'
 r['review_debt']=[f for f in r['flags'] if f in ('no-existing-code-or-test-reference-resolved','some-source-citations-do-not-resolve')]
 if d:r['flags']=[f for f in r['flags'] if f!='amendments-need-reconciliation']
 r['flags']=[f for f in r['flags'] if f!='contract-needs-splitting']

 # Keep an exact byte partition, including context and historical evidence.
 separator=raw.find(b'\n---\n');boundary=len(raw) if separator<0 else separator
 paragraph_ends=[match.end() for match in re.finditer(rb'\n[ \t]*\n',raw)]+[len(raw)]
 cursor=0
 for end in paragraph_ends:
  if end<=cursor:continue
  piece=raw[cursor:end];piece_text=piece.decode();digest=hashlib.sha256(piece).hexdigest()
  if r['disposition']=='historical':action='historical-contract'
  elif cursor<boundary:action='replaced-by-reconciled-summary-with-original-retained' if d or r['disposition']=='rewritten' else 'retained-requirement-or-context'
  else:action='retained-follow-up-for-review'
  ledger.append(dict(guarantee=key,origin=r['origin'],start_byte=cursor,end_byte=end,sha256=digest,action=action,
   destination=page,original_source=m['constructors'][r['origin']]['source'],
   reason=r.get('decision') or r.get('reason') or 'Existing requirement retained.',
   amendment_candidate=cursor>=boundary and bool(re.search(r'(?i)amend|supersed|must |shall |then |now ',piece_text))))
  cursor=end
 assert cursor==len(raw)

 audience=re.search(r'^For: (.+)$',old,re.M)
 body=f'# {r["title"]}\n\nFor: {audience[1] if audience else "Hickory authors, reviewers and maintainers tracing this behavior."}\n\n'
 if r['disposition']=='historical':
  body+=r.get('reason','The implementation belongs to a removed hosted product.')+'\n\nThis is historical product intent. Preserved source files below remain part of byte-exact construction and may serve other current behaviors.\n\n'
 else:
  current=normative(original)
  if d:
   body+=d['scenario']+'\n\n## What must happen\n\n'+d['contract']+'\n\n## Limits of the evidence\n\n'+d['limits']+'\n\n'
  elif r['disposition']=='rewritten':
   # Preserve the previously authored summary and its limits; remove old apparatus.
   start=old.index('## Behavior to preserve\n\n') if '## Behavior to preserve\n\n' in old else -1
   if start>=0:
    ending=old.find('\n## Follow the implementation',start)
    if ending<0:ending=old.find('\n## Construction owned here',start)
    body+=old[start:ending].replace('## Behavior to preserve','## What must happen')+'\n\n'
   else:body+=current.split('\n\n')[0]+'\n\n'
  else:
   first=current.split('\n\n')[0]
   body+=relocate(first,r['origin'],page)+'\n\nThe existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.\n\n'
  # Preserve complete inherited primary requirements as small detail pages.
  if not d and r['disposition']!='rewritten':
   body+='## Read the detailed requirements\n\n'
   for number,chunk in enumerate(chunks(current),1):
    child=directory+f'/requirements/{number:02}.md';r['clause_pages'].append(child)
    detail=f'# {r["title"]}: requirements {number}\n\nFor: maintainers checking the full inherited requirement before changing this behavior.\n\n'
    detail+='Return to '+link(child,page,'the current overview')+'.\n\n'+relocate(chunk,r['origin'],child)
    emit(child,detail)
    body+='- '+link(page,child,f'Requirements {number}')+'\n'
    body+=f'<book:include file="requirements/{number:02}.md" />\n'
   body+='\n'
  body+='## Inspect implementation and tests\n\n'+link(page,directory+'/review.md','Follow the actual construction, named witnesses and evidence gaps')+'.\n\n'
  body+='<book:include file="review.md" />\n\n'
 body+='Earlier wording and all later notes remain in '+link(page,m['constructors'][r['origin']]['source'],'the preserved original')+'. The paragraph ledger records their exact ranges and dispositions.\n\n'
 body+='Requirement decision: '+(r.get('decision') or r.get('reason') or 'Historical contract retained.')+'\n\n'
 body+='Implementation review: '+r['review']+'. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.\n\n'
 if owned:
  body+='## Source constructed here\n\n'+link(page,directory+'/construction.md','Open this behavior’s construction inputs')+'.\n\n<book:include file="construction.md" />\n'
  construction=f'# {r["title"]}: construction\n\nFor: maintainers changing files associated with this behavior.\n\nEach entry is constructed once. A historical requirement can retain files needed by the captured program without making that requirement current.\n\n'
  for path,local in owned:construction+='- '+link(directory+'/construction.md',m['constructors'][path]['source'],path)+f'\n<book:include file="{local}" />\n'
  emit(directory+'/construction.md',construction)
 emit(page,body)

 if r['disposition']!='historical':
  review=f'# {r["title"]}: implementation and witnesses\n\nFor: reviewers checking the code and assertions behind this requirement.\n\nInspect the implementation, then the assertions in the actual test source. A resolved citation or nearby function is not a claim of semantic coverage.\n\n'
  for label,roles in [('Implementation',('implementation',)),('Tests and fixtures',('test','fixture'))]:
   refs=[p for p in r['references'] if m['constructors'][p]['role'] in roles]
   if refs:
    review+=f'## {label}\n\n'
    for path in refs:review+='- '+link(directory+'/review.md',m['constructors'][path]['source'],path)+'\n'
    review+='\n'
  if r['tests']:
   review+='## Focused witnesses\n\n'
   for test in r['tests']:review+=f'- `{test["name"]}` in `{test["file"]}`.\n'
  review+='\n## Evidence status\n\n- LLM implementation review: '+r['review']+'. Editorial retention is not new code verification.\n'
  review+='- Executed checks: '+link(directory+'/review.md','.hickory-source/evidence/focused-checks.md','the run record')+' lists only checks actually executed.\n'
  review+='- Human judgment: stakeholder priorities confirmed; individual contract acceptance is unrecorded.\n'
  review+='- Dependencies: exact cited-file capture hashes are in the construction pages and behavior map; transitive source and environment changes can also stale evidence.\n'
  if r['review_debt']:review+='- Open evidence gaps: '+'; '.join(r['review_debt'])+'.\n'
  if r['reference_resolution']:review+='- Some earlier shorthand citations now resolve uniquely. Those resolutions are navigation candidates, not verified witnesses.\n'
  emit(directory+'/review.md',review)

m['unmapped_implementation']=[p for p,c in m['constructors'].items() if c['role']=='implementation' and not c['used_by']]
m['reconciliation_version']=1
m['reconciliation_policy']='Retain existing requirements; apply explicit supersession decisions; keep implementation review, execution and human acceptance separate.'
MAP.write_text(json.dumps(m,indent=2)+'\n')
(BASE/'reconciliation/paragraph-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n')
# Domain indexes reflect current dispositions, with the same declared inputs.
for page in m['domain_pages']:
 p=ROOT/page;text=p.read_text()
 for r in m['contracts'].values():
  if r['domain']!=Path(page).parent.name:continue
  target=os.path.relpath(r['page'],Path(page).parent).replace(os.sep,'/')
  text=re.sub(r'^- \[[^\n]+\]\('+re.escape(target)+r'\) — [^\n]+$', '- '+link(page,r['page'],r['title'])+' — '+r['disposition']+'.',text,flags=re.M)
 p.write_text(text)
print('Requirements:',dict(collections.Counter(r['disposition'] for r in m['contracts'].values())))
print('Original paragraph ranges accounted for:',len(ledger))
print('Additional unique shorthand citations:',sum(len(r['reference_resolution']) for r in m['contracts'].values()))
print('Implementation files without a direct reference:',len(m['unmapped_implementation']))
</storycode:entry>

</storycode:doc>
