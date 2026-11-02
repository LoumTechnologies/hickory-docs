"""Freeze each installed code model's own schema and readable type graph."""
import io
import json
import sys
from pathlib import Path
import subprocess
import tarfile
import tempfile
import queue
import threading
import signal
import os
import hashlib
from collections import deque

TYPE = 'kind name ofType { kind name ofType { kind name ofType { kind name } } }'
SCHEMA = '{ __schema { queryType { name } types { kind name fields { name args { name defaultValue type { '+TYPE+' } } type { '+TYPE+' } } possibleTypes { name } } } }'


def graph_query(schema):
    types = {item['name']: item for item in schema['types']}
    def underlying(value):
        while value.get('ofType'):
            value = value['ofType']
        return value
    def selection(name, depth=0, seen=()):
        value = types[name]
        if value['kind'] in ['SCALAR', 'ENUM']:
            return ''
        fields = ['__typename']
        for field in value.get('fields') or []:
            if field['name'].startswith('__') or field['name'] == 'sourceIndex':
                continue
            if any(arg['type']['kind'] == 'NON_NULL' and arg.get('defaultValue') is None for arg in field['args']):
                continue  # A static graph cannot invent parameter values.
            target = underlying(field['type'])
            if target['kind'] in ['SCALAR', 'ENUM']:
                fields.append(field['name'])
            elif depth < 4 and target['name'] not in seen:
                fields.append(field['name'] + selection(target['name'], depth+1, (*seen, name)))
        for variant in value.get('possibleTypes') or []:
            if variant['name'] not in seen and depth < 4:
                fields.append('... on '+variant['name']+' '+selection(variant['name'], depth+1, (*seen, name)))
        return '{ '+' '.join(fields)+' }'
    return selection(schema['queryType']['name'])


INDEX_QUERY = """{ sourceIndex {
  version coordinateEncoding
  files { path sha256 length occurrences { from to symbols type isDefinition isWrite unresolved external }
    calls { from to signature } folds { from to kind } classifications
    hints { position text kind whitespaceBefore whitespaceAfter } }
  symbols { id name kind documentation declarations { file from to } }
  diagnostics { file from to code category message }
} }"""


class ModelSession:
    """One bounded line protocol and one compiler session for a full export."""
    def __init__(self, command, root, timeout=120):
        self.process = subprocess.Popen(command, cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, text=True, start_new_session=True)
        self.stderr_tail = deque(maxlen=16)
        def read_stderr():
            for line in self.process.stderr: self.stderr_tail.append(line[:2000])
        self.stderr_reader = threading.Thread(target=read_stderr, daemon=True)
        self.stderr_reader.start()
        self.timeout = timeout
        self.responses = queue.Queue()
        def read():
            try:
                while True:
                    line = self.process.stdout.readline(128 * 1024 * 1024 + 1)
                    if not line:
                        raise EOFError('Code model closed before answering')
                    if len(line.encode()) > 128 * 1024 * 1024 or not line.endswith('\n'):
                        raise ValueError('Code model response exceeds capture bound')
                    self.responses.put(json.loads(line))
            except Exception as error:
                self.responses.put(error)
        self.reader = threading.Thread(target=read, daemon=True)
        self.reader.start()

    def query(self, text, *, timeout=None):
        self.process.stdin.write(json.dumps(dict(query=text)) + '\n')
        self.process.stdin.flush()
        try:
            result = self.responses.get(timeout=self.timeout if timeout is None else timeout)
        except queue.Empty:
            raise TimeoutError('Code model query exceeded capture deadline') from None
        if isinstance(result, Exception):
            if isinstance(result, EOFError):
                self.stderr_reader.join(timeout=0.2)
                raise EOFError(str(result)+'; '+''.join(self.stderr_tail)[-8000:])
            raise result
        if not isinstance(result, dict) or result.get('errors'):
            raise ValueError('Code model refused its static selection: ' + json.dumps(result.get('errors') if isinstance(result, dict) else result)[:2000])
        return result

    def __enter__(self):
        return self

    def __exit__(self, *_error):
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
        self.process.stdin.close()
        self.reader.join(timeout=1)
        self.process.stdout.close()
        self.stderr_reader.join(timeout=1)
        self.process.stderr.close()


def capture_models(root, revision, constructors, commands, *, files=None, install_dependencies=False):
    # Only generated code and project manifests declared by this same committed
    # source graph become compiler inputs. No dirty files, symlinks or .env.
    selected = [path for path in constructors if Path(path).suffix in ['.ts', '.tsx', '.mts', '.cts', '.js', '.jsx', '.mjs', '.cjs', '.go', '.cs', '.csproj', '.sln'] or Path(path).name in ['package.json', 'package-lock.json', 'go.mod', 'go.sum'] or Path(path).name.startswith('tsconfig') and Path(path).suffix == '.json']
    if not selected:
        raise ValueError('No selected committed model inputs')
    with tempfile.TemporaryDirectory(prefix='hickory-static-model-') as temporary:
        destination = Path(temporary)
        if files is None:
            archive = subprocess.check_output(['git', 'archive', revision, '--', *selected], cwd=root)
            with tarfile.open(fileobj=io.BytesIO(archive)) as archive_files:
                inputs = [(item.name, archive_files.extractfile(item).read()) for item in archive_files if item.isfile() and item.name in selected]
        else:
            inputs = [(item['path'], (item['source'].encode('utf-8') if 'source' in item else bytes(item['bytes']))) for item in files if item['path'] in selected]
        for name, data in inputs:
            target = destination / name
            if Path(name).is_absolute() or '..' in Path(name).parts:
                raise ValueError('Unsafe code model input path')
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        dependency_inputs = []
        if install_dependencies and 'typescript' in commands:
            for name, data in inputs:
                if Path(name).name != 'package-lock.json': continue
                subprocess.run(['npm', 'ci', '--ignore-scripts', '--no-fund', '--no-audit'],
                    cwd=destination / Path(name).parent, stdout=subprocess.DEVNULL, check=True, timeout=180)
                dependency_inputs.append(dict(path=name, sha256=hashlib.sha256(data).hexdigest(), scripts=False))
        results = []
        for language, command in commands.items():
            with ModelSession([*command, str(destination)], destination) as session:
                print(f"Static model {language}: reading schema", file=sys.stderr, flush=True)
                schema = session.query(SCHEMA)
                query_text = graph_query(schema['data']['__schema'])
                print(f"Static model {language}: reading declaration graph", file=sys.stderr, flush=True)
                graph = session.query(query_text)
                value = dict(language=language, revision=revision, schema=schema, query=query_text, graph=graph, dependency_inputs=dependency_inputs)
                root_type = schema['data']['__schema']['queryType']['name']
                fields = next(item['fields'] for item in schema['data']['__schema']['types'] if item['name'] == root_type)
                if language == 'typescript' and any(field['name'] == 'sourceIndex' for field in fields):
                    print(f"Static model {language}: indexing source occurrences", file=sys.stderr, flush=True)
                    value['source_index'] = session.query(INDEX_QUERY, timeout=300)['data']['sourceIndex']
                    index = value['source_index']
                    print(f"Static model {language}: {len(index['files'])} files, {sum(len(file['occurrences']) for file in index['files'])} occurrences", file=sys.stderr, flush=True)
                # Preserve native language distinctions; discard ephemeral paths.
                def normalize(item):
                    if isinstance(item, dict):
                        for key in item: item[key] = normalize(item[key])
                    elif isinstance(item, list):
                        for i in range(len(item)): item[i] = normalize(item[i])
                    elif isinstance(item, str):
                        return item.replace(str(destination), '.').replace(destination.as_posix(), '.')
                    return item
                results.append(normalize(value))
        return results
