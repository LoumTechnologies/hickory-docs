#!/usr/bin/env python3
"""Publish only explicitly selected, reviewed material using the existing CLI/viewer.

No globbing, sessions, credentials or implicit assets are copied. Run in CI or a
clean checkout. Each command runs in a temporary copy of selected inputs.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def selected(root, relative):
    if not isinstance(relative, str) or Path(relative).is_absolute() or '..' in Path(relative).parts:
        raise ValueError('Publication inputs must be workspace-relative paths without parent traversal')
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()) or not path.is_file():
        raise ValueError(f'Publication input is unavailable or outside root: {relative}')
    if any(part in ('sessions', '.git', '.hick-cache', '.aws', '.ssh') or part.startswith('.env') for part in Path(relative).parts):
        raise ValueError(f'Private/session input is not publishable: {relative}')
    return path


def publish(root, selection, destination, hick, runner=subprocess.run):
    if selection.get('version') != 1:
        raise ValueError('Unsupported publication selection')
    revision = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
    runner_hash = None
    if hick.is_file():
        hasher = hashlib.sha256()
        with hick.open('rb') as binary:
            for chunk in iter(lambda: binary.read(1024 * 1024), b''):
                hasher.update(chunk)
        runner_hash = hasher.hexdigest()
    index = dict(version=1, revision=revision, documents=[])
    destination.mkdir(parents=True, exist_ok=True)
    failed = False
    for entry in selection['documents']:
        if entry.get('reviewed_output') is not True:
            raise ValueError('Each published document/output must be explicitly reviewed')
        path = entry['path']
        source = selected(root, path).read_bytes()
        inputs = {path: source}
        for p in [*entry.get('dependencies', []), *entry.get('assets', [])]:
            inputs[p] = selected(root, p).read_bytes()
        if any(entry.get('reviewed_inputs', {}).get(p) != digest(data) for p, data in inputs.items()):
            raise ValueError('Selected input changed since publication review; review bytes and update reviewed_inputs before publishing')
        dependencies = {p: digest(inputs[p]) for p in entry.get('dependencies', [])}
        assets = {p: digest(inputs[p]) for p in entry.get('assets', [])}
        identity = digest(path.encode())[:16]
        with tempfile.TemporaryDirectory(prefix='hickory-publication-') as temporary:
            scratch = Path(temporary)
            for p in [path, *dependencies, *assets]:
                target = scratch / p
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(inputs[p])
            # These selected examples run locally; no inherited keys/provider config.
            environment = {'PATH': os.environ.get('PATH', ''), 'HOME': str(scratch / 'home'),
                           'HICKORY_EXECUTOR': 'local', 'HICKORY_ENGINE_WORKER': '1', 'LANG': 'C.UTF-8'}
            commands = [[str(hick), 'weave', path], [str(hick), 'test', path, '--json']]
            results = []
            blocks = []
            execution_evidence = None
            status = 'not_run'
            try:
                for command in commands:
                    result = runner(command, cwd=scratch, env=environment, capture_output=True, text=True, timeout=180)
                    results.append(dict(command=command, exit_code=result.returncode,
                                        stdout=result.stdout, stderr=result.stderr))
                    if command[1] == 'weave' and result.returncode:
                        status = 'unavailable'
                        break
                    if command[1] == 'test':
                        status = {0: 'passed', 1: 'stale', 2: 'unavailable', 3: 'failed'}.get(result.returncode, 'failed')
                        try:
                            model = json.loads(result.stdout)
                            model = model[0] if isinstance(model, list) else model
                            blocks = model['blocks']
                            execution_evidence = model.get('execution_evidence')
                        except (ValueError, KeyError, IndexError, TypeError):
                            if result.returncode == 0:
                                status = 'unavailable'
            except (OSError, subprocess.TimeoutExpired) as error:
                status = 'unavailable'
                results.append(dict(command=commands[len(results)], error=str(error)))
            cells = [b for b in blocks if b.get('kind') == 'exec']
            if status == 'passed':
                if not cells:
                    status = 'not_run'
                elif any(not isinstance(i, int) or i < 0 or i >= len(cells) or not cells[i].get('expect') for r in entry.get('requirements', []) for i in r['witnesses']):
                    status = 'unavailable'
            failed |= status != 'passed'
            text = source.decode('utf-8')
            snapshot = dict(format='hickory-viewer-snapshot', version=1, path=path,
                source=text, source_hash=digest(source), captured_at_ms=int(datetime.now(timezone.utc).timestamp()*1000),
                execution='CI verification command executed; see exact command/outcome. Cell playback origin is unavailable unless separately recorded.',
                cells=cells,
                verification=dict(status=status, revision=revision, source_hash=digest(source), dependencies=dependencies,
                    assets=assets, execution_evidence=execution_evidence, environment=dict(platform=platform.platform(), executor='local', runner_sha256=runner_hash, inherited_credentials=False),
                    commands=results, requirements=entry.get('requirements', []),
                    origin='executed-command; per-cell recorded/replayed origin unavailable',
                    scope='Only listed witnesses; deterministic allocation is not real network timing or arbitrary prose verification.'))
            # Asset references remain relative to the original document path.
            # Host an isolated publication subtree and use a matching snapshot path.
            artifact = destination / identity
            artifact.mkdir(exist_ok=True)
            for p in assets:
                output = artifact / p
                output.parent.mkdir(parents=True, exist_ok=True)
                output.write_bytes(inputs[p])
            filename = f'{identity}/snapshot.json'
            (destination / filename).write_text(json.dumps(snapshot, indent=2) + '\n')
            index['documents'].append(dict(path=path, status=status, snapshot=filename, revision=revision,
                                            source_hash=digest(source)))
    (destination / 'index.json').write_text(json.dumps(index, indent=2) + '\n')
    return not failed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--selection', type=Path, default=Path('publications/selected.json'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--hick', type=Path, required=True)
    args = parser.parse_args()
    root = Path.cwd().resolve()
    success = publish(root, json.loads(args.selection.read_text()), args.output, args.hick.resolve())
    raise SystemExit(0 if success else 1)


if __name__ == '__main__':
    main()
