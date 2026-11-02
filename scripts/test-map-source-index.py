import hashlib
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('projection', Path(__file__).with_name('map-source-index.py'))
projection = importlib.util.module_from_spec(spec)
spec.loader.exec_module(projection)


class ProjectionTests(unittest.TestCase):
    def fixture(self):
        code = 'const name = "😀"; name;\n'
        prefix = '# λ 😀\n<copy>\n'
        source = prefix + code + '</copy>'
        raw = code.encode()
        declaration = code.index('name')
        use = len(code[:code.rindex('name')].encode('utf-16-le'))//2
        file = dict(path='app.ts', bytes=list(raw), origins=[dict(output=[0, len(raw)], document='chapter.md', source=[len(prefix.encode()), len((prefix+code).encode())])])
        indexed = dict(path='app.ts', sha256=hashlib.sha256(raw).hexdigest(), length=len(code.encode('utf-16-le'))//2,
            occurrences=[dict(from_=position, to=position+4, symbols=['symbol'], type='string', isDefinition=i==0, isWrite=i==0, unresolved=False) for i, position in enumerate([declaration, use])], calls=[], folds=[], classifications=[], hints=[])
        for occurrence in indexed['occurrences']: occurrence['from'] = occurrence.pop('from_')
        index = dict(version=1, coordinateEncoding='utf-16', files=[indexed], symbols=[dict(id='symbol', name='name', documentation='A name.', declarations=[dict(file='app.ts', from_=declaration, to=declaration+4)])], diagnostics=[])
        index['symbols'][0]['declarations'][0]['from'] = index['symbols'][0]['declarations'][0].pop('from_')
        return index, [dict(path='chapter.md', source=source)], [file]

    def test_unicode_mapping_and_shared_reference_answers(self):
        index, documents, files = self.fixture()
        pages = projection.project_index(index, documents, files)
        page = pages['chapter.md']
        occurrence = page['occurrences'][0]
        source = documents[0]['source']
        start = len(source[:source.index('name')].encode('utf-16-le'))//2
        self.assertEqual(occurrence['from'], start)
        self.assertEqual(occurrence['position'], dict(line=2, character=6))
        self.assertEqual(page['origin'], 'code-model:typescript')
        self.assertEqual(len(page['shared']), 1)
        self.assertEqual(len(page['shared']['symbol']['references']['result']), 2)
        self.assertEqual(page['shared']['symbol']['definition']['result'][0]['range']['start'], dict(line=2, character=6))
        self.assertNotIn('references', occurrence)
        self.assertTrue(occurrence['hover']['result']['contents'])

    def test_nested_call_signatures_use_the_innermost_context(self):
        index, documents, files = self.fixture()
        first, second = index['files'][0]['occurrences']
        index['files'][0]['calls'] = [dict(from_=0, to=second['to'], signature='outer'), dict(from_=second['from'], to=second['to'], signature='inner')]
        for call in index['files'][0]['calls']: call['from'] = call.pop('from_')
        page = projection.project_index(index, documents, files)['chapter.md']
        self.assertEqual(page['occurrences'][0]['signatureHelp']['result']['signatures'], [dict(label='outer')])
        self.assertEqual(page['occurrences'][1]['signatureHelp']['result']['signatures'], [dict(label='inner')])

    def test_hash_and_origin_mismatches_are_refused(self):
        index, documents, files = self.fixture()
        index['files'][0]['sha256'] = '0'*64
        with self.assertRaisesRegex(ValueError, 'constructed bytes'): projection.project_index(index, documents, files)
        index, documents, files = self.fixture()
        files[0]['origins'][0]['source'][0] += 1
        with self.assertRaisesRegex(ValueError, 'preserve indexed bytes'): projection.project_index(index, documents, files)

    def test_coordinate_conversion_is_exact_at_every_character_boundary(self):
        text = 'aλ😀\n𝄞z'
        coordinates = projection.Coordinates(text)
        for i in range(len(text)+1):
            byte, units = len(text[:i].encode()), len(text[:i].encode('utf-16-le'))//2
            self.assertEqual(coordinates.byte_at(units), byte)
            self.assertEqual(coordinates.utf16_at(byte), units)
        windows = projection.Coordinates('a\r\n😀\rb')
        self.assertEqual(windows.position(3), dict(line=1, character=0))
        self.assertEqual(windows.position(6), dict(line=2, character=0))
        with self.assertRaises(KeyError): coordinates.byte_at(3)
        with self.assertRaises(KeyError): coordinates.utf16_at(2)

if __name__ == '__main__': unittest.main()
