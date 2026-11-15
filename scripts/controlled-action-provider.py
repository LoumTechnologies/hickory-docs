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
issued_types={}
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
    if method=="initialize":result={"capabilities":{"textDocumentSync":1,"documentSymbolProvider":True,"typeHierarchyProvider":True,"codeActionProvider":{"resolveProvider":language=="python"}}}
    elif method=="textDocument/prepareTypeHierarchy":
        uri=params["textDocument"]["uri"]
        item={"name":language+"Type","kind":5,"uri":uri,"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"selectionRange":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"tags":[1],"data":{"producer":language,"uri":uri,"nested":["opaque",{"id":17}]}}
        if params["position"]["character"]==2:item["data"]["slow"]=True
        issued_types[(uri,item["name"])]=item;result=[item]
    elif method in ("typeHierarchy/supertypes","typeHierarchy/subtypes"):
        item=params.get("item",{});uri=item.get("uri")
        if item!=issued_types.get((uri,item.get("name"))) or item.get("data",{}).get("producer")!=language:
            event({"error":"altered type item or wrong provider","language":language});send({"jsonrpc":"2.0","id":identity,"error":{"code":-32602,"message":"Original type payload was not preserved"}});continue
        event({"type":method,"language":language,"slow_type":item["data"].get("slow",False)})
        answer={**item,"name":language+("Base" if method.endswith("supertypes") else "Derived"),"data":{"producer":language,"uri":uri,"nested":["returned",{"id":23}]}}
        issued_types[(uri,answer["name"])]=answer;result=[answer]
        if item["data"].get("slow"):
            def release_type(identity=identity,result=result):
                deadline=time.monotonic()+10
                while not os.path.exists(gate) and time.monotonic()<deadline:time.sleep(.01)
                if os.path.exists(gate):send({"jsonrpc":"2.0","id":identity,"result":result})
                else:send({"jsonrpc":"2.0","id":identity,"error":{"code":-32603,"message":"Controlled type gate timed out"}})
            threading.Thread(target=release_type,daemon=True).start();continue
    elif method=="textDocument/documentSymbol":result=[]
    elif method=="textDocument/codeAction":
        uri=params["textDocument"]["uri"];opaque={"producer":language,"hickory_acquire":[{"uri":"file:///untrusted-child-path"}],"uri":uri,"nested":[1,{"uri":"file:///opaque-child-value"}]}
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
        if os.environ.get("HICKORY_CONTROLLED_ACTION_OTHER"):
            other=uri.rsplit("/",1)[0]+"/other.py"
            result["edit"]["changes"][other]=[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"other_fixed"}]
        if opaque.get("slow"):
            def release(identity=identity,result=result):
                deadline=time.monotonic()+10
                while not os.path.exists(gate) and time.monotonic()<deadline:time.sleep(.01)
                if not os.path.exists(gate):send({"jsonrpc":"2.0","id":identity,"error":{"code":-32603,"message":"Controlled gate timed out"}})
                else:send({"jsonrpc":"2.0","id":identity,"result":result})
            threading.Thread(target=release,daemon=True).start();continue
    else:result=None
    send({"jsonrpc":"2.0","id":identity,"result":result})
