import importlib.util
import json
import os
from pathlib import Path
import sys
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('capture_language', Path(__file__).with_name('capture-source-language.py'))
capture_language = importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture_language)

# A real stdio peer checks framing, handles notifications and advertises finite
# answers. It deliberately reports one unsupported method rather than crashing.
PEER = r'''
import sys,json
while True:
    headers={}
    while True:
        line=sys.stdin.buffer.readline()
        if not line:sys.exit(0)
        if line==b'\r\n':break
        key,value=line.decode().split(':',1);headers[key.lower()]=value.strip()
    request=json.loads(sys.stdin.buffer.read(int(headers['content-length'])))
    if 'id' not in request:continue
    method=request['method']
    result=None
    if method=='initialize':result={'capabilities':{}}
    if method=='hick/captureDocument':result={'processed':True,'parsed':True,'open_files':1}
    if method=='textDocument/hover':result={'contents':'number'}
    response={'jsonrpc':'2.0','id':request['id'],'result':result}
    if method=='textDocument/signatureHelp':response={'jsonrpc':'2.0','id':request['id'],'error':{'code':-32601,'message':'not supported'}}
    encoded=json.dumps(response).encode()
    sys.stdout.buffer.write(('Content-Length: %d\r\n\r\n'%len(encoded)).encode()+encoded);sys.stdout.buffer.flush()
'''


class LanguageCaptureTests(unittest.TestCase):
    @unittest.skipUnless(os.environ.get('HICKORY_CAPTURE_LSP'), 'requires an explicitly selected real Hickory LSP and TypeScript provider')
    def test_real_provider_captures_aliased_source_after_readiness(self):
        source = '<book:doc xmlns:book="http://www.hickorydocs.com/1.0">\n<book:file path="a.ts">\nconst label = "😀"; export const count = 1; count;\n</book:file>\n</book:doc>\n'
        start = source.rindex('count;')
        offset = len(source[:start].encode('utf-16-le')) // 2
        character = len(source[source.rfind('\n', 0, start)+1:start].encode('utf-16-le')) // 2
        plans = {'project.md': {'positions': [{'from': offset, 'to': offset+5, 'position': dict(line=2, character=character)}]}}
        result = capture_language.capture([dict(path='project.md', source=source)], plans,
            [os.environ['HICKORY_CAPTURE_LSP']])['project.md']
        occurrence = result['occurrences'][0]
        self.assertEqual(occurrence['from'], offset)
        self.assertIn('count', json.dumps(occurrence['hover']['result']))
        self.assertEqual(occurrence['definition']['result'][0]['uri'], 'hick:///project.md')
        self.assertGreaterEqual(len(occurrence['references']['result']), 2)

    def test_real_stdio_capture_keeps_null_and_unavailable_distinct(self):
        with tempfile.TemporaryDirectory() as temporary:
            peer = Path(temporary) / 'peer.py'
            peer.write_text(PEER)
            documents = [dict(path='project.md', source='😀 count')]
            plans = {'project.md': {'positions': [{'from': 3, 'to': 8, 'position': dict(line=0, character=3)}]}}
            result = capture_language.capture(documents, plans, [sys.executable, str(peer)])['project.md']
            self.assertEqual(result['queries']['documentSymbol'], dict(state='captured', result=None))
            occurrence = result['occurrences'][0]
            self.assertEqual(occurrence['hover']['result']['contents'], 'number')
            self.assertEqual(occurrence['signatureHelp'], dict(state='unavailable', code=-32601))
            self.assertNotIn(temporary, json.dumps(result))

    def test_no_child_provider_is_recorded_without_empty_identifier_answers(self):
        with tempfile.TemporaryDirectory() as temporary:
            peer = Path(temporary) / 'peer.py'
            peer.write_text(PEER.replace("'open_files':1", "'open_files':0"))
            plans = {'project.md': {'positions': [{'from': 0, 'to': 5, 'position': dict(line=0, character=0)}]}}
            result = capture_language.capture([dict(path='project.md', source='count')], plans,
                [sys.executable, str(peer)])['project.md']
            self.assertEqual(result['provider_count'], 0)
            self.assertEqual(result['occurrences'], [])
            self.assertEqual(result['queries'], {})

    def test_pipelined_answers_follow_request_ids_instead_of_arrival_order(self):
        # A dedicated peer is clearer than depending on scheduling: collect two
        # requests, emit the second first, then emit the first before reading.
        prefix = PEER[:PEER.index('while True:')]
        reader = PEER[PEER.index('    headers={}'):PEER.index("    if 'id' not in request")]
        body = prefix + 'requests=[]\nfor _ in range(2):\n' + reader + '    requests.append(request)\nfor request in reversed(requests):\n    response={"jsonrpc":"2.0","id":request["id"],"result":request["method"]}\n    encoded=json.dumps(response).encode()\n    sys.stdout.buffer.write(("Content-Length: %d\\r\\n\\r\\n"%len(encoded)).encode()+encoded);sys.stdout.buffer.flush()\n'
        with tempfile.TemporaryDirectory() as temporary:
            peer = Path(temporary) / 'reverse.py'
            peer.write_text(body)
            server = capture_language.CaptureServer([sys.executable, str(peer)], temporary, timeout=1)
            try:
                self.assertEqual(server.request_many([('first', {}), ('second', {})]),
                    [dict(state='captured', result='first'), dict(state='captured', result='second')])
            finally:
                server.close()

    def test_closed_server_fails_instead_of_publishing_partial_answers(self):
        with tempfile.TemporaryDirectory() as temporary:
            server = capture_language.CaptureServer([sys.executable, '-c', 'pass'], temporary, timeout=1)
            try:
                with self.assertRaises((EOFError, BrokenPipeError)):
                    server.request('initialize', {})
            finally:
                server.close()


class CodeModelCaptureTests(unittest.TestCase):
    def test_real_typescript_model_preserves_optional_properties_and_ignores_dirty_source(self):
        spec = importlib.util.spec_from_file_location('source_models', Path(__file__).with_name('capture-source-models.py'))
        models = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(models)
        command = Path(__file__).resolve().parent.parent / 'code-models/typescript/hick-model-typescript'
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for args in [('init', '-q'), ('config', 'user.email', 'test@example.invalid'), ('config', 'user.name', 'Test')]:
                subprocess.check_call(['git', *args], cwd=root)
            (root / 'types.ts').write_text('export interface Widget { count?: number }')
            subprocess.check_call(['git', 'add', '.'], cwd=root)
            subprocess.check_call(['git', 'commit', '-qm', 'Model fixture'], cwd=root)
            revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
            (root / 'types.ts').write_text('export interface DIRTY {}')
            result = models.capture_models(root, revision, {'types.ts': {}}, {'typescript': [str(command)]})[0]
            declarations = result['graph']['data']['types']
            self.assertEqual(declarations[0]['name'], 'Widget')
            self.assertEqual(declarations[0]['properties'][0]['name'], 'count')
            self.assertTrue(declarations[0]['properties'][0]['isOptional'])
            self.assertIn('unresolved', result['graph']['data'])
            self.assertNotIn('DIRTY', json.dumps(result))
            self.assertNotIn(temporary, json.dumps(result))
            self.assertNotIn('references', result['query'])


if __name__ == '__main__':
    unittest.main()
