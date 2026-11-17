<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise the native channel with one local wire fixture

For maintainers changing workspace services independently of the editor renderer.

This local test peer implements Hickory's own channel framing, not a third-party language server. Both native clients use the real WebSocket implementation against it. It counts connections, records request IDs, returns out-of-order source-tagged replies, emits URI-scoped notifications and drops a connection on request. Full-text document replay is observable through the HTTP state endpoint. The peer uses only Python's standard library, so native checks do not acquire a browser dependency. Its capture acknowledgement compares the requested text with the actual observed document stream and returns an effectless exact UTF-8 stamp before symbols; this is transport-fixture evidence, not a claim to compile or analyze a language. No workspace files are read or written. Real provider semantics remain covered by the separate browser/engine flows.

<ui:copy id="simple-shared-language-wire-fixture">
"""Local Hickory channel peer for native transport tests; no language semantics."""
import base64
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import socket
import struct
import threading

lock = threading.RLock()
connections = 0
latest = {}
request_ids = []
proposal_requests = []

class Peer(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_):
        pass

    def do_GET(self):
        global connections, latest
        if self.headers.get("Upgrade", "").lower() != "websocket":
            if self.path != "/state":
                self.send_error(404)
                return
            with lock:
                body = json.dumps({"connections": connections, "ids": request_ids, "documents": latest, "proposals": proposal_requests}).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        key = self.headers["Sec-WebSocket-Key"]
        accept = base64.b64encode(hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()).decode()
        self.send_response(101)
        self.send_header("Upgrade", "websocket")
        self.send_header("Connection", "Upgrade")
        self.send_header("Sec-WebSocket-Accept", accept)
        self.end_headers()
        self.documents = {}
        self.prepared = {}
        self.apply_replies = {}
        self.send_lock = threading.Lock()
        self.closed = threading.Event()
        self.close_connection = True
        with lock:
            connections += 1
            latest = self.documents
        try:
            fragments = bytearray()
            while not self.closed.is_set():
                header = self.read_exact(2)
                opcode = header[0] & 15
                length = header[1] & 127
                if length == 126:
                    length = struct.unpack("!H", self.read_exact(2))[0]
                elif length == 127:
                    length = struct.unpack("!Q", self.read_exact(8))[0]
                if length > 1024 * 1024:
                    break
                mask = self.read_exact(4) if header[1] & 128 else None
                payload = self.read_exact(length)
                if mask:
                    payload = bytes(value ^ mask[index % 4] for index, value in enumerate(payload))
                if opcode == 8:
                    self.frame(payload, 8)
                    break
                if opcode == 9:
                    self.frame(payload, 10)
                    continue
                if opcode == 10:
                    continue
                if opcode not in (0, 2):
                    continue
                fragments.extend(payload)
                if not header[0] & 128:
                    continue
                packet = bytes(fragments)
                fragments.clear()
                if packet[:1] == b"\x02":
                    self.message(json.loads(packet[1:]))
        except (OSError, EOFError, ValueError):
            pass
        finally:
            self.closed.set()

    def read_exact(self, count):
        value = self.rfile.read(count)
        if len(value) != count:
            raise EOFError()
        return value

    def frame(self, payload, opcode=2):
        if self.closed.is_set():
            return
        length = len(payload)
        header = bytes([128 | opcode, length]) if length < 126 else bytes([128 | opcode, 126]) + struct.pack("!H", length) if length < 65536 else bytes([128 | opcode, 127]) + struct.pack("!Q", length)
        try:
            with self.send_lock:
                self.wfile.write(header + payload)
                self.wfile.flush()
        except OSError:
            self.closed.set()

    def send(self, value):
        self.frame(b"\x02" + json.dumps(value, separators=(",", ":")).encode())

    def message(self, message):
        method = message.get("method")
        params = message.get("params", {})
        uri = params.get("textDocument", {}).get("uri")
        if method == "textDocument/didOpen":
            with lock:
                self.documents[uri] = params["textDocument"]["text"]
            self.send({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {"uri": uri, "diagnostics": [{"message": "Report for " + uri, "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}}}]}})
            return
        if method == "textDocument/didChange":
            with lock:
                self.documents[uri] = params["contentChanges"][0]["text"]
            return
        if method == "textDocument/didClose":
            with lock:
                self.documents.pop(uri, None)
            return
        identifier = message.get("id")
        if identifier is None:
            return
        if method is None:
            parent = self.apply_replies.pop(identifier, None)
            if parent is not None:
                self.send({"jsonrpc": "2.0", "id": parent, "result": message.get("result")})
            return
        with lock:
            request_ids.append(identifier)
        if method == "fixture/askApply":
            child = "server-apply-" + str(identifier)
            self.apply_replies[child] = identifier
            self.send({"jsonrpc": "2.0", "id": child, "method": "workspace/applyEdit", "params": {"edit": {"changes": {uri: []}}}})
            return
        if method == "fixture/drop":
            self.closed.set()
            self.connection.shutdown(socket.SHUT_RDWR)
            self.connection.close()
            return
        with lock:
            result = {"uri": uri, "text": self.documents.get(uri), "method": method}
        if method == "hick/captureDocument":
            captured = params.get("textDocument", {}).get("text")
            with lock:
                current = isinstance(captured, str) and self.documents.get(uri) == captured
                count = len(self.documents)
            result = {"processed": current, "current": current, "parsed": False, "open_files": count,
                      "source_hash": "sha256:" + hashlib.sha256(captured.encode()).hexdigest() if current else None,
                      "source_revision": hashlib.sha256((uri + "\0" + str(identifier) + "\0" + captured).encode()).hexdigest() if current else None,
                      "source_files_modified": False}
        if method == "hick/captureDocument" and result["current"]:
            self.prepared[uri] = {"source_hash": result["source_hash"], "source_revision": result["source_revision"]}
        if method in ("hick/rename", "hick/formatting"):
            with lock:
                proposal_requests.append({"method": method, "expected": params.get("expected"), "uri": uri})
                valid = params.get("expected") == self.prepared.get(uri)
            result = {"title": "Bound " + method, "edit": {"documentChanges": []}, "data": {"hickory_action": "fixture-only", "hickory_sources": [], "fixture_method": method, "fixture_expected": params.get("expected")}, "source_files_modified": False}
            if not valid:
                result["disabled"] = {"reason": "Missing exact captured preparation"}
        if method == "textDocument/documentSymbol":
            result = [{"name": "background", "kind": 12, "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}, "selectionRange": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}}]
        if method == "textDocument/prepareTypeHierarchy" or method in ("typeHierarchy/supertypes","typeHierarchy/subtypes"):
            target=uri if method == "textDocument/prepareTypeHierarchy" else params["item"]["uri"]
            name="seed" if method == "textDocument/prepareTypeHierarchy" else "supertype" if method.endswith("supertypes") else "subtype"
            result=[{"name":name,"kind":5,"uri":target,"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"selectionRange":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"data":{"fixture":"opaque type"}}]
        timer = threading.Timer(.2 if method in ("hick/rename", "hick/formatting") else .5 if method == "fixture/block" else .07 if method in ("fixture/slow", "textDocument/documentSymbol", "typeHierarchy/supertypes", "typeHierarchy/subtypes") else .001, lambda: self.send({"jsonrpc": "2.0", "id": identifier, "result": result}))
        timer.daemon = True
        timer.start()

server = ThreadingHTTPServer(("127.0.0.1", 0), Peer)
print(json.dumps({"origin": "http://127.0.0.1:" + str(server.server_port)}), flush=True)
server.serve_forever()
</ui:copy>

<ui:file path="scripts/simple-language-wire-fixture.py"><ui:paste select="#simple-shared-language-wire-fixture"/></ui:file>
</ui:doc>
