import hashlib
import gzip
import importlib.util
import json
import os
import shutil
from pathlib import Path
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('source_export', Path(__file__).with_name('export-source-site.py'))
exporter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(exporter)


class SourcePublicationTests(unittest.TestCase):
    def test_only_committed_maintained_markdown_enters_the_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            def git(*args):
                return subprocess.check_output(['git', *args], cwd=root, text=True).strip()
            git('init', '-q')
            git('config', 'user.email', 'source-test@example.invalid')
            git('config', 'user.name', 'Source publication test')
            source = '# Build Hickory\n\nλ and exact final bytes'
            (root / 'project.md').write_text(source)
            (root / '.hickory-source').mkdir()
            (root / '.hickory-source/chapter.md').write_text('# Chapter\n')
            (root / 'sessions').mkdir()
            (root / 'sessions/private.md').write_text('PRIVATE SESSION')
            (root / '.env').write_text('PRIVATE CREDENTIAL')
            git('add', '.')
            git('commit', '-qm', 'Public source fixture')
            revision = git('rev-parse', 'HEAD')
            (root / 'project.md').write_text('UNCOMMITTED EDIT')
            (root / '.hickory-source/untracked.md').write_text('UNTRACKED NOTE')
            output = root / 'public'
            manifest = exporter.export(root, output)
            self.assertEqual(manifest['revision'], revision)
            self.assertEqual({d['path'] for d in manifest['documents']}, {'project.md', '.hickory-source/chapter.md'})
            document = next(d for d in manifest['documents'] if d['path'] == 'project.md')
            payload = json.loads((output / document['url']).read_text())
            self.assertEqual(payload, dict(path='project.md', revision=revision, source=source))
            self.assertEqual(document['sha256'], hashlib.sha256(source.encode()).hexdigest())
            published = ''.join(p.read_text() for p in output.rglob('*.json'))
            for private in ['PRIVATE SESSION', 'PRIVATE CREDENTIAL', 'UNTRACKED NOTE', 'UNCOMMITTED EDIT']:
                self.assertNotIn(private, published)

    def test_compiled_highlights_follow_committed_cross_page_fragment_ownership(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for args in [('init', '-q'), ('config', 'user.email', 'test@example.invalid'), ('config', 'user.name', 'Test')]:
                subprocess.check_call(['git', *args], cwd=root)
            source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n😀\n<h:copy id="typed">\nconst count = 1;\n</h:copy>\n</h:doc>'
            # Exceed the OS pipe buffer: a synchronous read of nonblocking
            # subprocess stdin can fail with EAGAIN on a real publication.
            source = source.replace('<h:copy', ('Public prose before the code.\n' * 12000) + '<h:copy', 1)
            (root / 'project.md').write_text(source)
            (root / '.hickory-source').mkdir()
            (root / '.hickory-source/chapter.md').write_text('# Chapter')
            (root / 'hickory-dogfood').mkdir()
            (root / 'hickory-dogfood/behavior-map.json').write_text(json.dumps({'constructors': {'src/a.ts': {'fragments': ['typed']}}}))
            subprocess.check_call(['git', 'add', '.'], cwd=root)
            subprocess.check_call(['git', 'commit', '-qm', 'Language fixture'], cwd=root)
            (root / 'project.md').write_text('DIRTY SOURCE')
            output = root / 'out'
            manifest = exporter.export(root, output, compile_language=True)
            document = next(doc for doc in manifest['documents'] if doc['path'] == 'project.md')
            payload = json.loads((output / document['url']).read_text())
            analysis = json.loads(payload['intelligence'])
            self.assertEqual(payload['source'], source)
            self.assertEqual(analysis['state'], 'unavailable')
            self.assertIsNone(analysis['language'])
            self.assertEqual(analysis['spans'], [])
            packed = analysis['packed_spans']
            triples = zip(packed['data'][::3], packed['data'][1::3], packed['data'][2::3])
            start, length, _ = next(span for span in triples if packed['classes'][span[2]] == 'tok-keyword')
            self.assertEqual(source.encode('utf-16-le')[start*2:(start+length)*2].decode('utf-16-le'), 'const')
            encoded = json.dumps(analysis, ensure_ascii=False, separators=(',', ':')).encode()
            self.assertEqual(hashlib.sha256(encoded).hexdigest(), document['intelligence_sha256'])

    def test_plain_fences_keep_colours_without_a_workspace_language_scan(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for args in [('init', '-q'), ('config', 'user.email', 'test@example.invalid'), ('config', 'user.name', 'Test')]:
                subprocess.check_call(['git', *args], cwd=root)
            (root / 'project.md').write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n```ts\nconst count = 1;\n```\n</h:doc>')
            (root / '.hickory-source').mkdir()
            (root / '.hickory-source/chapter.md').write_text('# Chapter')
            (root / 'hickory-dogfood').mkdir()
            (root / 'hickory-dogfood/behavior-map.json').write_text('{"constructors":{}}')
            subprocess.check_call(['git', 'add', '.'], cwd=root)
            subprocess.check_call(['git', 'commit', '-qm', 'Prose fixture'], cwd=root)
            output = root / 'out'
            manifest = exporter.export(root, output, compile_language=True, lsp_command=['/nonexistent-language-server'])
            document = next(doc for doc in manifest['documents'] if doc['path'] == 'project.md')
            analysis = json.loads(json.loads((output / document['url']).read_text())['intelligence'])
            self.assertTrue(analysis['packed_spans']['data'])
            self.assertEqual(analysis['state'], 'unavailable')
            self.assertIn('no code ranges', analysis['reason'])

    def test_missing_committed_entry_is_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for args in [('init', '-q'), ('config', 'user.email', 'test@example.invalid'), ('config', 'user.name', 'Test')]:
                subprocess.check_call(['git', *args], cwd=root)
            (root / '.hickory-source').mkdir()
            (root / '.hickory-source/chapter.md').write_text('# Chapter\n')
            subprocess.check_call(['git', 'add', '.'], cwd=root)
            subprocess.check_call(['git', 'commit', '-qm', 'No project entry'], cwd=root)
            with self.assertRaises((ValueError, subprocess.CalledProcessError)):
                exporter.export(root, root / 'out')
            self.assertFalse((root / 'out/index.json').exists())

    def test_native_origins_convert_unicode_once_and_deduplicate_reused_highlights(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            code = '\nconst count = 1;\n'
            source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n' + 'Public prose 😀\n'*12000 + '<h:copy id="shared">' + code + '</h:copy>\n</h:doc>'
            start = source.index(code)
            origin = dict(output=[0, len(code.encode())], document='book.md', source=[len(source[:start].encode()), len(source[:start+len(code)].encode())])
            snapshot = dict(version=1, entry='book.md', revision='test-snapshot', sources={'book.md': source}, files=[dict(path=f'app{i}.ts', bytes=list(code.encode()), origins=[origin]) for i in range(50)])
            manifest = exporter.export(root, root/'out', snapshot=snapshot, compile_language=True)
            document = manifest['documents'][0]
            payload = json.loads((root/'out'/document['url']).read_text())
            packed = json.loads(payload['intelligence'])['packed_spans']
            triples = list(zip(packed['data'][::3], packed['data'][1::3], packed['data'][2::3]))
            keywords = [item for item in triples if packed['classes'][item[2]]=='tok-keyword']
            self.assertEqual(len(keywords), 1)
            self.assertEqual(keywords[0][0], len(source[:source.index('const count')].encode('utf-16-le'))//2)


class GenericProjectExportTests(unittest.TestCase):
    @unittest.skipUnless(os.environ.get('HICKORY_EXPORT_HICK'), 'requires the explicitly selected built CLI')
    def test_cli_exports_copied_source_and_semantic_navigation_without_a_running_lsp(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            def git(*args): return subprocess.check_output(['git', *args], cwd=root, text=True).strip()
            git('init', '-q'); git('config', 'user.email', 'export@example.invalid'); git('config', 'user.name', 'Export fixture')
            source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n# A different project\n<h:project sources="chapter.md uses.md"/>\n<h:file path="a.ts"><h:paste select="#code"/></h:file>\n<h:file path="b.ts"><h:paste select="#uses"/></h:file>\n<h:file path="main.rs">pub fn main() {}</h:file>\n</h:doc>'
            chapter = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n# λ 😀\n<h:project entry="book.md"/>\n<h:copy id="code">\n/** Preserve a count. */\nexport const count = 1; count;\n</h:copy>\n</h:doc>'
            (root/'book.md').write_text(source); (root/'chapter.md').write_text(chapter)
            (root/'uses.md').write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project entry="book.md"/><h:copy id="uses">import { count } from "./a"; export const result = count;</h:copy></h:doc>')
            (root/'private.md').write_text('PRIVATE UNDECLARED NOTE')
            git('add', '.'); git('commit', '-qm', 'Export fixture')
            revision = git('rev-parse', 'HEAD')
            (root/'chapter.md').write_text('DIRTY CONTENT')
            repository = Path(__file__).resolve().parent.parent
            model = repository/'code-models/typescript/hick-model-typescript'
            # Exercise the installed grammar path from an unrelated project.
            tools = root/'tools'; tools.mkdir()
            cli = tools/Path(os.environ['HICKORY_EXPORT_HICK']).name
            try: os.link(os.environ['HICKORY_EXPORT_HICK'], cli)
            except OSError: shutil.copy2(os.environ['HICKORY_EXPORT_HICK'], cli)
            subprocess.run(['node', 'scripts/build-static-highlights.mjs', str(tools/'hick-export/static-highlights.mjs')], cwd=repository, check=True, capture_output=True, text=True)
            completed = subprocess.run([str(cli), 'project', 'export', 'book.md', '--root', str(root), '--output', str(root/'out'), '--model', 'typescript='+str(model)], capture_output=True, text=True, timeout=120)
            self.assertEqual(completed.returncode, 0, completed.stderr)
            manifest = json.loads((root/'out/index.json').read_text())
            self.assertEqual(manifest['entry'], 'book.md'); self.assertEqual(manifest['revision'], revision)
            self.assertEqual({item['path'] for item in manifest['documents']}, {'book.md', 'chapter.md', 'uses.md'})
            root_document = next(item for item in manifest['documents'] if item['path']=='book.md')
            root_payload = json.loads(gzip.decompress((root/'out'/root_document['url']).read_bytes()))
            paint = json.loads(root_payload['intelligence'])['packed_spans']
            tokens = zip(paint['data'][::3], paint['data'][1::3], paint['data'][2::3])
            self.assertTrue(any(source.encode('utf-16-le')[start*2:(start+length)*2].decode('utf-16-le')=='pub' for start, length, kind in tokens if paint['classes'][kind]=='tok-keyword'))
            doc = next(item for item in manifest['documents'] if item['path']=='chapter.md')
            payload = json.loads(gzip.decompress((root/'out'/doc['url']).read_bytes()))
            self.assertEqual(payload['source'], chapter)
            data = json.loads(payload['intelligence'])
            self.assertEqual(data['language']['origin'], 'code-model:typescript')
            self.assertGreater(len(data['packed_spans']['data']), 0)
            occurrences = data['language']['occurrences']
            names = [item for item in occurrences if chapter.encode('utf-16-le')[item['from']*2:item['to']*2].decode('utf-16-le')=='count']
            self.assertEqual(len(names), 2)
            self.assertTrue(names[1]['hover']['result']['contents'])
            symbol = names[1]['symbolIds'][0]
            self.assertEqual(data['language']['shared'][symbol]['definition']['result'][0]['uri'], 'hick:///chapter.md')
            self.assertEqual(len(data['language']['shared'][symbol]['references']['result']), 4)
            self.assertEqual({item['uri'] for item in data['language']['shared'][symbol]['references']['result']}, {'hick:///chapter.md', 'hick:///uses.md'})
            self.assertNotIn('PRIVATE UNDECLARED NOTE', (root/'out/index.json').read_text())

if __name__ == '__main__':
    unittest.main()
