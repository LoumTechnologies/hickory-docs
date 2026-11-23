"""Static Python navigation; project modules are never imported."""
from pathlib import Path
import ast
import hashlib
import io
import tokenize

import jedi


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

    def info(self):
        return dict(name='hick-model-python', version='1', language='python',
                    root=str(self.root), fileCount=len(self.files))

    def unresolved(self):
        return [dict(message=f"{r['status']}: {r['name']}", at=r['span'])
                for r in self.references if r['status'] in {'unresolved', 'ambiguous'}]
