#!/usr/bin/env python3
"""Reconstruct from Markdown only; check baseline bytes and focused safety controls."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tempfile

BASE = Path(__file__).resolve().parents[1]
ROOT = BASE.parent
ENGINE = ROOT / 'target/dogfood/release/hickory-behavior-engine'


def run(action, root, expected_error=None):
    result = subprocess.run([str(ENGINE), action, str(root)], capture_output=True, text=True)
    if expected_error:
        if not result.returncode or expected_error not in result.stderr:
            raise AssertionError('Expected refusal: ' + result.stdout + result.stderr)
    elif result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result


def inventory(root):
    paths = set()
    for parent, dirs, files in os.walk(root, followlinks=False):
        if Path(parent) == root:
            dirs[:] = [name for name in dirs if name != '.hickory-source']
        for name in list(dirs):
            if (Path(parent) / name).is_symlink():
                dirs.remove(name)
                files.append(name)
        paths.update((Path(parent) / name).relative_to(root).as_posix() for name in files)
    paths.discard('project.md')
    return paths


def check_inventory(root, expected):
    actual = inventory(root)
    if actual != expected:
        raise AssertionError(f'Inventory differs: missing={sorted(expected-actual)}, extra={sorted(actual-expected)}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', action='store_true', help='Also enforce the original migration baseline; intentional later implementation changes will differ.')
    args = parser.parse_args()
    if not ENGINE.is_file():
        raise ValueError('Run just literate-bootstrap first.')
    baseline = json.loads((BASE / 'baseline.json').read_text())
    promotion = json.loads((BASE / 'promotion.json').read_text())
    constructors = json.loads((BASE / 'behavior-map.json').read_text())['constructors']
    exempt = set(promotion['intentional_changed_outputs'])
    checked = 0
    with tempfile.TemporaryDirectory(prefix='hickory-root-reconstruction-') as temporary:
        root = Path(temporary)
        shutil.copy2(ROOT / 'project.md', root / 'project.md')
        for source in (ROOT / '.hickory-source').rglob('*.md'):
            target = root / source.relative_to(ROOT)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
        run('generate', root)
        check_inventory(root, set(constructors))
        run('check', root)
        for entry in baseline['entries'] if args.baseline else []:
            if entry['path'] in exempt:
                continue
            path = root / entry['path']
            metadata = path.lstat()
            link = stat.S_ISLNK(metadata.st_mode)
            data = os.readlink(path).encode() if link else path.read_bytes()
            if (hashlib.sha256(data).hexdigest() != entry['sha256']
                    or len(data) != entry['bytes']
                    or ('symlink' if link else 'file') != entry['kind']
                    or (not link and bool(metadata.st_mode & 0o111)) != entry['executable']):
                raise AssertionError('Unapproved migration difference: ' + entry['path'])
            checked += 1
        for path, expected_hash in (promotion['upstream_addition_hashes'] if args.baseline else {}).items():
            if hashlib.sha256((root / path).read_bytes()).hexdigest() != expected_hash:
                raise AssertionError('Upstream addition differs: ' + path)

        target = root / 'Cargo.toml'
        original = target.read_bytes()
        target.write_bytes(original + b'\n# deliberate drift\n')
        run('check', root, 'output bytes differ: Cargo.toml')
        target.write_bytes(original)
        extra = root / 'unexpected-output.txt'
        extra.write_text('must be rejected\n')
        try:
            check_inventory(root, set(constructors))
        except AssertionError as error:
            if 'unexpected-output.txt' not in str(error):
                raise
        else:
            raise AssertionError('Unexpected output accepted')
        extra.unlink()
        with tempfile.TemporaryDirectory(prefix='hickory-outside-') as outside:
            sentinel = Path(outside) / 'sentinel'
            sentinel.write_bytes(b'outside must stay unchanged')
            target.unlink()
            target.symlink_to(sentinel)
            run('generate', root, 'symlink')
            if sentinel.read_bytes() != b'outside must stay unchanged':
                raise AssertionError('Outside target changed')
            target.unlink()
            target.write_bytes(original)
        run('check', root)
        print(f'PASS: {len(constructors)} outputs reconstructed from Markdown only; '
              f'{checked} baseline outputs checked ({len(exempt)} migration maintenance exceptions); '
              'byte drift, extra output and outside symlink rejected.')


if __name__ == '__main__':
    main()
