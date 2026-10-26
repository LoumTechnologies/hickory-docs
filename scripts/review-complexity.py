#!/usr/bin/env python3
"""Report review complexity proxies for maintained TypeScript and Python."""
import argparse
import ast
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
VERSION = 'review-complexity-v1'
FUNCTIONS = (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)
BRANCHES = (ast.If, ast.IfExp, ast.For, ast.AsyncFor, ast.While,
            ast.ExceptHandler, ast.comprehension)


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode()).hexdigest()


def python_file(path, code):
    lines = code.splitlines(keepends=True)
    def point(line, column):
        # Python AST columns are UTF-8 byte offsets; editor positions use UTF-16.
        prefix = lines[line - 1].encode()[:column].decode()
        return dict(line=line - 1, character=len(prefix.encode('utf-16-le')) // 2)
    def span(node):
        return dict(start=point(node.lineno, node.col_offset),
                    end=point(node.end_lineno, node.end_col_offset))
    try:
        tree = ast.parse(code, filename=path)
    except SyntaxError as error:
        location = dict(line=(error.lineno or 1) - 1, character=len((lines[(error.lineno or 1) - 1][:(error.offset or 1) - 1]).encode('utf-16-le')) // 2)
        return dict(path=path, errors=[dict(message=error.msg, range=dict(start=location, end=location))])
    def call_target(node):
        if isinstance(node, ast.Name):
            return ['function', node.id]
        if isinstance(node, ast.Attribute):
            return ['member', node.attr, call_target(node.value) if isinstance(node.value, ast.Attribute) else ['receiver']]
        return canonical(node)
    def canonical(node, root=False):
        if isinstance(node, list):
            return [canonical(n) for n in node]
        if not isinstance(node, ast.AST):
            return node
        if isinstance(node, FUNCTIONS + (ast.ClassDef,)) and not root:
            return ['nested-definition']
        if isinstance(node, ast.Name):
            return ['name']
        if isinstance(node, ast.Constant):
            return ['literal', type(node.value).__name__]
        values = []
        for key, value in ast.iter_fields(node):
            if key in ('ctx', 'annotation', 'returns', 'type_comment', 'type_params', 'decorator_list'):
                continue
            if isinstance(node, ast.Call) and key == 'func':
                value = call_target(value)
            elif key in ('id', 'arg', 'name'):
                value = '<name>'
            values.append([key, canonical(value)])
        return [type(node).__name__, *values]
    units, declarations, imports = [], [], set()
    def measure(body, node, name):
        sites, calls, count, nesting = [], set(), 0, 0
        def walk(item, depth):
            nonlocal count, nesting
            if isinstance(item, FUNCTIONS + (ast.ClassDef,)):
                return
            if isinstance(item, (ast.Load, ast.Store, ast.Del)):
                return
            count += 1
            decisions = len(item.values) - 1 if isinstance(item, ast.BoolOp) else int(isinstance(item, BRANCHES))
            if isinstance(item, ast.match_case):
                decisions = int(not (isinstance(item.pattern, ast.MatchAs) and item.pattern.pattern is None))
            if decisions:
                location = item if hasattr(item, 'lineno') else item.pattern if isinstance(item, ast.match_case) else item.target
                sites.extend([span(location)] * decisions)
                depth += 1
                nesting = max(nesting, depth)
            if isinstance(item, ast.Call):
                calls.add(ast.unparse(item.func) if isinstance(item.func, (ast.Name, ast.Attribute)) else '<computed>')
            for key, value in ast.iter_fields(item):
                if key in ('annotation', 'returns', 'type_comment', 'type_params', 'decorator_list'):
                    continue
                for child in value if isinstance(value, list) else [value]:
                    if isinstance(child, ast.AST):
                        walk(child, depth)
        for item in body:
            walk(item, 0)
        return dict(name=name, range=span(node), node_count=count, decision_sites=sites,
                    max_nesting=nesting, calls=sorted(calls), pattern=canonical(body))
    for node in ast.walk(tree):
        if isinstance(node, FUNCTIONS):
            body = [node.body] if isinstance(node, ast.Lambda) else node.body
            # A docstring is explanatory data, not executable reasoning.
            if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant) and isinstance(body[0].value.value, str):
                body = body[1:]
            units.append(measure(body, node, getattr(node, 'name', '<lambda>')))
        if isinstance(node, ast.AnnAssign):
            declarations.append(dict(kind='field', range=span(node)))
        if isinstance(node, ast.Import):
            imports.update(alias.name for alias in node.names)
        if isinstance(node, ast.ImportFrom):
            imports.add('.' * node.level + (node.module or ''))
    top = [n for n in tree.body if not isinstance(n, FUNCTIONS + (ast.ClassDef, ast.Import, ast.ImportFrom))]
    if top:
        unit = measure(top, top[0], '<module>')
        unit['range']['end'] = span(top[-1])['end']
        units.append(unit)
    return dict(path=path, errors=[], units=units, declarations=declarations, imports=sorted(imports))


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def read(path, revision=None):
    if revision:
        result = subprocess.run(['git', 'show', revision + ':' + path], cwd=ROOT, capture_output=True)
        if result.returncode:
            return None
        return result.stdout.decode('utf-8')
    target = ROOT / path
    if not target.is_file():
        return None
    if not target.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError('Source escapes the repository: ' + path)
    return target.read_text(encoding='utf-8')


def role(path, constructor):
    if constructor.get('role') == 'generated-code' or constructor.get('generated_by'):
        return 'generated'
    if constructor.get('role') == 'test':
        return 'test'
    if re.search(r'(^|/)(tests?|__tests__)/|(^|/)(test_[^/]+\.py)|\.(test|spec)\.[cm]?[jt]sx?$', path):
        return 'test'
    return 'implementation'


def source_segments(path, code, constructor, narrative, revision):
    # Reconstruct copy ranges from the maintained payloads. Refuse to invent a
    # location when a raw construction or stale output prevents exact matching.
    fragments = {f['id']: f for f in narrative.get('fragments', [])}
    result, cursor = [], 0
    for fid in constructor.get('fragments', []):
        owner = fragments.get(fid)
        text = read(owner['page'], revision) if owner else None
        match = re.search(r'<([\w.-]+):copy\b[^>]*\bid="' + re.escape(fid) + r'"[^>]*>\n(.*?)</\1:copy>', text, re.S) if text else None
        if not match:
            return []
        payload = match[2]
        result.append(dict(output_start=cursor, output_end=cursor + len(payload),
                           page=owner['page'], fragment=fid, source_start=match.start(2), source_text=text))
        cursor += len(payload)
    if not result or ''.join(s['source_text'][s['source_start']:s['source_start'] + s['output_end'] - s['output_start']] for s in result) != code:
        return []
    return result


def offset(code, point):
    lines = code.splitlines(keepends=True)
    prefix = lines[point['line']].encode('utf-16-le')[:point['character'] * 2].decode('utf-16-le') if point['line'] < len(lines) else ''
    return sum(len(line) for line in lines[:point['line']]) + len(prefix)


def position(code, offset):
    prefix = code[:offset]
    return dict(line=prefix.count('\n'), character=len(prefix.rsplit('\n', 1)[-1].encode('utf-16-le')) // 2)


def anchors(span, code, segments):
    start, end = offset(code, span['start']), offset(code, span['end'])
    result = []
    for segment in segments:
        a, b = max(start, segment['output_start']), min(end, segment['output_end'])
        if a < b:
            location = segment['source_start'] - segment['output_start']
            result.append(dict(page=segment['page'], fragment=segment['fragment'], range=dict(
                start=position(segment['source_text'], a + location), end=position(segment['source_text'], b + location))))
    return result


def snapshot(paths, revision=None, min_nodes=4):
    inventory = json.loads(read('hickory-dogfood/behavior-map.json', revision))['constructors']
    narrative = json.loads(read('hickory-dogfood/narrative-map.json', revision))
    pending, skipped, files, codes = [], [], [], {}
    for path in sorted(set(paths)):
        constructor = inventory.get(path)
        if not constructor:
            skipped.append(dict(path=path, reason='not-in-construction-inventory'))
            continue
        if role(path, constructor) == 'generated':
            skipped.append(dict(path=path, reason='generated-output'))
            continue
        if Path(path).suffix not in ('.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs', '.py'):
            skipped.append(dict(path=path, reason='unsupported-language'))
            continue
        code = read(path, revision)
        if code is None:
            raise ValueError('Inventoried source is missing: ' + path)
        codes[path] = code
        if path.endswith('.py'):
            files.append(python_file(path, code))
        else:
            pending.append(dict(path=path, code=code))
    parser = None
    if pending:
        run = subprocess.run(['node', str(ROOT / 'scripts/review-complexity-ts.mjs')],
                             input=json.dumps(pending), capture_output=True, text=True, check=True)
        parsed = json.loads(run.stdout)
        files.extend(parsed['files'])
        parser = parsed['parser']
    groups = defaultdict(list)
    shapes = {}
    source_hashes = {}
    totals = {category: Counter() for category in ('implementation', 'test')}
    for file in sorted(files, key=lambda f: f['path']):
        path = file['path']
        file['content_sha256'] = hashlib.sha256(codes[path].encode()).hexdigest()
        file['owner'] = inventory[path].get('source')
        file['category'] = role(path, inventory[path])
        file['language'] = 'python' if path.endswith('.py') else 'typescript' if Path(path).suffix in ('.ts', '.tsx') else 'javascript'
        if file['errors']:
            continue
        segments = source_segments(path, codes[path], inventory[path], narrative, revision)
        file['source_mapping'] = 'exact-copy-fragments' if segments else 'unavailable'
        for segment in segments:
            source_hashes[segment['page']] = hashlib.sha256(segment['source_text'].encode()).hexdigest()
        total = totals[file['category']]
        total['files'] += 1
        total['declarations'] += len(file['declarations'])
        total['import_edges'] += len(file['imports'])
        for site in file['declarations']:
            site['source'] = anchors(site['range'], codes[path], segments)
        for unit in file['units']:
            shape = unit.pop('pattern')
            unit['pattern_id'] = digest([file['language'], shape])
            shapes[(file['category'], file['language'], unit['pattern_id'])] = shape
            unit['source'] = anchors(unit['range'], codes[path], segments)
            for site in unit['decision_sites']:
                site['source'] = anchors(site, codes[path], segments)
            total['units'] += 1
            total['decision_points'] += len(unit['decision_sites'])
            total['max_nesting'] = max(total['max_nesting'], unit['max_nesting'])
            total['max_unit_decisions'] = max(total['max_unit_decisions'], len(unit['decision_sites']))
            total['max_unit_nodes'] = max(total['max_unit_nodes'], unit['node_count'])
            if unit['node_count'] >= min_nodes and unit['name'] != '<module>':
                groups[(file['category'], file['language'], unit['pattern_id'])].append(dict(
                    path=path, name=unit['name'], range=unit['range'], source=unit['source']))
    repeats = []
    for (category, language, pattern), occurrences in sorted(groups.items()):
        if len(occurrences) < 2:
            continue
        spread = len({o['path'] for o in occurrences})
        totals[category]['repeat_groups'] += 1
        totals[category]['repeat_excess_units'] += len(occurrences) - 1
        totals[category]['max_repeat_files'] = max(totals[category]['max_repeat_files'], spread)
        repeats.append(dict(pattern_id=pattern, category=category, language=language,
                            instances=len(occurrences), files=spread, normalized_shape=shapes[(category, language, pattern)], occurrences=occurrences))
    fields = ('files', 'units', 'declarations', 'import_edges', 'decision_points', 'max_nesting',
              'max_unit_decisions', 'max_unit_nodes', 'repeat_groups', 'repeat_excess_units', 'max_repeat_files')
    report = dict(parsers=dict(python=platform.python_version(), typescript=parser),
                source_sha256=dict(sorted(source_hashes.items())),
                totals={c: {f: total[f] for f in fields} for c, total in totals.items()},
                files=sorted(files, key=lambda f: f['path']), skipped=skipped, repeats=repeats)
    report['snapshot_id'] = digest(report)
    return report


def build(paths, base=None, min_nodes=4):
    if min_nodes < 1:
        raise ValueError('min-nodes must be positive')
    paths = sorted(set(paths))
    for path in paths:
        if Path(path).is_absolute() or '..' in Path(path).parts:
            raise ValueError('Expected repository-relative paths: ' + path)
    revision = git('rev-parse', '--verify', base + '^{commit}').decode().strip() if base else None
    known = set(json.loads(read('hickory-dogfood/behavior-map.json'))['constructors'])
    if revision:
        known.update(json.loads(read('hickory-dogfood/behavior-map.json', revision))['constructors'])
    if set(paths) - known:
        raise ValueError('Unknown constructed paths: ' + ', '.join(sorted(set(paths) - known)))
    current = snapshot(paths, min_nodes=min_nodes)
    result = dict(format=1, algorithm=VERSION, min_nodes=min_nodes, scope=paths, current=current,
                  analyzer_sha256=digest([(p, hashlib.sha256((ROOT / p).read_bytes()).hexdigest())
                    for p in ('scripts/review-complexity.py', 'scripts/review-complexity-ts.mjs')]))
    if base:
        previous = snapshot(paths, revision, min_nodes)
        result.update(base_revision=revision, previous=previous,
                      delta={c: {k: current['totals'][c][k] - previous['totals'][c][k]
                                 for k in current['totals'][c]} for c in current['totals']})
    result['complete'] = not any(f['errors'] for snap in (result.get('previous'), current) if snap for f in snap['files'])
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('paths', nargs='*', help='constructed output paths; defaults to inventory union')
    parser.add_argument('--base', help='compare with a Git commit using the same analyzer and parsers')
    parser.add_argument('--min-nodes', type=int, default=4)
    parser.add_argument('--output', type=Path, help='write JSON here instead of stdout')
    args = parser.parse_args()
    paths = args.paths
    if not paths:
        paths = list(json.loads(read('hickory-dogfood/behavior-map.json'))['constructors'])
        if args.base:
            revision = git('rev-parse', '--verify', args.base + '^{commit}').decode().strip()
            paths += list(json.loads(read('hickory-dogfood/behavior-map.json', revision))['constructors'])
    result = build(paths, args.base, args.min_nodes)
    text = json.dumps(result, sort_keys=True, indent=2, ensure_ascii=True) + '\n'
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(text)
    else:
        print(text, end='')
    return 0 if result['complete'] else 2


if __name__ == '__main__':
    sys.exit(main())
