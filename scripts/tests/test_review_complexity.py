"""Focused checks for metric invariants and truthful source locations."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('review_complexity', ROOT / 'scripts/review-complexity.py')
metric = importlib.util.module_from_spec(spec)
spec.loader.exec_module(metric)


def typescript(code):
    result = subprocess.run(['node', str(ROOT / 'scripts/review-complexity-ts.mjs')],
                            input=json.dumps([dict(path='sample.ts', code=code)]),
                            capture_output=True, text=True, check=True)
    return json.loads(result.stdout)['files'][0]


def functions(file):
    return [u for u in file['units'] if u['name'] != '<module>']


class StructureTests(unittest.TestCase):
    def test_typescript_ignores_layout_comments_names_literals_and_types(self):
        a = functions(typescript('const a = (x: string) => request<string>("GET", `/doc/${x}`);'))[0]
        b = functions(typescript('// comment\nconst renamed = (other: number) =>\n ((request<number>( "POST", `/item/${other}` )));'))[0]
        self.assertEqual(a['pattern'], b['pattern'])
        self.assertEqual(a['node_count'], b['node_count'])
        changed = functions(typescript('const a = (x: string) => encodeURIComponent(x);'))[0]
        self.assertNotEqual(a['pattern'], changed['pattern'])

    def test_python_ignores_docstrings_names_literals_and_annotations(self):
        a = functions(metric.python_file('a.py', 'def a(x: str):\n    """Explanation."""\n    return send(x, 1)\n'))[0]
        b = functions(metric.python_file('a.py', '# comment\ndef renamed(other: int):\n    return send(other, 200)\n'))[0]
        self.assertEqual(a['pattern'], b['pattern'])
        self.assertEqual(a['node_count'], b['node_count'])

    def test_branches_and_nested_functions_are_counted_once(self):
        file = metric.python_file('a.py', 'def outer(x):\n    def inner(y):\n        if y:\n            return 1\n    if x:\n        while x:\n            x -= 1\n    return inner(x)\n')
        units = {u['name']: u for u in functions(file)}
        self.assertEqual(len(units['outer']['decision_sites']), 2)
        self.assertEqual(units['outer']['max_nesting'], 2)
        self.assertEqual(len(units['inner']['decision_sites']), 1)
        ts = functions(typescript('const f = x => { if (x) { while (x) { x--; } } };'))[0]
        self.assertEqual(len(ts['decision_sites']), 2)
        self.assertEqual(ts['max_nesting'], 2)

    def test_typed_declarations_have_no_imperative_units(self):
        file = typescript('interface Routes { read(id: string): string; save(body: string): void; }')
        self.assertEqual(len(file['declarations']), 2)
        self.assertEqual(file['units'], [])

    def test_syntax_errors_are_reported(self):
        self.assertTrue(typescript('const f = ( => {')['errors'])
        self.assertTrue(metric.python_file('a.py', 'def broken(:\n')['errors'])

    def test_unicode_ranges_use_utf16_in_both_languages(self):
        code = 'def f():\n    return send("😀", 1)\n'
        unit = functions(metric.python_file('a.py', code))[0]
        self.assertEqual(metric.offset(code, unit['range']['end']), len(code.rstrip('\n')))
        point = metric.position('😀x', 1)
        self.assertEqual(point, dict(line=0, character=2))
        ts_code = 'const f = () => send("😀", 1);'
        unit = functions(typescript(ts_code))[0]
        self.assertEqual(ts_code[metric.offset(ts_code, unit['range']['start']):metric.offset(ts_code, unit['range']['end'])], 'send("😀", 1)')


class InventoryTests(unittest.TestCase):
    def fixture(self, root):
        code = 'def a(x):\n    return send(x, 1)\n\ndef b(y):\n    return send(y, 2)\n'
        test = 'def test_a(x):\n    return send(x, 3)\n'
        for path, data in [('src.py', code), ('test_api.py', test), ('derived.py', 'not python!')]:
            (root / path).write_text(data)
        page = '<doc:copy id="first">\n' + code[:code.index('def b')] + '</doc:copy>\nExplanation.\n<doc:copy id="second">\n' + code[code.index('def b'):] + '</doc:copy>'
        (root / 'owner.md').write_text(page)
        constructors = {'src.py': dict(source='owner.md', fragments=['first', 'second']),
                        'test_api.py': dict(source='test.md'),
                        'derived.py': dict(role='generated-code')}
        folder = root / 'hickory-dogfood'
        folder.mkdir()
        (folder / 'behavior-map.json').write_text(json.dumps(dict(constructors=constructors)))
        (folder / 'narrative-map.json').write_text(json.dumps(dict(fragments=[dict(id=i, page='owner.md') for i in ('first', 'second')])))
        return code, page

    def test_exclusion_separate_tests_dispersion_and_exact_source_mapping(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(metric, 'ROOT', Path(tmp)):
            root = Path(tmp)
            code, page = self.fixture(root)
            result = metric.snapshot(['test_api.py', 'src.py', 'derived.py'])
            self.assertEqual(result, metric.snapshot(['derived.py', 'src.py', 'test_api.py', 'src.py']))
            self.assertEqual(result['totals']['implementation']['repeat_excess_units'], 1)
            self.assertEqual(result['totals']['test']['repeat_excess_units'], 0)
            self.assertEqual(result['totals']['test']['files'], 1)
            self.assertEqual(result['skipped'], [dict(path='derived.py', reason='generated-output')])
            file = next(f for f in result['files'] if f['path'] == 'src.py')
            second = next(u for u in file['units'] if u['name'] == 'b')
            location, = second['source']
            self.assertEqual(location['fragment'], 'second')
            self.assertTrue(page[metric.offset(page, location['range']['start']):metric.offset(page, location['range']['end'])].startswith('def b'))
            (root / 'src.py').write_text(code + '# stale output\n')
            stale = metric.snapshot(['src.py'])['files'][0]
            self.assertEqual(stale['source_mapping'], 'unavailable')
            self.assertTrue(all(not u['source'] for u in stale['units']))

    def test_splitting_repeats_across_files_increases_dispersion(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(metric, 'ROOT', Path(tmp)):
            root = Path(tmp)
            self.fixture(root)
            path = root / 'hickory-dogfood/behavior-map.json'
            data = json.loads(path.read_text())
            data['constructors']['other.py'] = dict(source='other.md')
            path.write_text(json.dumps(data))
            (root / 'other.py').write_text('def c(z):\n    return send(z, 3)\n')
            result = metric.snapshot(['src.py', 'other.py'])
            self.assertEqual(result['totals']['implementation']['max_repeat_files'], 2)
            self.assertEqual(result['totals']['implementation']['repeat_excess_units'], 2)

    def test_comparison_uses_each_inventory_and_exposes_central_complexity(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(metric, 'ROOT', Path(tmp)):
            root = Path(tmp)
            self.fixture(root)
            (root / 'scripts').mkdir()
            for name in ('review-complexity.py', 'review-complexity-ts.mjs'):
                (root / 'scripts' / name).write_text('fixture analyzer identity')
            def git(*args):
                return subprocess.check_output(['git', '-c', 'core.hooksPath=/dev/null', *args], cwd=root, stderr=subprocess.DEVNULL)
            git('init', '-q')
            git('add', '.')
            git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@local', 'commit', '-qm', 'baseline')
            (root / 'src.py').write_text('def central(x):\n    if x:\n        while x:\n            x -= 1\n    return send(x, 1)\n')
            path = root / 'hickory-dogfood/behavior-map.json'
            inventory = json.loads(path.read_text())
            inventory['constructors']['routes.py'] = dict(source='routes.md')
            path.write_text(json.dumps(inventory))
            (root / 'routes.py').write_text('route: str\n')
            result = metric.build(['src.py', 'routes.py', 'derived.py'], 'HEAD')
            self.assertEqual(result, metric.build(['derived.py', 'routes.py', 'src.py'], 'HEAD'))
            self.assertTrue(result['complete'])
            self.assertEqual(result['previous']['totals']['implementation']['repeat_excess_units'], 1)
            self.assertEqual(result['current']['totals']['implementation']['repeat_excess_units'], 0)
            self.assertEqual(result['delta']['implementation']['max_unit_decisions'], 2)
            self.assertEqual(result['delta']['implementation']['declarations'], 1)
            self.assertIn(dict(path='routes.py', reason='not-in-construction-inventory'), result['previous']['skipped'])

    def test_errors_mark_comparison_incomplete_and_unknown_paths_refuse(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(metric, 'ROOT', Path(tmp)):
            root = Path(tmp)
            self.fixture(root)
            (root / 'scripts').mkdir()
            for name in ('review-complexity.py', 'review-complexity-ts.mjs'):
                (root / 'scripts' / name).write_text('fixture identity')
            (root / 'src.py').write_text('def broken(:\n')
            self.assertFalse(metric.build(['src.py'])['complete'])
            with self.assertRaises(ValueError):
                metric.build(['typo.py'])

    def test_rejects_paths_outside_the_repository(self):
        with self.assertRaises(ValueError):
            metric.build(['../outside.py'])


if __name__ == '__main__':
    unittest.main()
