#!/usr/bin/env python3
import json,os,sys,threading,time
EVENTS,GATE=sys.argv[1:3];LOCK=threading.Lock();documents={}
def send(value):
 raw=json.dumps(value).encode()
 with LOCK:sys.stdout.buffer.write(f'Content-Length: {len(raw)}\r\n\r\n'.encode()+raw);sys.stdout.buffer.flush()
def record(value):
 with open(EVENTS,'a') as output:output.write(json.dumps(value)+'\n')
def reply(identity,method,params):
 uri=params.get('textDocument',{}).get('uri','');name=params.get('newName','');record({'method':method,'uri':uri,'name':name})
 if name.startswith('held_'):
  record({'held':name})
  while not os.path.exists(GATE):time.sleep(.005)
 target=uri.rsplit('/',1)[0]+'/b.py' if name=='held_target' else uri.rsplit('/',1)[0]+'/hidden.py' if name=='discover' else uri
 edit={'range':{'start':{'line':0,'character':0},'end':{'line':0,'character':5}},'newText':name or 'formatted'}
 if method=='textDocument/rename':result={'changes':{target:[edit]}}
 elif method=='textDocument/formatting':
  if uri.endswith('/z-bad.py'):send({'jsonrpc':'2.0','id':identity,'error':{'code':-32603,'message':'Owned formatter failure'}});return
  result=[edit]
  if uri.endswith('/unmapped.py'):result.append({'range':{'start':{'line':900,'character':0},'end':{'line':900,'character':1}},'newText':'must refuse whole operation'})
 else:result=[]
 send({'jsonrpc':'2.0','id':identity,'result':result})
while True:
 headers={}
 while True:
  line=sys.stdin.buffer.readline()
  if not line:sys.exit(0)
  if line in (b'\n',b'\r\n'):break
  key,value=line.decode().split(':',1);headers[key.lower()]=value.strip()
 request=json.loads(sys.stdin.buffer.read(int(headers['content-length'])));method=request.get('method');params=request.get('params',{});identity=request.get('id')
 if method=='textDocument/didOpen':documents[params['textDocument']['uri']]=params['textDocument']['text'];continue
 if method=='exit':sys.exit(0)
 if identity is None:continue
 if method=='initialize':send({'jsonrpc':'2.0','id':identity,'result':{'capabilities':{'textDocumentSync':1,'renameProvider':True,'documentFormattingProvider':True,'documentSymbolProvider':True}}});continue
 if method=='shutdown':send({'jsonrpc':'2.0','id':identity,'result':None});continue
 threading.Thread(target=reply,args=(identity,method,params),daemon=True).start()
