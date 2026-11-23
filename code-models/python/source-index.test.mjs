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
