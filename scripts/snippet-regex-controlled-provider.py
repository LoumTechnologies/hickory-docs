"""External lazy child fixture; never supplies a compiled program or writes source."""
import json,sys
EVENTS=sys.argv[1]
from pathlib import Path
CASES=json.loads((Path(__file__).resolve().parents[1]/'apps/simple-contract/snippet-regex-live-cases.json').read_text())
SYNTAX=CASES['syntax'];BUDGET=CASES['budget_syntax']
def record(value):
 with open(EVENTS,'a') as stream:stream.write(json.dumps(value)+'\n')
def send(value):
 raw=json.dumps(value).encode();sys.stdout.buffer.write(b'Content-Length: '+str(len(raw)).encode()+b'\r\n\r\n'+raw);sys.stdout.buffer.flush()
issued={};snippets=False
while True:
 headers={}
 while True:
  line=sys.stdin.buffer.readline()
  if not line:sys.exit()
  if line in (b'\r\n',b'\n'):break
  name,value=line.decode().split(':',1);headers[name.lower()]=value.strip()
 message=json.loads(sys.stdin.buffer.read(int(headers['content-length'])));identity=message.get('id');method=message.get('method');params=message.get('params',{})
 if identity is None:continue
 if method=='initialize':
  snippets=params.get('capabilities',{}).get('textDocument',{}).get('completion',{}).get('completionItem',{}).get('snippetSupport') is True;record({'initialize_snippets':snippets});result={'capabilities':{'textDocumentSync':1,'documentSymbolProvider':True,'completionProvider':{'resolveProvider':True,'triggerCharacters':['.']}}}
 elif method=='textDocument/documentSymbol':result=[]
 elif method=='textDocument/completion':
  uri=params['textDocument']['uri'];record({'query':uri});item={'label':'Regex mirrors','filterText':'OldOld','data':{'opaque':{'uri':uri,'unicode':'é😀','range':{'start':{'line':91,'character':2},'end':{'line':91,'character':4}}}}}
  if uri.endswith('/inline.py'):item.update(insertTextFormat=2,insertText=SYNTAX)
  issued[uri]=item;result=[item] if snippets else []
 elif method=='completionItem/resolve':
  uri=params.get('data',{}).get('opaque',{}).get('uri');assert params==issued.get(uri),'Original opaque provider item was changed';record({'resolve':uri});result={**params,'insertTextFormat':2,'textEdit':{'range':{'start':{'line':0,'character':0},'end':{'line':0,'character':6}},'newText':BUDGET if uri.endswith('/budget.py') else SYNTAX}}
 elif method=='workspace/executeCommand':raise AssertionError('Snippet evaluation must never execute provider commands')
 elif method=='shutdown':result=None
 else:result=None
 send({'jsonrpc':'2.0','id':identity,'result':result})
