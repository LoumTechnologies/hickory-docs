<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Freeze language-specific code model graphs

A code model is richer than an LSP hover and its schema belongs to its language. Introspect each explicitly selected installed server, then select its parameter-free readable fields to a finite depth. Keep concrete interface variants with `__typename`; do not flatten Go, TypeScript and C# into a universal type model. Required-argument operations remain outside this snapshot, and cycles stop at the stated depth.

<static:copy id="static-scripts-capture-source-models-py-0">
"""Freeze each installed code model's own schema and readable type graph."""
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile

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
            if field['name'].startswith('__'):
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

</static:copy>

Run model queries against only the committed code and manifests declared by the construction inventory. These inputs are private build scratch; only the schema, finite query, answer and unresolved binding evidence are published. A malformed, failed or timed-out provider aborts the build. This freezes knowledge from one revision and never executes document cells or provides a hosted GraphQL service.

<static:copy id="static-scripts-capture-source-models-py-1">

def capture_models(root, revision, constructors, commands):
    # Only generated code and project manifests declared by this same committed
    # source graph become compiler inputs. No dirty files, symlinks or .env.
    selected = [path for path in constructors if Path(path).suffix in ['.ts', '.tsx', '.js', '.jsx', '.go', '.cs', '.csproj', '.sln'] or Path(path).name in ['package.json', 'tsconfig.json', 'go.mod', 'go.sum']]
    if not selected:
        raise ValueError('No selected committed model inputs')
    with tempfile.TemporaryDirectory(prefix='hickory-static-model-') as temporary:
        destination = Path(temporary)
        archive = subprocess.check_output(['git', 'archive', revision, '--', *selected], cwd=root)
        with tarfile.open(fileobj=io.BytesIO(archive)) as files:
            for item in files:
                if not item.isfile() or item.name not in selected:
                    continue
                target = destination / item.name
                if target.is_absolute() and not target.is_relative_to(destination) or '..' in Path(item.name).parts:
                    raise ValueError('Unsafe code model input path')
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(files.extractfile(item).read())
        results = []
        for language, command in commands.items():
            def query(text):
                response = subprocess.run([*command, str(destination)], cwd=destination,
                    input=json.dumps(dict(query=text))+'\n', text=True, capture_output=True, timeout=120, check=True)
                lines = response.stdout.splitlines()
                if len(lines) != 1 or len(response.stdout) > 32*1024*1024:
                    raise ValueError('Unexpected code model response')
                result = json.loads(lines[0])
                if result.get('errors'):
                    details = [dict(message=error.get('message'), path=error.get('path')) for error in result['errors'][:3]]
                    raise ValueError('Code model refused its static selection: '+json.dumps(details))
                return result
            schema = query(SCHEMA)
            query_text = graph_query(schema['data']['__schema'])
            graph = query(query_text)
            # Preserve language-specific fields and unresolved binding evidence.
            encoded = json.dumps(dict(language=language, revision=revision, schema=schema, query=query_text, graph=graph)).replace(str(destination), '.')
            results.append(json.loads(encoded))
        return results
</static:copy>

<static:file path="scripts/capture-source-models.py"><static:paste select="#static-scripts-capture-source-models-py-0"/><static:paste select="#static-scripts-capture-source-models-py-1"/></static:file>
</static:doc>
