<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Capture finite language answers before hosting

The static host cannot ask new semantic questions. During the build, speak ordinary Content-Length JSON-RPC to an explicitly selected local language server. Keep diagnostics while awaiting responses, answer configuration requests without inheriting user settings, and bound each wait. A stalled or truncated stream fails the build instead of publishing an apparently complete capture.

<static:copy id="static-scripts-capture-source-language-py-0">
"""Finite build-time LSP capture; no sockets or server process reach the site."""
import json
from pathlib import Path
import queue
import os
import signal
import subprocess
import tempfile
import threading
import time


class CaptureServer:
    def __init__(self, command, root, timeout=120):
        self.temporary = tempfile.TemporaryDirectory(prefix='.language-analysis-', dir=root)
        environment = dict(os.environ, TMPDIR=self.temporary.name, TEMP=self.temporary.name, TMP=self.temporary.name)
        self.process = subprocess.Popen(command, cwd=root, env=environment, stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, start_new_session=True)
        self.messages = queue.Queue()
        self.timeout = timeout
        self.sequence = 0
        self.diagnostics = {}
        def read():
            try:
                while True:
                    headers = {}
                    while True:
                        line = self.process.stdout.readline()
                        if not line:
                            raise EOFError('Language server closed')
                        if line == b'\r\n':
                            break
                        key, value = line.decode().split(':', 1)
                        headers[key.lower()] = value.strip()
                    size = int(headers['content-length'])
                    if size > 32 * 1024 * 1024:
                        raise ValueError('Language response exceeds publication bound')
                    self.messages.put(json.loads(self.process.stdout.read(size)))
            except Exception as error:
                self.messages.put(error)
        self.reader = threading.Thread(target=read, daemon=True)
        self.reader.start()

    def send(self, method, params, identity=None):
        message = dict(jsonrpc='2.0', method=method, params=params)
        if identity is not None:
            message['id'] = identity
        data = json.dumps(message).encode()
        self.process.stdin.write(f'Content-Length: {len(data)}\r\n\r\n'.encode() + data)
        self.process.stdin.flush()

    def request(self, method, params):
        return self.request_many([(method, params)])[0]

    def request_many(self, requests):
        identities = []
        for method, params in requests:
            self.sequence += 1
            identities.append(self.sequence)
            self.send(method, params, self.sequence)
        answers = {}
        deadline = time.monotonic() + self.timeout
        while len(answers) < len(identities):
            try:
                message = self.messages.get(timeout=max(0, deadline - time.monotonic()))
            except queue.Empty:
                # A timeout may leave the server processing old work. Abort the
                # build rather than label a partial capture as a finished one.
                raise TimeoutError('Static language capture timed out') from None
            if isinstance(message, Exception):
                raise message
            if message.get('method') == 'textDocument/publishDiagnostics':
                value = message['params']
                self.diagnostics[value['uri']] = value['diagnostics']
            elif 'method' in message and 'id' in message:
                reply = dict(jsonrpc='2.0', id=message['id'], result=None)
                if message['method'] == 'workspace/configuration':
                    reply['result'] = [{} for _ in message.get('params', {}).get('items', [])]
                data = json.dumps(reply).encode()
                self.process.stdin.write(f'Content-Length: {len(data)}\r\n\r\n'.encode() + data)
                self.process.stdin.flush()
            elif message.get('id') in identities:
                identity = message['id']
                answers[identity] = (dict(state='unavailable', code=message['error'].get('code')) if 'error' in message
                    else dict(state='captured', result=message.get('result')))
        return [answers[identity] for identity in identities]

    def close(self):
        try:
            if os.name == 'posix':
                os.killpg(self.process.pid, signal.SIGTERM)
            else:
                self.process.terminate()
        except ProcessLookupError:
            pass
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        if os.name == 'posix':
            try:
                os.killpg(self.process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        self.process.stdin.close()
        self.process.stdout.close()
        self.reader.join(timeout=1)
        self.temporary.cleanup()

</static:copy>

The acknowledgement also reports opened provider files. When none were opened, retain Hickory’s own diagnostics and explicitly mark semantic analysis unavailable instead of issuing thousands of empty identifier requests, especially for raw metadata entries.

Each document gets a fresh dispatcher, and shutdown retires the entire process group, including child providers. Hickory stages a distinct project root per document; keeping one dispatcher for thousands of pages would accumulate thousands of language-server processes.

Now open the exact selected Markdown in an isolated temporary root and ask the read-only questions at each code identifier. Preserve captured nulls separately from protocol errors. This snapshot records the provider's answers, including empty results; it does not establish that every embedded language provider was installed or fully bound. Diagnostics are those observed during the finite request sequence, not a claim that asynchronous analysis reached quiescence. We never request rename, formatting, code actions, completion edits or execution. Normalize temporary file URIs before serializing, and always retire the process.

<static:copy id="static-scripts-capture-source-language-py-1">

def capture(documents, plans, command):
    # Only selected committed Markdown enters the analysis workspace. In
    # particular this never mounts the publisher's dirty checkout or secrets.
    with tempfile.TemporaryDirectory(prefix='hickory-static-language-') as temporary:
        root = Path(temporary)
        for document in documents:
            path = root / document['path']
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(document['source'])
        results = {}
        for document in documents:
            server = CaptureServer(command, root)
            try:
                initialized = server.request('initialize', dict(processId=None, rootUri=root.as_uri(), capabilities={}))
                if initialized['state'] != 'captured':
                    raise ValueError('Static language server could not initialize')
                capabilities = initialized['result'].get('capabilities', {})
                server.send('initialized', {})
                path, source = document['path'], document['source']
                uri = (root / path).as_uri()
                prepared = server.request('hick/captureDocument', dict(textDocument=dict(uri=uri, languageId='markdown', version=1, text=source)))
                if prepared['state'] != 'captured' or not (prepared.get('result') or {}).get('processed'):
                    raise ValueError('Static capture requires a Hickory server with document readiness acknowledgement')
                base = dict(textDocument=dict(uri=uri))
                provider_count = prepared['result'].get('open_files', 0)
                result = dict(capabilities=capabilities, queries={}, occurrences=[], provider_count=provider_count)
                if provider_count == 0:
                    result['diagnostics'] = server.diagnostics.get(uri, [])
                    results[path] = result
                    continue
                for method in ['documentSymbol', 'semanticTokens/full', 'foldingRange', 'inlayHint']:
                    params = dict(base)
                    if method == 'inlayHint':
                        lines = source.split('\n')
                        params['range'] = dict(start=dict(line=0, character=0), end=dict(line=len(lines)-1, character=len(lines[-1].encode('utf-16-le'))//2))
                    result['queries'][method] = server.request('textDocument/' + method, params)
                requests, destinations = [], []
                for occurrence in plans[path]['positions']:
                    item = dict(occurrence)
                    result['occurrences'].append(item)
                    for method in ['hover', 'definition', 'references', 'documentHighlight', 'signatureHelp']:
                        params = dict(base, position=occurrence['position'])
                        if method == 'references':
                            params['context'] = dict(includeDeclaration=True)
                        requests.append(('textDocument/' + method, params))
                        destinations.append((item, method))
                # Pipeline a bounded batch instead of paying a full transport
                # round trip for every identifier and every read-only feature.
                for start in range(0, len(requests), 32):
                    answers = server.request_many(requests[start:start+32])
                    for (item, method), answer in zip(destinations[start:start+32], answers):
                        item[method] = answer
                result['diagnostics'] = server.diagnostics.get(uri, [])
                # URI normalization removes ephemeral build paths. Unpublished
                # targets remain unopenable at the viewer's allowlist boundary.
                encoded = json.dumps(result).replace(root.as_uri() + '/', 'hick:///')
                results[path] = json.loads(encoded)
                server.send('textDocument/didClose', dict(textDocument=dict(uri=uri)))
            finally:
                server.close()
        return results
</static:copy>

<static:file path="scripts/capture-source-language.py"><static:paste select="#static-scripts-capture-source-language-py-0"/><static:paste select="#static-scripts-capture-source-language-py-1"/></static:file>
</static:doc>
