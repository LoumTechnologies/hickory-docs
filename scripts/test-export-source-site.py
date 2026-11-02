import hashlib
import importlib.util
import json
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


if __name__ == '__main__':
    unittest.main()
