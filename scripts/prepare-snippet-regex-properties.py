from pathlib import Path
import re,json,sys
root=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else (_ for _ in ()).throw(SystemExit("Pass the prepared Unicode17 scratch directory"))
rows=lambda file:[[item.strip() for item in row.split('#',1)[0].split(';')] for row in (root/file).read_text().splitlines() if row.split('#',1)[0].strip()]
properties={}
for row in rows('PropertyAliases.txt'):
 for alias in row:properties[alias]=row[1]
oxc=next((Path.home()/'.cargo/registry/src').glob('*/oxc_regular_expression-0.110.0/src/parser/pattern_parser/unicode_property.rs')).read_text()
script_names=set(re.findall(r'"([^"]+)"',re.search(r'static SC_PROPERTY_VALUES:.*?phf_set! \{(.*?)\};',oxc,re.S)[1]))
values={};requests={};aliases={}
for row in rows('PropertyValueAliases.txt'):
 if row[0] not in ['gc','sc']:continue
 kind=row[0];canonical=row[2]
 if kind=='sc' and not any(alias in script_names for alias in row[1:]):continue
 for alias in row[1:]:values[(kind,alias)]=canonical
 kinds=['General_Category'] if kind=='gc' else ['Script','Script_Extensions']
 for full in kinds:
  key=full+'='+canonical;requests[key]=key
  for alias in row[1:]:
   aliases[full+'='+alias]=key;aliases[('gc' if full=='General_Category' else 'sc' if full=='Script' else 'scx')+'='+alias]=key
   if kind=='gc':aliases[alias]=key
oxc=next((Path.home()/'.cargo/registry/src').glob('*/oxc_regular_expression-0.110.0/src/parser/pattern_parser/unicode_property.rs')).read_text()
body=re.search(r'static BINARY_UNICODE_PROPERTIES:.*?phf_set! \{(.*?)\};',oxc,re.S)[1]
for alias in re.findall(r'"([^"]+)"',body):
 canonical=properties.get(alias,alias);requests[canonical]=canonical;aliases[alias]=canonical
(root/'property-requests.json').write_text(json.dumps({'requests':sorted(requests),'aliases':aliases},separators=(',',':'))+'\n')
print('Prepared',len(requests),'canonical Unicode property scans and',len(aliases),'exact aliases')
