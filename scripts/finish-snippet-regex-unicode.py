from pathlib import Path
import json,hashlib,sys
root=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else (_ for _ in ()).throw(SystemExit("Pass the prepared Unicode17 scratch directory"))
properties=json.loads((root/'properties-17.json').read_text());case=json.loads((root/'case-17-partial.json').read_text());parent={}
def find(cp):
 parent.setdefault(cp,cp)
 if parent[cp]!=cp:parent[cp]=find(parent[cp])
 return parent[cp]
def union(a,b):
 a,b=find(a),find(b)
 if a!=b:parent[max(a,b)]=min(a,b)
for row in (root/'CaseFolding.txt').read_text().splitlines():
 row=row.split('#',1)[0].strip()
 if not row:continue
 cp,status,mapping,*_=map(str.strip,row.split(';'))
 if status in ['C','S']:
  points=mapping.split();assert len(points)==1;union(int(cp,16),int(points[0],16))
groups={}
for cp in parent:groups.setdefault(find(cp),[]).append(cp)
case['space']=properties['properties']['General_Category=Space_Separator'];case['upper_letter']=properties['properties']['General_Category=Uppercase_Letter'];case['lower_letter']=properties['properties']['General_Category=Lowercase_Letter'];
case['fold']=sorted([[cp,sorted(peer for peer in peers if peer!=cp)] for peers in groups.values() for cp in peers]);case['identifier_start']=properties['properties']['ID_Start'];case['identifier_continue']=properties['properties']['ID_Continue']
sets={}
for file in ['emoji-sequences.txt','emoji-zwj-sequences.txt']:
 text=(root/file).read_text();assert '# Version: 17.0' in text
 for row in text.splitlines():
  row=row.split('#',1)[0].strip()
  if not row:continue
  fields=[item.strip() for item in row.split(';')];points,kind=fields[:2];target=sets.setdefault(kind,set())
  if '..' in points:
   first,last=map(lambda value:int(value,16),points.split('..'))
   for cp in range(first,last+1):target.add((cp,))
  else:target.add(tuple(int(cp,16) for cp in points.split()))
sets['RGI_Emoji']=set().union(*sets.values())
def ranges(points):
 result=[]
 for point in sorted(points):
  if result and result[-1][1]+1==point:result[-1][1]=point
  else:result.append([point,point])
 return result
properties['strings']={name:{'ranges':ranges(value[0] for value in values if len(value)==1),'strings':[list(value) for value in sorted((value for value in values if len(value)!=1),key=lambda value:(-len(value),value))]} for name,values in sorted(sets.items())}
properties['sources']={file:{'sha256':hashlib.sha256((root/file).read_bytes()).hexdigest()} for file in ['PropertyAliases.txt','PropertyValueAliases.txt','CaseFolding.txt','emoji-sequences.txt','emoji-zwj-sequences.txt']}
case['sources']={'CaseFolding.txt':properties['sources']['CaseFolding.txt'],'runtime':properties['oracle']}
properties['oracle_discrepancies']={}
for name in ['Script=Katakana_Or_Hiragana','Script_Extensions=Katakana_Or_Hiragana']:
 value=properties['properties'][name];assert 'oracle_error' in value;properties['oracle_discrepancies'][name]={'observed':value,'resolution':'empty Unicode property set','unicode_reference':'https://www.unicode.org/reports/tr31/tr31-45.html','ecmascript_reference':'https://tc39.es/ecma262/multipage/text-processing.html#sec-runtime-semantics-unicodematchpropertyvalue-p-v'};properties['properties'][name]=[]
(root/'unicode-17.json').write_text(json.dumps(properties,ensure_ascii=False,separators=(',',':'))+'\n');(root/'case-17.json').write_text(json.dumps(case,ensure_ascii=False,separators=(',',':'))+'\n')
errors={name:value for name,value in properties['properties'].items() if not isinstance(value,list)}
print('Pinned Unicode17:',len(properties['properties']),'properties;',len(case['fold']),'simple-fold keys;',len(sets['RGI_Emoji']),'emoji strings/codepoints')
print('Explicit oracle discrepancies requiring review:',json.dumps(errors))
