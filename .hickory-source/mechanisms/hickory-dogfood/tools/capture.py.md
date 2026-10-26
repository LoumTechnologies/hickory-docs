<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/tools/capture.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-92ee44e7e741"></a>
## `hickory-dogfood/tools/capture.py`

Construct and verify the maintained literate project, preserving migration evidence separately from current verification.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="hickory-dogfood/tools/capture.py">
#!/usr/bin/env python3
"""One-time, non-destructive capture of Git-visible working-tree files."""
import collections
import hashlib
import html
import json
import os
from pathlib import Path
import stat
import subprocess

BASE = Path(__file__).resolve().parents[1]
ROOT = BASE.parent
OUT = BASE / 'workspace'
if OUT.exists():
    raise SystemExit('Refusing to overwrite an existing capture.')
paths = sorted(set(subprocess.check_output(
    ['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'], cwd=ROOT
).decode().strip('\0').split('\0')))
paths = [p for p in paths if p and not p.startswith(BASE.name + '/')]
manifest = []
missing = []
groups = collections.defaultdict(list)
for name in paths:
    path = ROOT / name
    if not path.exists() and not path.is_symlink():
        missing.append(name)
        continue
    if not name.isascii() or any(c.isspace() for c in name):
        raise SystemExit(f'Unsupported portable inventory path: {name!r}')
    mode = path.lstat().st_mode
    if stat.S_ISLNK(mode):
        kind, data = 'symlink', os.readlink(path).encode()
    elif stat.S_ISREG(mode):
        kind, data = 'file', path.read_bytes()
    else:
        raise SystemExit(f'Unsupported entry kind: {name}')
    executable = kind == 'file' and bool(mode & 0o111)
    record = dict(path=name, kind=kind, executable=executable,
                  bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
    manifest.append(record)
    parts = name.split('/')
    if len(parts) == 1:
        group = 'root'
    elif parts[0] in ('crates', 'apps', 'docs', 'code-models', 'experiments'):
        group = '/'.join(parts[:2])
    else:
        group = parts[0]
    try:
        text = data.decode('utf-8')
        # Entry is a raw-content tag: embedded Hickory examples remain bytes.
        # Its own delimiter cannot be quoted; use the supported binary encoding.
        encoded = '</hick:entry' in text or '\x00' in text
    except UnicodeDecodeError:
        encoded = True
    attrs = f'path="{html.escape(name, quote=True)}"'
    if kind == 'symlink':
        attrs += ' kind="symlink"'
    if executable:
        attrs += ' executable="true"'
    if encoded:
        attrs += ' encoding="hex"'
        body = '\n'.join(data[i:i+64].hex() for i in range(0, len(data), 64))
    else:
        body = text
    record['construction'] = 'hex-entry' if encoded else 'utf8-entry'
    block = f'## `{name}`\n\n<hick:entry {attrs}>\n{body}</hick:entry>\n\n'
    groups[group].append(block)

sources = []
chapters = []
OUT.mkdir()
for group, blocks in sorted(groups.items()):
    chunks, current, size = [], [], 0
    for block in blocks:
        if current and size + len(block.encode()) > 180_000:
            chunks.append(current)
            current, size = [], 0
        current.append(block)
        size += len(block.encode())
    if current:
        chunks.append(current)
    for number, chunk in enumerate(chunks, 1):
        name = f'.hickory-source/{group}/{number:03}.md'
        sources.append(name)
        chapters.append(dict(source=name, component=group, entries=len(chunk)))
        dest = OUT / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        title = f'# {group} — construction {number}\n\n'
        desc = ('Frozen documentation and requirements, retained as historical inputs to the reset. '
                'Their claims have not been adopted as a new contract.\n\n' if group.startswith('docs/') else
                'These current file definitions construct this component directly from an empty tree. '
                'Edit the owning entry to change the program; no earlier revision is required.\n\n')
        dest.write_bytes(('<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n' + title + desc + ''.join(chunk) + '</hick:doc>\n').encode())
entry = '# Hickory Docs: complete literate construction\n\n'
entry += ('Start here. The source closure constructs the captured working tree without Git, '
          'network access, execution cells, recordings, or existing generated files. '
          'Chapters are grouped by component and split to keep ordinary chapters manageable.\n\n'
          'The docs chapters freeze the old specifications for review; they do not resolve '
          'their conflicting requirements. Build caches and dependencies are excluded explicitly.\n\n')
entry += '<hick:project sources="' + ' '.join(sources) + '" exclude="target/ node_modules/ apps/web/node_modules/ apps/web/dist/ apps/desktop/src-tauri/target/ apps/agent/.venv/" />\n\n'
for component in sorted(groups):
    entry += f'- {component}: ' + ', '.join(f'[{Path(c["source"]).stem}]({c["source"]})' for c in chapters if c['component'] == component) + '\n'
(OUT / 'project.md').write_text(entry)
(BASE / 'baseline.json').write_text(json.dumps(dict(
    revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
    policy='Existing tracked files plus untracked nonignored files, at capture time; ignored files and deleted tracked paths are absent.',
    absent_tracked=missing, entries=manifest, chapters=chapters,
    initial_status=subprocess.check_output(['git', 'status', '--porcelain', '-uall'], cwd=ROOT).decode()
), indent=2) + '\n')
print(f'Captured {len(manifest)} entries, {sum(e["bytes"] for e in manifest):,} bytes, {len(sources)} chapters.')
</storycode:entry>

</storycode:doc>
