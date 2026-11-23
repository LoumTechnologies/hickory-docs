"""One GraphQL response per input line, with diagnostics on stderr."""
import json
import sys

from graphql import build_schema, graphql_sync
from hick_python_model import CodeModel


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


if __name__ == '__main__':
    sys.exit(main())
