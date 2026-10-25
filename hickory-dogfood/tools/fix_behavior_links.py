#!/usr/bin/env python3
"""One-time relocation of editorial links; construction payloads stay untouched."""
import json
import os
import posixpath
import re
from pathlib import Path
BASE=Path(__file__).resolve().parents[1]
ROOT=BASE/'integrated'
m=json.loads((BASE/'behavior-map.json').read_text())
fixed=0
for key,r in m['contracts'].items():
    page=ROOT/r['page']
    text=page.read_text()
    def rewrite(match):
        global fixed
        url=match[2]
        if url.startswith(('#','http:','https:','mailto:','/')):
            return match[0]
        path, sep, anchor=url.partition('#')
        original=posixpath.normpath(posixpath.join(posixpath.dirname(r['origin']),path))
        # The already-generated links resolve in the new source tree.
        if (page.parent/path).is_file():
            return match[0]
        target=None
        if original.startswith('docs/guarantees/'):
            related=original.removeprefix('docs/guarantees/')
            if related in m['contracts']: target=m['contracts'][related]['page']
        if target is None and original in m['constructors']:
            target=m['constructors'][original]['source']
        if target:
            relative=os.path.relpath(ROOT/target,page.parent).replace(os.sep,'/')
            fixed+=1
            return f'[{match[1]}]({relative}{sep}{anchor})'
        return match[0]
    text=re.sub(r'\[([^\]\n]+)\]\(([^)\s]+)\)',rewrite,text)
    page.write_text(text)
print('Relocated editorial links:',fixed)
