<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/tools/verify.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9344d17a00b8"></a>
## `hickory-dogfood/tools/verify.py`

Construct and verify the maintained literate project, preserving migration evidence separately from current verification.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="hickory-dogfood/tools/verify.py">
#!/usr/bin/env python3
"""Check exact inventory, bytes, kinds and executable bits against the capture."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import tempfile

BASE = Path(__file__).resolve().parents[1]


def inventory(root, omit_sources=False):
    result = {}
    for parent, dirs, files in os.walk(root, followlinks=False):
        if omit_sources and Path(parent) == root:
            dirs[:] = [d for d in dirs if d != '.hickory-source']
        for name in list(dirs):
            if (Path(parent) / name).is_symlink():
                dirs.remove(name)
                files.append(name)
        for name in files:
            path = Path(parent) / name
            relative = path.relative_to(root).as_posix()
            if omit_sources and relative == 'project.md':
                continue
            mode = path.lstat().st_mode
            kind = 'symlink' if stat.S_ISLNK(mode) else 'file'
            if kind == 'file' and not stat.S_ISREG(mode):
                raise ValueError(f'Unsupported file kind: {relative}')
            data = os.readlink(path).encode() if kind == 'symlink' else path.read_bytes()
            result[relative] = dict(path=relative, kind=kind,
                executable=kind == 'file' and bool(mode & 0o111), bytes=len(data),
                sha256=hashlib.sha256(data).hexdigest())
    return result


def check(root):
    expected = {e['path']: {k: v for k, v in e.items() if k != 'construction'}
                for e in json.loads((BASE / 'baseline.json').read_text())['entries']}
    actual = inventory(root, omit_sources=True)
    errors = []
    for path in sorted(expected.keys() | actual.keys()):
        if path not in actual:
            errors.append(f'Missing: {path}')
        elif path not in expected:
            errors.append(f'Extra: {path}')
        elif expected[path] != actual[path]:
            errors.append(f'Different: {path}')
    if errors:
        raise ValueError('\n'.join(errors[:20]) + f'\n({len(errors)} differences total)')
    return len(expected), sum(e['bytes'] for e in expected.values())


def run(hick, *args):
    result = subprocess.run([str(hick), *map(str, args)], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stderr[-6000:] or result.stdout[-6000:])
    print(result.stdout.strip().splitlines()[-1] if result.stdout.strip() else 'Command passed.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--hick', type=Path, required=True)
    parser.add_argument('--root', type=Path, default=BASE / 'workspace', help='Generated project to check against the original capture.')
    parser.add_argument('--engine', type=Path, help='Use the core harness for fast native materialization.')
    parser.add_argument('--fresh', action='store_true', help='Rebuild in a new directory from Markdown only.')
    args = parser.parse_args()
    hick = args.hick.resolve(strict=True)
    source = args.root.resolve(strict=True)
    count, size = check(source)
    print(f'Independent baseline: {count} entries, {size:,} bytes; hashes, kinds and executable bits match.')
    run(hick, 'project', 'verify', 'project.md', '--root', source)
    if args.fresh:
        # No source files, .git, sidecars or caches are copied into this directory.
        with tempfile.TemporaryDirectory(prefix='hickory-empty-tree-') as temporary:
            root = Path(temporary)
            for path in [source / 'project.md', *sorted(p for p in (source / '.hickory-source').rglob('*.md') if p.is_file())]:
                target = root / path.relative_to(source)
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(path.read_bytes())
            if args.engine:
                run(args.engine.resolve(strict=True), root, BASE / 'baseline.json', '--materialize')
            else:
                run(hick, 'weave', root / 'project.md')
            count, size = check(root)
            run(hick, 'project', 'verify', 'project.md', '--root', root)
            print(f'Fresh Git-free reconstruction: {count} entries, {size:,} bytes match the independent baseline.')
            # Focused negative controls exercise both external and engine comparators.
            sentinel = root / 'unexpected-source.txt'
            sentinel.write_text('must be rejected\n')
            result = subprocess.run([str(hick), 'project', 'verify', 'project.md', '--root', str(root)], capture_output=True)
            assert result.returncode != 0 and b'unexpected-source.txt' in result.stderr, result.stderr
            try:
                check(root)
            except ValueError as error:
                assert 'Extra: unexpected-source.txt' in str(error)
            else:
                raise AssertionError('Independent comparator accepted an extra source.')
            sentinel.unlink()
            target = root / 'Cargo.toml'
            original = target.read_bytes()
            target.write_bytes(original + b'\n# drift\n')
            result = subprocess.run([str(hick), 'project', 'verify', 'project.md', '--root', str(root)], capture_output=True)
            assert result.returncode != 0 and b'Cargo.toml' in result.stderr, result.stderr
            try:
                check(root)
            except ValueError as error:
                assert 'Different: Cargo.toml' in str(error)
            else:
                raise AssertionError('Independent comparator accepted byte drift.')
            target.write_bytes(original)
            print('Negative controls: extra file and byte drift rejected by both comparators.')
            if args.engine:
                # Safety witness for the existing native writer, with a real outside target.
                with tempfile.TemporaryDirectory(prefix='hickory-outside-') as outside:
                    destination = Path(outside) / 'must-not-change'
                    destination.write_bytes(b'outside sentinel')
                    target.unlink()
                    target.symlink_to(destination)
                    result = subprocess.run([str(args.engine.resolve()), str(root),
                        str(BASE / 'baseline.json'), '--materialize'], capture_output=True)
                    assert result.returncode != 0 and b'symlink' in result.stderr, result.stderr
                    assert destination.read_bytes() == b'outside sentinel'
                    target.unlink()
                    target.write_bytes(original)
                check(root)
                print('Safety control: native tree writer refused an outside symlink; outside bytes unchanged.')


if __name__ == '__main__':
    main()
</storycode:entry>

</storycode:doc>
