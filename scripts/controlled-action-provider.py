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
issued_completions={}
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
    if method=="initialize":result={"capabilities":{"textDocumentSync":1,"documentSymbolProvider":True,"typeHierarchyProvider":True,"completionProvider":{"resolveProvider":language=="python","triggerCharacters":["."]},"codeActionProvider":{"resolveProvider":language=="python"}}}
    elif method=="textDocument/completion":
        uri=params["textDocument"]["uri"];opaque={"producer":language,"uri":uri,"range":{"start":{"line":91,"character":2},"end":{"line":91,"character":4}},"unicode":"é😀"}
        if params["position"]["character"]==2:opaque["slow"]=True
        edit_range={"start":{"line":0,"character":0},"end":{"line":0,"character":5}}
        item={"label":language+" completion","filterText":"value","textEditText":language+"_initial"}
        original={**item,"data":opaque,"insertTextFormat":1,"textEdit":{"range":edit_range,"newText":item["textEditText"]}}
        issued_completions[uri]=original
        result={"isIncomplete":False,"itemDefaults":{"editRange":edit_range,"data":opaque,"insertTextFormat":1},"items":[item]}
        event({"completion_query":language,"context":params.get("context")})
    elif method=="completionItem/resolve":
        uri=params.get("data",{}).get("uri")
        if language!="python" or params!=issued_completions.get(uri):
            event({"error":"altered completion or wrong provider","language":language});send({"jsonrpc":"2.0","id":identity,"error":{"code":-32602,"message":"Original completion payload was not preserved"}});continue
        event({"completion_resolved":language,"slow_completion":params["data"].get("slow",False)})
        result={**params,"textEdit":{**params["textEdit"],"newText":"python_completed"},"additionalTextEdits":[{"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":0}},"newText":"# imported\n"}]}
        if uri.endswith("/snippet.py"):result.update(insertTextFormat=2,textEdit={**params["textEdit"],"newText":"${1:😀 value} + $1$0"})
        if uri.endswith("/nested.py"):result.update(insertTextFormat=2,textEdit={**params["textEdit"],"newText":"${1:value} + $1$0"})
        if params["data"].get("slow"):
            def release_completion(identity=identity,result=result):
                deadline=time.monotonic()+10
                while not os.path.exists(gate) and time.monotonic()<deadline:time.sleep(.01)
                if os.path.exists(gate):send({"jsonrpc":"2.0","id":identity,"result":result});event({"completion_released":language})
                else:send({"jsonrpc":"2.0","id":identity,"error":{"code":-32603,"message":"Controlled completion gate timed out"}})
            threading.Thread(target=release_completion,daemon=True).start();continue
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
        if params.get("context",{}).get("only")==["source.fixAll"]:
            event({"bulk_query":language,"context":params["context"],"range":params["range"]})
            result[0]["kind"]="source.fixAll.controlled"
            if uri.endswith("/unavailable.py"):result=[]
        if result:issued[uri]=result[0]
    elif method=="codeAction/resolve":
        opaque=params.get("data",{});uri=opaque.get("uri");expected=issued.get(uri)
        if params!=expected or opaque.get("producer")!=language:
            event({"error":"altered opaque action or wrong provider","language":language,"received":params});send({"jsonrpc":"2.0","id":identity,"error":{"code":-32602,"message":"Original provider payload was not preserved"}});continue
        event({"resolved":language,"uri":uri,"slow":opaque.get("slow",False)})
        result={**params,"edit":{"changes":{uri:[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":language+"_fixed"}]}}}
        if os.environ.get("HICKORY_CONTROLLED_ACTION_OTHER"):
            other=uri.rsplit("/",1)[0]+"/other.py"
            result["edit"]["changes"][other]=[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"other_fixed"}]
        if os.environ.get("HICKORY_CONTROLLED_ACTION_RESOURCES"):
            base=uri.rsplit("/",1)[0]
            result["edit"]={"documentChanges":[
                {"textDocument":{"uri":uri,"version":91},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"python_fixed"}]},
                {"kind":"create","uri":base+"/created.py"},
                {"textDocument":{"uri":base+"/created.py","version":None},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"created = 1\n"}]},
                {"kind":"rename","oldUri":base+"/other.py","newUri":base+"/moved.py"},
                {"textDocument":{"uri":base+"/moved.py","version":None},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"other_fixed"}]},
                {"kind":"delete","uri":base+"/delete.py"}
            ]}
        if os.environ.get("HICKORY_CONTROLLED_DEBUG_RESOURCE_PHASE"):
            phase=open(os.environ["HICKORY_CONTROLLED_DEBUG_RESOURCE_PHASE"]).read().strip();base=uri.rsplit("/",1)[0]
            rename=lambda old,new:{"kind":"rename","oldUri":base+"/"+old,"newUri":base+"/"+new}
            create=lambda path:[{"kind":"create","uri":base+"/"+path},{"textDocument":{"uri":base+"/"+path,"version":None},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"replacement = 99\n"}]}]
            operations={"move":[rename("a.py","moved.py")],"swap":[rename("moved.py","swap.py"),rename("other.py","moved.py"),rename("swap.py","other.py")],"delete":[{"kind":"delete","uri":base+"/other.py"}],"recreate-a":create("a.py"),"recreate-other":create("other.py")}
            result["edit"]={"documentChanges":operations[phase]}
        if opaque.get("slow"):
            def release(identity=identity,result=result):
                deadline=time.monotonic()+10
                while not os.path.exists(gate) and time.monotonic()<deadline:time.sleep(.01)
                if not os.path.exists(gate):send({"jsonrpc":"2.0","id":identity,"error":{"code":-32603,"message":"Controlled gate timed out"}})
                else:send({"jsonrpc":"2.0","id":identity,"result":result})
            threading.Thread(target=release,daemon=True).start();continue
    else:result=None
    send({"jsonrpc":"2.0","id":identity,"result":result})
