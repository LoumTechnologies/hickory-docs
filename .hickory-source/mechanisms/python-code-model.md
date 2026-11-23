<pycode:doc xmlns:pycode="http://www.hickorydocs.com/1.0">
# Query Python architecture without running project code

For maintainers building repository-owned diagram scripts. This read-only provider
implements issue #120. The script owns graph selection and layout; this provider
returns static navigation facts and explicitly incomplete resolution.

## Contract and limits

One JSON request per line receives one GraphQL response per line. Introspection,
variables and operation names use graphql-core. Invalid requests do not end the
session. There is no mutation schema. Diagnostics go to stderr.

Analyze Python source without importing project modules. Jedi follows lexical
bindings, package imports, aliases and inferred attributes. Multiple candidates
remain ambiguous, absent targets unresolved and non-project targets external.
This is static inference, not a complete Python runtime call graph: monkey
patching, dynamic imports, generated attributes and runtime path modifications
may remain unresolved or incompletely inferred. No application execution is
authorized by this provider.

Locations use project-relative POSIX paths, one-based lines and Unicode code-point
columns, with exclusive ends. Module symbols use a zero-width file-start anchor.
Qualified declaration identities survive unrelated line insertions; repeated
bindings with identical qualified names add source coordinates to distinguish
them. IDs are navigation identities, not immutable revision evidence. Syntax
errors become file diagnostics; hidden directories, virtual environments named
`venv`, node_modules and paths resolving outside the root are excluded.

## Prepare and query

From the repository root, install into an isolated environment and expose its
console script on PATH:

```sh
python3 -m venv .hick-cache/python-model
.hick-cache/python-model/bin/python -m pip install ./code-models/python
export PATH="$PWD/.hick-cache/python-model/bin:$PATH"
hick code-model python path/to/source
hick code-model python path/to/source --query '{ symbols(name: "Worker") { id declarations { file startLine startColumn } } }'
```

The first command prints the schema. Query `sourceIndex` for files, symbols,
references and syntax diagnostics, then `references(symbol: "ID")` for the
uses of a selected declaration. `unresolved` includes ambiguous references.
Reference `symbols` contains project targets only; `external` records excluded
non-project candidates. `isWrite` distinguishes assignment targets from reads;
imports and declaration sites also remain available through symbol declarations.
Run the focused process witnesses after installation:

```sh
PYTHON="$PWD/.hick-cache/python-model/bin/python" node --test code-models/python/source-index.test.mjs
```

The model is a snapshot: restart after editing source. Python 3.10+ is required;
focused evidence below was run with Python 3.12.16 and Node 26.13.0.

## Imports

Use static analysis dependencies; never load application modules into the provider.

<pycode:copy id="python-model-hick_python_model-py-imports">
"""Static Python navigation; project modules are never imported."""
from pathlib import Path
import ast
import hashlib
import io
import tokenize

import jedi


</pycode:copy>

## Class model

Keep one model snapshot across queries. File discovery excludes hidden and dependency trees and resolves symlinks before accepting a project path.

<pycode:copy id="python-model-hick_python_model-py-class-model">
class CodeModel:
    def __init__(self, root):
        self.root = Path(root).resolve(strict=True)
        if not self.root.is_dir():
            raise ValueError(f"not a source directory: {root}")
        self.symbols = {}
        self.files = []
        self.references = []
        self.diagnostics = []
        project = jedi.Project(
            self.root, added_sys_path=[str(self.root), str(self.root / 'src')],
            load_unsafe_extensions=False,
        )
        for path in sorted(self.root.rglob('*.py')):
            rel = path.relative_to(self.root)
            if any(p.startswith('.') or p in {'__pycache__', 'node_modules', 'venv'}
                   for p in rel.parts) or not path.resolve().is_relative_to(self.root):
                continue
            self.index_file(path, rel.as_posix(), project)
        self.references.sort(key=lambda r: (r['span']['file'], r['span']['startLine'],
                                            r['span']['startColumn']))

</pycode:copy>

## Span

Jedi columns count Unicode characters. Convert to the documented one-based navigation convention and reject external destinations.

<pycode:copy id="python-model-hick_python_model-py-span">
    def span(self, name):
        path = name.module_path
        if path is None or name.line is None:
            return None
        try:
            file = Path(path).resolve().relative_to(self.root).as_posix()
        except ValueError:
            return None
        return dict(file=file, startLine=name.line, startColumn=name.column + 1,
                    endLine=name.line, endColumn=name.column + len(name.name) + 1)

</pycode:copy>

## Symbol

Preserve qualified binding identities across unrelated line movement. Repeated bindings must remain separate instead of overwriting the first declaration.

<pycode:copy id="python-model-hick_python_model-py-symbol">
    def symbol(self, name):
        span = self.span(name)
        if span is None:
            return None
        # Qualified names survive unrelated line insertions. Repeated bindings
        # keep separate declaration identities rather than overwriting evidence.
        parent = name.parent()
        qualified = name.full_name or f"{parent.full_name or parent.name}.{name.name}"
        if name.type == 'module' and name.full_name == name.module_name:
            span = dict(file=span['file'], startLine=1, startColumn=1, endLine=1, endColumn=1)
        identity = f"python:{span['file']}:{qualified}"
        existing = self.symbols.get(identity)
        if existing and existing['declarations'][0] != span:
            identity += f":{name.line}:{name.column}"
        self.symbols.setdefault(identity, dict(
            id=identity, name=name.name, qualifiedName=qualified, kind=name.type,
            declarations=[span],
        ))
        return identity

</pycode:copy>

## Index file

Read each file once so its digest describes the analyzed bytes. Parse first so invalid Python becomes visible diagnostics. Resolve uses through Jedi, retain every candidate, and attribute writes from AST contexts instead of guessing from spelling. AST byte offsets are converted before comparing them with Jedi character columns.

<pycode:copy id="python-model-hick_python_model-py-index-file">
    def index_file(self, path, relative, project):
        try:
            raw = path.read_bytes()
            encoding, _ = tokenize.detect_encoding(io.BytesIO(raw).readline)
            source = raw.decode(encoding)
            tree = ast.parse(source, filename=relative)
        except (SyntaxError, UnicodeError, OSError) as error:
            self.diagnostics.append(dict(file=relative, message=str(error)))
            return
        self.files.append(dict(path=relative, sha256=hashlib.sha256(raw).hexdigest()))
        scopes = [n for n in ast.walk(tree)
                  if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef))]
        script = jedi.Script(source, path=path, project=project)
        for name in script.get_names(all_scopes=True, definitions=True, references=False):
            self.symbol(name)
        lines = source.splitlines()
        writes = set()
        for node in ast.walk(tree):
            if isinstance(node, (ast.Name, ast.Attribute)) and isinstance(node.ctx, ast.Store):
                line = lines[node.lineno - 1]
                column = len(line.encode('utf-8')[:node.col_offset].decode('utf-8'))
                if isinstance(node, ast.Attribute):
                    end = len(lines[node.end_lineno - 1].encode('utf-8')[:node.end_col_offset].decode('utf-8'))
                    writes.add((node.end_lineno, end - len(node.attr)))
                else:
                    writes.add((node.lineno, column))
        for name in script.get_names(all_scopes=True, definitions=True, references=True):
            is_write = (name.line, name.column) in writes
            if name.is_definition() and not is_write:
                continue
            targets = name.goto(follow_imports=True, follow_builtin_imports=False)
            ids = sorted({identity for target in targets
                          if (identity := self.symbol(target)) is not None})
            external = any(self.span(t) is None for t in targets)
            # More than one inferred target is explicitly ambiguous, even when
            # only one of those targets belongs to this project.
            status = ('unresolved' if not targets else 'ambiguous' if len(targets) > 1
                      else 'external' if external else 'resolved')
            parents = [n for n in scopes if n.lineno <= name.line <= n.end_lineno]
            owner = min(parents, key=lambda n: n.end_lineno - n.lineno).name if parents else ''
            self.references.append(dict(
                name=name.name, symbols=ids, status=status, external=external,
                span=self.span(name), fromModule=relative, fromDeclaration=owner,
                isWrite=is_write,
            ))
        # Imports bind local definitions too; following those definitions exposes
        # missing imports even if no later expression uses the alias.
        for name in script.get_names(all_scopes=True, definitions=True, references=False):
            if name.is_definition() and name.type == 'module':
                targets = name.goto(follow_imports=True)
                if not targets:
                    self.references.append(dict(
                        name=name.name, symbols=[], status='unresolved', external=False,
                        span=self.span(name), fromModule=relative, fromDeclaration='', isWrite=False,
                    ))

</pycode:copy>

## Info

Describe the analyzed snapshot without exposing environment-dependent absolute paths in declaration targets.

<pycode:copy id="python-model-hick_python_model-py-info">
    def info(self):
        return dict(name='hick-model-python', version='1', language='python',
                    root=str(self.root), fileCount=len(self.files))

</pycode:copy>

## Unresolved

Publish uncertainty as queryable data; generators must not mistake missing edges for a complete architecture.

<pycode:copy id="python-model-hick_python_model-py-unresolved">
    def unresolved(self):
        return [dict(message=f"{r['status']}: {r['name']}", at=r['span'])
                for r in self.references if r['status'] in {'unresolved', 'ambiguous'}]
</pycode:copy>

<pycode:file path="code-models/python/hick_python_model.py"><pycode:paste select="#python-model-hick_python_model-py-imports"/><pycode:paste select="#python-model-hick_python_model-py-class-model"/><pycode:paste select="#python-model-hick_python_model-py-span"/><pycode:paste select="#python-model-hick_python_model-py-symbol"/><pycode:paste select="#python-model-hick_python_model-py-index-file"/><pycode:paste select="#python-model-hick_python_model-py-info"/><pycode:paste select="#python-model-hick_python_model-py-unresolved"/></pycode:file>

## Protocol imports

The provider and GraphQL engine are ordinary installed dependencies; application code remains input text.

<pycode:copy id="python-model-hick_python_server-py-protocol-imports">
"""One GraphQL response per input line, with diagnostics on stderr."""
import json
import sys

from graphql import build_schema, graphql_sync
from hick_python_model import CodeModel


</pycode:copy>

## Schema

Keep the Python schema small and explicit. References carry candidate IDs and status, so a consumer can render ambiguity without inventing one destination.

<pycode:copy id="python-model-hick_python_server-py-schema">
SCHEMA = '''
"""One-based lines and Unicode code-point columns; end is exclusive."""
type Span {
  file: String!, startLine: Int!, startColumn: Int!, endLine: Int!, endColumn: Int!
}
type ServerInfo {
  name: String!, version: String!, language: String!, root: String!, fileCount: Int!
}
type UnresolvedReference { message: String!, at: Span }
type PythonSymbol {
  id: String!, name: String!, qualifiedName: String!, kind: String!, declarations: [Span!]!
}
type PythonFile { path: String!, sha256: String! }
type PythonDiagnostic { file: String!, message: String! }
type PythonReference {
  name: String!, symbols: [String!]!, status: String!, external: Boolean!, span: Span!,
  fromModule: String!, fromDeclaration: String!, isWrite: Boolean!
}
type PythonSourceIndex {
  version: Int!, coordinateEncoding: String!, files: [PythonFile!]!,
  symbols: [PythonSymbol!]!, references: [PythonReference!]!, diagnostics: [PythonDiagnostic!]!
}
type Query {
  version: ServerInfo!, sourceIndex: PythonSourceIndex!, unresolved: [UnresolvedReference!]!,
  symbols(name: String): [PythonSymbol!]!, references(symbol: String!): [PythonReference!]!
}
'''


</pycode:copy>

## Resolvers

Share the same cached facts between bulk queries and per-symbol navigation; standard GraphQL performs validation and introspection.

<pycode:copy id="python-model-hick_python_server-py-resolvers">
def schema_for(model):
    schema = build_schema(SCHEMA)
    fields = schema.query_type.fields
    fields['version'].resolve = lambda *_: model.info()
    fields['unresolved'].resolve = lambda *_: model.unresolved()
    fields['sourceIndex'].resolve = lambda *_: dict(
        version=1, coordinateEncoding='unicode-code-points-1-based', files=model.files,
        symbols=sorted(model.symbols.values(), key=lambda s: s['id']),
        references=model.references, diagnostics=model.diagnostics,
    )
    fields['symbols'].resolve = lambda _r, _i, name=None: [
        s for s in sorted(model.symbols.values(), key=lambda s: s['id'])
        if name is None or s['name'] == name]
    fields['references'].resolve = lambda _r, _i, symbol: [
        r for r in model.references if symbol in r['symbols']]
    return schema


</pycode:copy>

## Serve

Invalid roots fail with a repair instruction. Frame errors inside one response and continue reading, keeping diagnostic text away from stdout.

<pycode:copy id="python-model-hick_python_server-py-serve">
def main():
    try:
        model = CodeModel(sys.argv[1] if len(sys.argv) > 1 else '.')
        schema = schema_for(model)
    except Exception as error:
        print(f'hick-model-python: {error}. Pass a readable Python source directory.', file=sys.stderr)
        return 2
    print(f"hick-model-python: {len(model.files)} file(s); ready", file=sys.stderr)
    for diagnostic in model.diagnostics:
        print(f"{diagnostic['file']}: {diagnostic['message']}", file=sys.stderr)
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            request = json.loads(line)
            result = graphql_sync(schema, request.get('query', ''),
                                  variable_values=request.get('variables'),
                                  operation_name=request.get('operationName'))
            answer = result.formatted
        except Exception as error:
            answer = dict(errors=[dict(message=str(error))])
        print(json.dumps(answer, ensure_ascii=True), flush=True)
    return 0


</pycode:copy>

## Entrypoint

Return startup failures as a nonzero process status.

<pycode:copy id="python-model-hick_python_server-py-entrypoint">
if __name__ == '__main__':
    sys.exit(main())
</pycode:copy>

<pycode:file path="code-models/python/hick_python_server.py"><pycode:paste select="#python-model-hick_python_server-py-protocol-imports"/><pycode:paste select="#python-model-hick_python_server-py-schema"/><pycode:paste select="#python-model-hick_python_server-py-resolvers"/><pycode:paste select="#python-model-hick_python_server-py-serve"/><pycode:paste select="#python-model-hick_python_server-py-entrypoint"/></pycode:file>

## Process tests

Drive the real line protocol from Node rather than reaching into Python internals. These focused witnesses cover package alias navigation, explicit uncertainty, Unicode destinations, invalid requests and recovery, introspection and the absence of application execution.

### Process harness

Keep process framing in a small fixture helper. Each case gets its own temporary project and tears it down after observing replies.

<pycode:copy id="python-model-test-process-harness">
// Protects the Python navigation contract in python-code-model.md.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const provider = dirname(fileURLToPath(import.meta.url));
const query = `{ sourceIndex { files { path } symbols { id name declarations { file startLine startColumn endColumn } }
 references { name status symbols span { file startLine startColumn endColumn } fromDeclaration isWrite external }
 diagnostics { file message } } unresolved { message } }`;

function fixture(files, run) {
  const root = mkdtempSync(join(tmpdir(), 'hick-python-model-'));
  try {
    for (const [path, text] of Object.entries(files)) {
      mkdirSync(dirname(join(root, path)), { recursive: true });
      writeFileSync(join(root, path), text);
    }
    const ask = (requests) => {
      const result = spawnSync(process.env.PYTHON ?? 'python3', [join(provider, 'hick_python_server.py'), root], {
        input: requests.map(r => typeof r === 'string' ? r : JSON.stringify(r)).join('\n') + '\n', encoding: 'utf8',
      });
      assert.equal(result.status, 0, result.stderr);
      assert.match(result.stderr, /ready/);
      return result.stdout.trim().split('\n').map(JSON.parse);
    };
    run(ask, root);
  } finally { rmSync(root, { recursive: true, force: true }); }
}

</pycode:copy>

### Package resolution

Re-exports and module aliases must lead to the original class and method declaration, with exact file, line and column destinations. Writes retain their assignment role.

<pycode:copy id="python-model-test-package-resolution">
test('package reexports, relative imports, module aliases and source destinations', () => fixture({
  'pkg/__init__.py': 'from .core import Worker as Alias\n',
  'pkg/core.py': 'class Worker:\n    def run(self):\n        return 1\n',
  'use.py': 'from pkg import Alias\nimport pkg.core as c\nx = Alias()\nx.run()\nc.Worker()\n',
}, ask => {
  const [answer] = ask([{ query }]);
  assert.equal(answer.errors, undefined);
  const index = answer.data.sourceIndex;
  const worker = index.symbols.find(s => s.name === 'Worker' && s.declarations[0].file === 'pkg/core.py');
  assert.deepEqual(worker.declarations[0], { file: 'pkg/core.py', startLine: 1, startColumn: 7, endColumn: 13 });
  for (const [line, name] of [[3, 'Alias'], [5, 'Worker']]) {
    const ref = index.references.find(r => r.span.file === 'use.py' && r.span.startLine === line && r.name === name);
    assert.equal(ref.status, 'resolved');
    assert.deepEqual(ref.symbols, [worker.id]);
  }
  const method = index.references.find(r => r.name === 'run');
  const [references] = ask([{ query: 'query($s: String!) { references(symbol: $s) { name span { file startLine } } }', variables: { s: method.symbols[0] } }]);
  assert.deepEqual(references.data.references, [{ name: 'run', span: { file: 'use.py', startLine: 4 } }]);
  assert(index.references.some(r => r.name === 'x' && r.isWrite));
  assert.deepEqual(ask([{ query }])[0], answer);
}));

</pycode:copy>

### Uncertainty

Two possible receiver types must retain both method candidates. Missing imports and names stay unresolved; syntax diagnostics and Unicode columns remain inspectable.

<pycode:copy id="python-model-test-uncertainty">
test('ambiguity, missing imports, syntax diagnostics and Unicode coordinates stay explicit', () => fixture({
  'sample.py': 'class A:\n    def run(self): pass\nclass B:\n    def run(self): pass\nx = A() if unknown else B()\nx.run()\nfrom absent import missing as unused_alias\nπ = 1; absent_name()\n',
  'broken.py': 'def broken(:\n',
}, ask => {
  const [answer] = ask([{ query }]);
  const index = answer.data.sourceIndex;
  const ambiguous = index.references.find(r => r.name === 'run');
  assert.equal(ambiguous.status, 'ambiguous');
  assert.equal(ambiguous.symbols.length, 2);
  assert(index.references.some(r => r.name === 'unused_alias' && r.status === 'unresolved'));
  const missing = index.references.find(r => r.name === 'absent_name');
  assert.equal(missing.status, 'unresolved');
  assert.equal(missing.span.startColumn, 8);
  assert(index.diagnostics.some(d => d.file === 'broken.py'));
  assert(answer.data.unresolved.some(r => /ambiguous/.test(r.message)));
}));

</pycode:copy>

### Protocol safety

Malformed JSON and invalid GraphQL cannot poison the next request. Introspection describes a query-only schema, and a sentinel checks that analyzing an application never runs its top-level effects.

<pycode:copy id="python-model-test-protocol-safety">
test('introspection, variables, invalid requests and recovery; never executes analyzed modules', () => fixture({
  'danger.py': 'from pathlib import Path\nPath(__file__).with_suffix(".executed").write_text("bad")\nraise RuntimeError("must not execute")\n',
}, (ask, root) => {
  const replies = ask(['{broken', { query: '{ absentField }' }, { query: '{ __schema { queryType { name } mutationType { name } } }' }, { query }]);
  assert(replies[0].errors);
  assert(replies[1].errors);
  assert.deepEqual(replies[2].data.__schema, { queryType: { name: 'Query' }, mutationType: null });
  assert.equal(replies[3].errors, undefined);
  assert.equal(existsSync(join(root, 'danger.executed')), false);
}));

</pycode:copy>

### Stable identities

Namespace packages under src are valid import boundaries. Inserting blank lines changes the destination line without changing a qualified declaration ID.

<pycode:copy id="python-model-test-stable-identities">
test('src-layout namespace packages resolve and declaration IDs survive unrelated inserted lines', () => fixture({
  'src/space/core.py': 'def work():\n    return 1\n',
  'client.py': 'from space.core import work as alias\ndef caller():\n    return alias()\n',
}, (ask, root) => {
  const [before] = ask([{ query }]);
  const symbol = before.data.sourceIndex.symbols.find(s => s.name === 'work');
  const ref = before.data.sourceIndex.references.find(r => r.name === 'alias');
  assert.deepEqual(ref.symbols, [symbol.id]);
  assert.equal(ref.fromDeclaration, 'caller');
  writeFileSync(join(root, 'src/space/core.py'), '\n\ndef work():\n    return 1\n');
  const [after] = ask([{ query }]);
  const updated = after.data.sourceIndex.symbols.find(s => s.name === 'work');
  assert.equal(updated.id, symbol.id);
  assert.equal(updated.declarations[0].startLine, 3);
}));
</pycode:copy>

<pycode:file path="code-models/python/source-index.test.mjs"><pycode:paste select="#python-model-test-process-harness"/><pycode:paste select="#python-model-test-package-resolution"/><pycode:paste select="#python-model-test-uncertainty"/><pycode:paste select="#python-model-test-protocol-safety"/><pycode:paste select="#python-model-test-stable-identities"/></pycode:file>

## Installation

Install a normal console script so PATH and project-local discovery use the existing code-model mechanism. Pin analysis and GraphQL versions to make the observed semantics reproducible.

<pycode:copy id="python-model-pyproject-toml-installation">
[build-system]
requires = ["setuptools>=68"]
build-backend = "setuptools.build_meta"

[project]
name = "hick-model-python"
version = "0.1.0"
requires-python = ">=3.10"
dependencies = ["jedi==0.19.2", "parso==0.8.5", "graphql-core==3.2.7"]

[project.scripts]
hick-model-python = "hick_python_server:main"

[tool.setuptools]
py-modules = ["hick_python_model", "hick_python_server"]
</pycode:copy>

<pycode:file path="code-models/python/pyproject.toml"><pycode:paste select="#python-model-pyproject-toml-installation"/></pycode:file>

## Keep installation artifacts local

Building the console-script package creates wheel metadata beside its source.
Ignore those disposable products so installation does not suggest source edits.

<pycode:entry path="code-models/python/.gitignore">
*.egg-info/
build/
dist/
.venv/
</pycode:entry>

## Focused verification — November 23, 2026

The four Node process witnesses passed with Python 3.12.16, Jedi 0.19.2,
parso 0.8.5, graphql-core 3.2.7 and Node 26.13.0. Installation into an isolated
Python environment produced a working `hick-model-python` console script. The
actual `hick code-model python` path returned the Worker declaration from a
package fixture and printed the introspected schema without a query. The focused
Rust registry/cache-discovery witness passed for Python and C#. Reconstruction
check and ownership audit passed; these do not prove complete Python inference.
Broad regressions and post-commit checks were not monitored.

</pycode:doc>
