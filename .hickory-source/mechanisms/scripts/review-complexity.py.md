<review:doc xmlns:review="http://www.hickorydocs.com/1.0">
# Review repeated reasoning in maintained source

For maintainers evaluating a refactor and designers adding editor review hints.
Version 1 parses TypeScript, JavaScript and Python using installed local parsers.
The scorecard reports syntax-based proxies for repeated reasoning. It cannot infer
business-policy equivalence or measure a person's understanding or review time.

Start with the API rewrite, using installed `apps/web` dependencies and Python 3:

```sh
just review-complexity --base c87bcaee --output target/dogfood/review-complexity.json apps/web/src/api/client.ts apps/web/src/api/routes.ts scripts/generate-api.py apps/web/src/api/generatedApi.ts apps/web/src/api/client.test.ts
```

The current analyzer and parsers read both revisions. The historical constructor
inventory decides which files existed and which were generated. Unsupported
languages and absent inventory records are explicitly listed; parse errors make
`complete` false and the CLI exits 2. Omitting paths analyzes the inventory union.
Generated outputs are excluded by inventory metadata; generators count as ordinary
maintained code. Tests have a separate lane and never create implementation clones.

Run the focused invariance and source-location witnesses with
`just test-review-complexity`. The web CI and local web check plan run the same
witnesses. These checks exercise the analyzer; they do not gate product code on
a complexity threshold.

## Read the scorecard

- `repeat_excess_units` counts occurrences beyond the first in each normalized
  function-body group of at least four executable AST nodes. `repeat_groups` and
  `max_repeat_files` expose how many patterns repeat and their largest file spread.
  Module bodies contribute complexity but are excluded from clone groups.
- `declarations` counts TypeScript fields, method signatures and type aliases, and
  Python annotated assignments. These are typed-declaration counts, not a count
  of all configuration entries or a claim that every declaration is easy.
- `decision_points`, `max_unit_decisions` and `max_nesting` expose central logic.
  A branch is an if/ternary, loop, catch, non-default switch/match case, boolean
  choice or Python comprehension. Nesting is syntactic branch depth; logical
  expression trees also affect it. No subjective weights are applied.
- `import_edges` counts distinct imported modules per file. `max_unit_nodes`
  measures the largest executable body. Node counts depend on language ASTs and
  must not be interpreted as a common cross-language complexity scale.

Identifiers, literal values and TypeScript grouping parentheses are normalized, while operators, literal kinds,
called function names and accessed member names remain. Types, imports and nested function
bodies are excluded from their enclosing function's pattern. Python docstrings
are excluded. Each nested function is measured independently. Different-language
patterns never merge. Matching syntax is a review candidate, not proof of shared
policy; parameter renames used as direct call targets still change a pattern.

There is no aggregate quality score or CI threshold. Concentrating code can reduce
repeats while increasing the generator's decisions and dependencies. Review both.
Parser versions, analyzer digest, threshold and scope travel with every report.
The JSON has no wall-clock timestamp or machine-specific root. Locations naturally
change when formatting changes; structural totals and patterns should remain stable.

## Normalize and measure Python

Use Python's parser and report malformed input instead of inventing a score.
Both parsers return zero-based UTF-16 ranges, matching editor/LSP positions.
Function metrics cover executable bodies; decorators, default arguments and class
initializers are outside this first version's complexity coverage. The report is
an aid to review, not exhaustive static analysis.

## Python structure

Normalize bodies without evaluating source. Preserve call targets so unrelated
functions with similar argument shapes do not automatically merge. Count each
function independently, excluding nested definitions from parent totals.

<review:copy id="review-metric-python-structure">
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


</review:copy>

## Inventory and source

Read each revision with its own inventory. An exact concatenation of copy
payloads is required before mapping to Markdown. A raw construction or stale
output produces an explicit unavailable mapping, rather than a guessed gutter mark.

<review:copy id="review-metric-inventory-and-source">
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


</review:copy>

## Editor ranges

Convert UTF-16 positions to offsets, intersect the finding with each copy
fragment, then convert the corresponding Markdown spans back to UTF-16. Findings
can span several owning pages; retain every intersection.

<review:copy id="review-metric-editor-ranges">
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


</review:copy>

## Snapshot scorecard

Parse supported maintained files in a batch. Separate tests before grouping,
retain every occurrence and its owner, and report generated or unsupported
inputs explicitly. Stable iteration order makes identical snapshots reproducible.

<review:copy id="review-metric-snapshot-scorecard">
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


</review:copy>

## Compare and command

Compare both revisions with the same algorithm, parser versions and threshold.
The report retains the previous and current measurements plus numeric deltas;
errors remain visible and prevent a successful complete-report exit.

<review:copy id="review-metric-compare-and-command">
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
</review:copy>

<review:file path="scripts/review-complexity.py"><review:paste select="#review-metric-python-structure"/><review:paste select="#review-metric-inventory-and-source"/><review:paste select="#review-metric-editor-ranges"/><review:paste select="#review-metric-snapshot-scorecard"/><review:paste select="#review-metric-compare-and-command"/></review:file>

## Typescript structure

The locked application TypeScript package supplies the parser. Import resolution
and type checking are deliberately not required to measure syntax. Skip erased
type syntax and represent nested functions opaquely so bodies are not duplicated
in parent clone patterns. Keep call targets and member names in the pattern.

<review:copy id="review-metric-typescript-structure">
// Parse TypeScript without resolving imports or executing project code.
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
const require = createRequire(new URL('../apps/web/package.json', import.meta.url));
const ts = require('typescript');
const K = ts.SyntaxKind;
const functionKinds = new Set([K.FunctionDeclaration, K.FunctionExpression, K.ArrowFunction,
  K.MethodDeclaration, K.Constructor, K.GetAccessor, K.SetAccessor]);
const decisions = new Set([K.IfStatement, K.ConditionalExpression, K.ForStatement,
  K.ForInStatement, K.ForOfStatement, K.WhileStatement, K.DoStatement,
  K.CaseClause, K.CatchClause]);
const logical = new Set([K.AmpersandAmpersandToken, K.BarBarToken, K.QuestionQuestionToken]);
const literalKinds = new Set([K.StringLiteral, K.NoSubstitutionTemplateLiteral, K.NumericLiteral,
  K.BigIntLiteral, K.TrueKeyword, K.FalseKeyword, K.NullKeyword, K.RegularExpressionLiteral,
  K.TemplateHead, K.TemplateMiddle, K.TemplateTail]);
const callee = node => ts.isIdentifier(node) ? node.text : ts.isPropertyAccessExpression(node)
  ? callee(node.expression) + '.' + node.name.text : '<computed>';

function analyze(input) {
  const file = ts.createSourceFile(input.path, input.code, ts.ScriptTarget.Latest, true,
    input.path.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
  const point = pos => {
    const p = file.getLineAndCharacterOfPosition(pos);
    return { line: p.line, character: p.character };
  };
  const range = node => ({ start: point(node.getStart(file)), end: point(node.end) });
  const errors = file.parseDiagnostics.map(d => ({ message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
    range: { start: point(d.start ?? 0), end: point((d.start ?? 0) + (d.length ?? 0)) } }));
  if (errors.length) return { path: input.path, errors };
  const units = [], declarationSites = [], imports = new Set();
  function canonical(node, root = false) {
    if (ts.isParenthesizedExpression(node)) return canonical(node.expression, root);
    if (ts.isTypeNode(node) || ts.isTypeParameterDeclaration(node) || ts.isImportDeclaration(node)) return null;
    if (!root && functionKinds.has(node.kind)) return ['nested-function'];
    // Annotations and generic arguments are erased. Keep operators and call targets.
    if (ts.isIdentifier(node)) {
      const p = node.parent;
      if (ts.isCallExpression(p) && p.expression === node) return ['call-name', node.text];
      if (ts.isPropertyAccessExpression(p) && p.name === node) return ['member', node.text];
      return ['identifier'];
    }
    if (literalKinds.has(node.kind)) return ['literal', [K.StringLiteral, K.NoSubstitutionTemplateLiteral].includes(node.kind) ? 'string' : [K.TrueKeyword, K.FalseKeyword].includes(node.kind) ? 'boolean' : K[node.kind]];
    if (ts.isTemplateSpan(node)) return ['template-span', canonical(node.expression)];
    const children = [];
    ts.forEachChild(node, child => { const v = canonical(child); if (v) children.push(v); });
    return [K[node.kind], ...children];
  }
  function measure(node, label, rootNode = node) {
    let nodeCount = 0, maxNesting = 0;
    const sites = [], calls = new Set();
    function walk(n, depth) {
      if (ts.isParenthesizedExpression(n)) { walk(n.expression, depth); return; }
      if (ts.isTypeNode(n) || ts.isTypeParameterDeclaration(n)) return;
      if (n !== rootNode && functionKinds.has(n.kind)) return;
      nodeCount++;
      const branch = decisions.has(n.kind) || (ts.isBinaryExpression(n) && logical.has(n.operatorToken.kind));
      if (branch) { sites.push(range(n)); depth++; maxNesting = Math.max(maxNesting, depth); }
      if (ts.isCallExpression(n) || ts.isNewExpression(n)) calls.add(callee(n.expression));
      ts.forEachChild(n, child => walk(child, depth));
    }
    walk(node, 0);
    return { name: label, range: range(node), node_count: nodeCount,
      decision_sites: sites, max_nesting: maxNesting, calls: [...calls].sort(),
      pattern: canonical(node, true) };
  }
  function visit(node) {
    if (ts.isImportDeclaration(node) && ts.isStringLiteral(node.moduleSpecifier)) imports.add(node.moduleSpecifier.text);
    if (ts.isPropertySignature(node) || ts.isMethodSignature(node) || ts.isTypeAliasDeclaration(node)) {
      declarationSites.push({ kind: ts.isMethodSignature(node) ? 'method' : ts.isPropertySignature(node) ? 'field' : 'alias', range: range(node) });
    }
    if (functionKinds.has(node.kind) && node.body) {
      let label = node.name ? node.name.getText(file) : '<anonymous>';
      if (label === '<anonymous>' && (ts.isPropertyAssignment(node.parent) || ts.isVariableDeclaration(node.parent))) label = node.parent.name.getText(file);
      units.push(measure(node.body, label));
    }
    ts.forEachChild(node, visit);
  }
  visit(file);
  // Top-level runtime statements are a separate unit. Declarations/imports do
  // not duplicate function metrics; initializers containing calls still count.
  const top = file.statements.filter(n => !functionKinds.has(n.kind) && !ts.isClassDeclaration(n)
    && !ts.isInterfaceDeclaration(n) && !ts.isTypeAliasDeclaration(n) && !ts.isImportDeclaration(n)
    && !ts.isExportDeclaration(n));
  if (top.length) {
    const module = measure(file, '<module>');
    // Re-measure only selected top-level statements to exclude declaration syntax.
    const parts = top.map(n => measure(n, '<module>'));
    module.node_count = parts.reduce((s, p) => s + p.node_count, 0);
    module.decision_sites = parts.flatMap(p => p.decision_sites);
    module.max_nesting = Math.max(0, ...parts.map(p => p.max_nesting));
    module.calls = [...new Set(parts.flatMap(p => p.calls))].sort();
    module.pattern = ['module', ...top.map(n => canonical(n))];
    units.push(module);
  }
  return { path: input.path, errors: [], units, declarations: declarationSites, imports: [...imports].sort() };
}
const inputs = JSON.parse(readFileSync(0, 'utf8'));
process.stdout.write(JSON.stringify({ parser: `typescript-${ts.version}`, files: inputs.map(analyze) }));
</review:copy>

<review:file path="scripts/review-complexity-ts.mjs"><review:paste select="#review-metric-typescript-structure"/></review:file>

## Structural witnesses

These focused tests check invariance under layout, comments, ordinary parameter
renames and literal changes; distinct call targets must remain distinct. Branch
counts, typed declarations, syntax errors and UTF-16 spans have independent
expected outcomes.

<review:copy id="review-metric-structural-witnesses">
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


</review:copy>

## Inventory witnesses

Generated code deliberately contains invalid syntax in this fixture: parsing it
would fail the test. Test bodies cannot contribute implementation clones. A
stale constructed output must lose its source mapping, and splitting a pattern
across files must increase the reported dispersion.

<review:copy id="review-metric-inventory-witnesses">
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
</review:copy>

<review:file path="scripts/tests/test_review_complexity.py"><review:paste select="#review-metric-structural-witnesses"/><review:paste select="#review-metric-inventory-witnesses"/></review:file>

## Editor design: make the reason inspectable

The report is an analysis artifact. No editor decoration is installed yet.
Findings include output ranges plus exact owning Markdown ranges and fragment IDs.
This allows a future editor consumer to decorate canonical literate source and
open other occurrences without treating generated adapters as manual review work.

Use a small gutter marker beside a repeated body. On hover, say “Repeated shape:
5 bodies in 2 files,” show the normalized-rule version, and offer “Show occurrences.”
A click opens an inline list with names, owning pages and links. Avoid a red error
marker: repetition is a candidate for review, not a correctness failure.

At a section header, show the separate measures: “12 typed declarations · 0
branch points” or “8 branch points · depth 3.” For a baseline comparison, show
repeat and declaration deltas alongside the shared generator's complexity. A
single green/red quality score would conceal the tradeoff this tool exists to show.

Each file has a content digest; exact source documents have digests too. Each
snapshot has a digest over its result, providing an identity for scope-wide
findings. Recompute from a document revision and discard results if that revision
changes. Check the source digest before decorating Markdown; output digests alone
cannot detect prose edits that shift source positions.
Base comparisons and clone groups must share one scope, parser and threshold.
When mapping is unavailable, show a file-level hint explaining why; do not guess
line decorations. Offer filters for implementation, tests and generated output,
with generated output labeled excluded. This is a design proposal; no editor
latency, comprehension benefit or automatic policy equivalence has been measured.

</review:doc>
