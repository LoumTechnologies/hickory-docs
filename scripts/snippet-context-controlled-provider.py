"""Controlled lazy child; provider data remains opaque to the engine."""
import json,sys

events=sys.argv[1]
def record(value):
 with open(events,'a') as output:output.write(json.dumps(value)+'\n')
def send(value):
 raw=json.dumps(value).encode();sys.stdout.buffer.write(b'Content-Length: '+str(len(raw)).encode()+b'\r\n\r\n'+raw);sys.stdout.buffer.flush()
issued={}
while True:
 headers={}
 while True:
  line=sys.stdin.buffer.readline()
  if not line:sys.exit()
  if line in (b'\r\n',b'\n'):break
  key,value=line.decode().split(':',1);headers[key.lower()]=value.strip()
 message=json.loads(sys.stdin.buffer.read(int(headers['content-length'])));identity=message.get('id');method=message.get('method');params=message.get('params',{})
 if identity is None:continue
 if method=='initialize':result={'capabilities':{'textDocumentSync':1,'documentSymbolProvider':True,'completionProvider':{'resolveProvider':True,'triggerCharacters':['.']}}}
 elif method=='textDocument/documentSymbol':result=[]
 elif method=='textDocument/completion':
  uri=params['textDocument']['uri'];item={'label':'context completion','filterText':'old','data':{'opaque':{'uri':uri,'unicode':'é😀','range':{'start':{'line':91,'character':2},'end':{'line':91,'character':4}}}}};issued[uri]=item;record({'query':uri});result=[item]
 elif method=='completionItem/resolve':
  uri=params.get('data',{}).get('opaque',{}).get('uri');assert params==issued.get(uri),'Original opaque provider item was changed';record({'resolve':uri})
  syntax='${TM_SELECTED_TEXT:${CLIPBOARD}}$0' if uri.endswith('/unused.py') else '${CLIPBOARD:fallback}$0' if uri.endswith('/denied.py') else '${TM_FILENAME}|${CURRENT_YEAR}|${RANDOM_HEX}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}|${CLIPBOARD:fallback}$0'
  if uri.endswith('/browser.py'):syntax=syntax.rsplit('|${CLIPBOARD:fallback}',1)[0]+'$0'
  result={**params,'insertTextFormat':2,'textEdit':{'range':{'start':{'line':0,'character':0},'end':{'line':0,'character':3}},'newText':syntax}}
 elif method=='shutdown':result=None
 else:result=None
 send({'jsonrpc':'2.0','id':identity,'result':result})
