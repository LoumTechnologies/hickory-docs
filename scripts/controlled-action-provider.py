"""Controlled child LSP for routing/opaque-action tests, not language semantics."""
import json
import os
import sys
import threading
import time
language,events=sys.argv[1:3]
gate=sys.argv[3] if len(sys.argv)>3 else events+".release"
output_lock=threading.Lock()
documents={}
issued={}
def send(value):
    raw=json.dumps(value,separators=(",",":")).encode()
    with output_lock:sys.stdout.buffer.write(f"Content-Length: {len(raw)}\r\n\r\n".encode()+raw);sys.stdout.buffer.flush()
def event(value):
    descriptor=os.open(events,os.O_WRONLY|os.O_CREAT|os.O_APPEND,0o600)
    try:os.write(descriptor,(json.dumps(value)+"\n").encode())
    finally:os.close(descriptor)
while True:
    headers={}
    while True:
        line=sys.stdin.buffer.readline()
        if not line:sys.exit(0)
        if line in (b"\r\n",b"\n"):break
        key,value=line.decode().split(":",1);headers[key.lower()]=value.strip()
    message=json.loads(sys.stdin.buffer.read(int(headers["content-length"])));method=message.get("method");params=message.get("params",{});identity=message.get("id")
    if method=="textDocument/didOpen":documents[params["textDocument"]["uri"]]=params["textDocument"]["text"];continue
    if method=="textDocument/didChange":documents[params["textDocument"]["uri"]]=params["contentChanges"][-1]["text"];continue
    if method=="exit":sys.exit(0)
    if identity is None:continue
    if method=="initialize":result={"capabilities":{"textDocumentSync":1,"documentSymbolProvider":True,"codeActionProvider":{"resolveProvider":language=="python"}}}
    elif method=="textDocument/documentSymbol":result=[]
    elif method=="textDocument/codeAction":
        uri=params["textDocument"]["uri"];opaque={"producer":language,"uri":uri,"nested":[1,{"uri":"file:///opaque-child-value"}]}
        if params["range"]["start"]["character"]==2:opaque["slow"]=True
        result=[{"title":f"Controlled {language} fix","kind":"quickfix","data":opaque}]
        if language!="python":result[0]["edit"]={"changes":{uri:[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":language+"_fixed"}]}}
        issued[uri]=result[0]
    elif method=="codeAction/resolve":
        opaque=params.get("data",{});uri=opaque.get("uri");expected=issued.get(uri)
        if params!=expected or opaque.get("producer")!=language:
            event({"error":"altered opaque action or wrong provider","language":language,"received":params});send({"jsonrpc":"2.0","id":identity,"error":{"code":-32602,"message":"Original provider payload was not preserved"}});continue
        event({"resolved":language,"uri":uri,"slow":opaque.get("slow",False)})
        result={**params,"edit":{"changes":{uri:[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":language+"_fixed"}]}}}
        if opaque.get("slow"):
            def release(identity=identity,result=result):
                deadline=time.monotonic()+10
                while not os.path.exists(gate) and time.monotonic()<deadline:time.sleep(.01)
                if not os.path.exists(gate):send({"jsonrpc":"2.0","id":identity,"error":{"code":-32603,"message":"Controlled gate timed out"}})
                else:send({"jsonrpc":"2.0","id":identity,"result":result})
            threading.Thread(target=release,daemon=True).start();continue
    else:result=None
    send({"jsonrpc":"2.0","id":identity,"result":result})
