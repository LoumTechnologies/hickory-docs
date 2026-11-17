#!/usr/bin/env python3
"""Controlled complete-file provider: protocol timing/projection, not language semantics."""
import json,os,sys,threading,time
EVENTS,GATE=sys.argv[1:3]
LOCK=threading.Lock();documents={};issued={};sequence=0
def send(value):
 raw=json.dumps(value,separators=(',',':')).encode()
 with LOCK:sys.stdout.buffer.write(f'Content-Length: {len(raw)}\r\n\r\n'.encode()+raw);sys.stdout.buffer.flush()
def event(value):
 descriptor=os.open(EVENTS,os.O_WRONLY|os.O_CREAT|os.O_APPEND,0o600)
 try:os.write(descriptor,(json.dumps(value)+'\n').encode())
 finally:os.close(descriptor)
def edits(uri):
 text=documents[uri];rows=[]
 for line,content in enumerate(text.split('\n')):
  offset=content.find('value')
  if offset>=0:rows.append({'range':{'start':{'line':line,'character':len(content[:offset].encode('utf-16-le'))//2},'end':{'line':line,'character':len(content[:offset+5].encode('utf-16-le'))//2}},'newText':'fixed'})
 if uri.endswith('/cross.py') and len(rows)>1:return [{'range':{'start':rows[0]['range']['start'],'end':rows[-1]['range']['end']},'newText':'must not erase contributors'}]
 return rows
while True:
 headers={}
 while True:
  line=sys.stdin.buffer.readline()
  if not line:sys.exit(0)
  if line in (b'\n',b'\r\n'):break
  name,value=line.decode().split(':',1);headers[name.lower()]=value.strip()
 message=json.loads(sys.stdin.buffer.read(int(headers['content-length'])));method=message.get('method');params=message.get('params',{});identity=message.get('id')
 if method=='textDocument/didOpen':documents[params['textDocument']['uri']]=params['textDocument']['text'];continue
 if method=='textDocument/didChange':documents[params['textDocument']['uri']]=params['contentChanges'][-1]['text'];continue
 if method=='textDocument/didClose':documents.pop(params['textDocument']['uri'],None);continue
 if method=='exit':sys.exit(0)
 if identity is None:continue
 if method=='initialize':result={'capabilities':{'textDocumentSync':1,'documentSymbolProvider':True,'codeActionProvider':{'resolveProvider':True,'codeActionKinds':['source.fixAll.controlled','quickfix']}}}
 elif method=='textDocument/documentSymbol':result=[]
 elif method=='textDocument/codeAction':
  uri=params['textDocument']['uri'];text=documents[uri];expected={'start':{'line':0,'character':0},'end':{'line':text.count('\n'),'character':len(text.rsplit('\n',1)[-1].encode('utf-16-le'))//2}}
  context=params.get('context',{});event({'query':uri,'range':params['range'],'context':context,'whole':params['range']==expected,'source':text})
  if context.get('only')!=['source.fixAll'] or params['range']!=expected:result=[{'title':'Contextual only','kind':'quickfix'}]
  else:
   result=[]
   for index in range(513 if os.path.exists(GATE+'.overbudget') else 1):
    sequence+=1;action={'title':'Fix complete file'+(f' {index}' if index else ''),'kind':'source.fixAll.controlled','data':{'sequence':sequence,'uri':uri,'slow':context.get('triggerKind')==2,'vendor':{'range':{'line':97,'character':3},'literal':'é😀?#%'}}};issued[sequence]=action;result.append(action)
 elif method=='codeAction/resolve':
  original=issued.get(params.get('data',{}).get('sequence'))
  if params!=original:
   event({'error':'original opaque provider payload was changed'});send({'jsonrpc':'2.0','id':identity,'error':{'code':-32602,'message':'Original action payload required'}});continue
  uri=params['data']['uri'];result={**params,'edit':{'changes':{uri:edits(uri)}}};event({'resolved_started':uri,'slow':params['data']['slow'],'original':True})
  if params['data']['slow']:
   def release(identity=identity,result=result):
    deadline=time.monotonic()+15
    while not os.path.exists(GATE) and time.monotonic()<deadline:time.sleep(.01)
    send({'jsonrpc':'2.0','id':identity,'result':result} if os.path.exists(GATE) else {'jsonrpc':'2.0','id':identity,'error':{'code':-32603,'message':'Owned gate timed out'}})
   threading.Thread(target=release,daemon=True).start();continue
 else:result=None
 send({'jsonrpc':'2.0','id':identity,'result':result})
