"""Publish committed maintained Markdown for the static source browser."""
import argparse
import hashlib
import io
import importlib.util
import shutil
import os
import json
import sys
from pathlib import Path
import re
import subprocess
import tarfile
import gzip
from urllib.parse import urlparse


def export(root, destination, revision='HEAD', *, compile_language=False, lsp_command=None, model_commands=None, snapshot=None, compiler=None, compiler_root=None, install_dependencies=False, compress=False):
    # Archive the committed allowlist, never local sessions, credentials or outputs.
    if snapshot is not None:
        if snapshot.get('version') != 1: raise ValueError('Unsupported project snapshot')
        revision, entry, source_files = snapshot['revision'], snapshot['entry'], snapshot['sources']
        frozen_files = snapshot['files']
        constructors = {file['path']: {} for file in frozen_files}
    else:
        revision = subprocess.check_output(['git', 'rev-parse', '--verify', '--end-of-options', revision + '^{commit}'], cwd=root, text=True).strip()
        archive = subprocess.check_output(['git', 'archive', revision, '--', 'project.md', '.hickory-source'], cwd=root)
        source_files = {}
        with tarfile.open(fileobj=io.BytesIO(archive)) as archive_files:
            for item in archive_files:
                if item.isfile() and item.name.endswith('.md'):
                    source_files[item.name] = archive_files.extractfile(item).read().decode('utf-8')
        entry, frozen_files, constructors = 'project.md', None, None
    documents = []
    payloads = []
    models = []
    destination = Path(destination)
    (destination / 'documents').mkdir(parents=True, exist_ok=True)
    for path, source in source_files.items():
        if Path(path).is_absolute() or '..' in Path(path).parts or not path.endswith('.md'):
            raise ValueError('Unsafe declared source path')
        title = re.search(r'^# (.+)', source, re.M)
        identity = hashlib.sha256(path.encode()).hexdigest()
        digest = hashlib.sha256(source.encode()).hexdigest()
        url = f'documents/{identity}.json'
        payloads.append(dict(path=path, revision=revision, source=source))
        documents.append(dict(path=path, title=title[1] if title else path, sha256=digest, url=url))
    if entry not in source_files: raise ValueError('Committed source has no entry document')
    captured_models, bulk = [], {}
    if model_commands:
        if constructors is None:
            constructors = json.loads(subprocess.check_output(['git', 'show', f'{revision}:hickory-dogfood/behavior-map.json'], cwd=root))['constructors']
        spec = importlib.util.spec_from_file_location('source_models', Path(__file__).with_name('capture-source-models.py'))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        captured_models = module.capture_models(root, revision, constructors, model_commands, files=frozen_files, install_dependencies=install_dependencies)
        for model in captured_models:
            source_index = model.pop('source_index', None)
            if source_index is not None and frozen_files is not None:
                spec = importlib.util.spec_from_file_location('source_projection', Path(__file__).with_name('map-source-index.py'))
                projection = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(projection)
                bulk.update(projection.project_index(source_index, payloads, frozen_files))
            del source_index
    if compile_language:
        # The routing metadata must describe the same commit as the source.
        if constructors is None:
            constructors = json.loads(subprocess.check_output(['git', 'show', f'{revision}:hickory-dogfood/behavior-map.json'], cwd=root))['constructors']
        compiler = compiler or Path(__file__).with_name('compile-source-highlights.mjs')
        print(f"Static export: compiling syntax for {len(payloads)} documents", file=sys.stderr, flush=True)
        compiled = subprocess.run(['node', str(compiler)], cwd=compiler_root or Path(__file__).resolve().parent.parent,
            input=json.dumps(dict(documents=payloads, constructors=constructors, files=frozen_files, capturePositions=bool(lsp_command))), text=True, capture_output=True, check=True)
        plans = json.loads(compiled.stdout)
        captures = dict(bulk)
        if lsp_command:
            spec = importlib.util.spec_from_file_location('source_language', Path(__file__).with_name('capture-source-language.py'))
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            pending = [payload for payload in payloads if payload['path'] not in bulk]
            captures.update(module.capture(pending, plans, lsp_command, workers=int(os.environ.get('HICKORY_STATIC_CAPTURE_WORKERS', '1'))))
        for payload, document in zip(payloads, documents):
            spans = plans[payload['path']]['spans']
            classes = list(dict.fromkeys(span['class'] for span in spans))
            class_ids = {name: index for index, name in enumerate(classes)}
            packed = [value for span in spans for value in (span['from'], span['to']-span['from'], class_ids[span['class']])]
            intelligence = dict(version=1, source_hash=document['sha256'], spans=[], packed_spans=dict(classes=classes, data=packed),
                reason=captures.get(payload['path'], {}).get('unavailable_reason'), language=captures.get(payload['path']), state='captured' if captures.get(payload['path'], {}).get('provider_count', 0) else 'unavailable')
            encoded = json.dumps(intelligence, ensure_ascii=False, separators=(',', ':'))
            payload['intelligence'] = encoded
            document['intelligence_sha256'] = hashlib.sha256(encoded.encode()).hexdigest()
    if captured_models:
        (destination / 'models').mkdir(exist_ok=True)
        for model in captured_models:
            encoded = json.dumps(model, ensure_ascii=False, separators=(',', ':'))
            digest = hashlib.sha256(encoded.encode()).hexdigest()
            url = f'models/{digest}.json' + ('.gz' if compress else '')
            if len(encoded.encode()) > 25 * 1024 * 1024:
                raise ValueError(f'Code model {model["language"]} exceeds the host asset limit of 25 MiB')
            (destination / url).write_bytes(gzip.compress(encoded.encode(), compresslevel=6, mtime=0) if compress else encoded.encode())
            models.append(dict(language=model['language'], sha256=digest, url=url))
    for payload, document in zip(payloads, documents):
        encoded = json.dumps(payload, ensure_ascii=False, separators=(',', ':')) + '\n'
        if len(encoded.encode()) > 25 * 1024 * 1024:
            raise ValueError(f'Source {payload["path"]} exceeds the host asset limit of 25 MiB')
        digest = hashlib.sha256(encoded.encode()).hexdigest()
        document['url'] = f'documents/{digest}.json' + ('.gz' if compress else '')
        (destination / document['url']).write_bytes(gzip.compress(encoded.encode(), compresslevel=6, mtime=0) if compress else encoded.encode())
    manifest = dict(version=1, revision=revision, entry=entry, documents=sorted(documents, key=lambda d: d['path']))
    if models:
        manifest['models'] = models
    source_url = snapshot.get('source_url') if snapshot else f'https://github.com/LoumTechnologies/hickory-docs/blob/{revision}/'
    if source_url:
        parsed = urlparse(source_url)
        if parsed.scheme not in ['http', 'https'] or not parsed.hostname or parsed.username or parsed.password or parsed.query or parsed.fragment:
            raise ValueError('Source URL must be an explicit public HTTP repository prefix')
        manifest['source_url'] = source_url.rstrip('/') + '/'
    (destination / 'index.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--revision', default='HEAD')
    parser.add_argument('--snapshot', type=Path)
    parser.add_argument('--install-dependencies', action='store_true')
    parser.add_argument('--compress', action='store_true')
    parser.add_argument('--compiler', type=Path)
    parser.add_argument('--compiler-root', type=Path)
    parser.add_argument('--lsp', type=Path, help='Capture read-only answers from this installed stdio language server')
    parser.add_argument('--model', action='append', default=[], metavar='LANGUAGE=EXECUTABLE', help='Capture an installed line-protocol code model')
    args = parser.parse_args()
    models = {}
    for selection in args.model:
        language, executable = selection.split('=', 1)
        resolved = str(Path(shutil.which(executable) or executable).resolve(strict=True))
        models[language] = [resolved]
    result = export(Path.cwd(), args.output, args.revision, compile_language=True,
        lsp_command=[str(args.lsp.resolve())] if args.lsp else None, model_commands=models,
        snapshot=json.loads(args.snapshot.read_text()) if args.snapshot else None, compiler=args.compiler, compiler_root=args.compiler_root, install_dependencies=args.install_dependencies, compress=args.compress)
    print(f"Published {len(result['documents'])} Markdown sources at {result['revision']}")


if __name__ == '__main__':
    main()
