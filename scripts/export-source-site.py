"""Publish committed maintained Markdown for the static source browser."""
import argparse
import hashlib
import io
import importlib.util
import shutil
import json
from pathlib import Path
import re
import subprocess
import tarfile


def export(root, destination, revision='HEAD', *, compile_language=False, lsp_command=None, model_commands=None):
    # Archive the committed allowlist, never local sessions, credentials or outputs.
    revision = subprocess.check_output(['git', 'rev-parse', '--verify', '--end-of-options', revision + '^{commit}'], cwd=root, text=True).strip()
    archive = subprocess.check_output(['git', 'archive', revision, '--', 'project.md', '.hickory-source'], cwd=root)
    documents = []
    payloads = []
    models = []
    destination = Path(destination)
    (destination / 'documents').mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(archive)) as files:
        for item in files:
            if not item.isfile() or not item.name.endswith('.md'):
                continue
            if item.name != 'project.md' and not item.name.startswith('.hickory-source/'):
                raise ValueError('Source outside the maintained Markdown allowlist')
            source = files.extractfile(item).read().decode('utf-8')
            title = re.search(r'^# (.+)', source, re.M)
            identity = hashlib.sha256(item.name.encode()).hexdigest()
            digest = hashlib.sha256(source.encode()).hexdigest()
            url = f'documents/{identity}.json'
            payloads.append(dict(path=item.name, revision=revision, source=source))
            documents.append(dict(path=item.name, title=title[1] if title else item.name, sha256=digest, url=url))
    if not any(d['path'] == 'project.md' for d in documents):
        raise ValueError('Committed source has no project.md')
    if compile_language:
        # The routing metadata must describe the same commit as the source.
        constructors = json.loads(subprocess.check_output(['git', 'show', f'{revision}:hickory-dogfood/behavior-map.json'], cwd=root))['constructors']
        compiler = Path(__file__).with_name('compile-source-highlights.mjs')
        compiled = subprocess.run(['node', str(compiler)], cwd=Path(__file__).resolve().parent.parent,
            input=json.dumps(dict(documents=payloads, constructors=constructors)), text=True, capture_output=True, check=True)
        plans = json.loads(compiled.stdout)
        captures = {}
        if lsp_command:
            spec = importlib.util.spec_from_file_location('source_language', Path(__file__).with_name('capture-source-language.py'))
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            captures = module.capture(payloads, plans, lsp_command)
        for payload, document in zip(payloads, documents):
            intelligence = dict(version=1, source_hash=document['sha256'], spans=plans[payload['path']]['spans'],
                language=captures.get(payload['path']), state='captured' if captures.get(payload['path'], {}).get('provider_count', 0) else 'unavailable')
            encoded = json.dumps(intelligence, ensure_ascii=False, separators=(',', ':'))
            payload['intelligence'] = encoded
            document['intelligence_sha256'] = hashlib.sha256(encoded.encode()).hexdigest()
    if model_commands:
        constructors = json.loads(subprocess.check_output(['git', 'show', f'{revision}:hickory-dogfood/behavior-map.json'], cwd=root))['constructors']
        spec = importlib.util.spec_from_file_location('source_models', Path(__file__).with_name('capture-source-models.py'))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        (destination / 'models').mkdir(exist_ok=True)
        for model in module.capture_models(root, revision, constructors, model_commands):
            encoded = json.dumps(model, ensure_ascii=False, separators=(',', ':'))
            digest = hashlib.sha256(encoded.encode()).hexdigest()
            url = f'models/{digest}.json'
            (destination / url).write_text(encoded)
            models.append(dict(language=model['language'], sha256=digest, url=url))
    for payload, document in zip(payloads, documents):
        (destination / document['url']).write_text(json.dumps(payload, ensure_ascii=False) + '\n')
    manifest = dict(version=1, revision=revision, entry='project.md', documents=sorted(documents, key=lambda d: d['path']))
    if models:
        manifest['models'] = models
    (destination / 'index.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--revision', default='HEAD')
    parser.add_argument('--lsp', type=Path, help='Capture read-only answers from this installed stdio language server')
    parser.add_argument('--model', action='append', default=[], metavar='LANGUAGE=EXECUTABLE', help='Capture an installed line-protocol code model')
    args = parser.parse_args()
    models = {}
    for selection in args.model:
        language, executable = selection.split('=', 1)
        resolved = str(Path(shutil.which(executable) or executable).resolve(strict=True))
        models[language] = [resolved]
    result = export(Path.cwd(), args.output, args.revision, compile_language=True,
        lsp_command=[str(args.lsp.resolve())] if args.lsp else None, model_commands=models)
    print(f"Published {len(result['documents'])} Markdown sources at {result['revision']}")


if __name__ == '__main__':
    main()
