<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Publish the maintained Markdown at one committed revision

For engineers maintaining the public source browser.

Archive only `project.md` and `.hickory-source/` from the named Git commit. Untracked notes, private sessions, local environment files and generated application files never enter this publication. Each page carries its original text, path, revision and SHA-256; viewers fetch it on demand instead of loading the entire codebase. The source mirror is distinct from executable-spec evidence: publishing code runs no cells and claims no test result.

<storycode:copy id="public-source-scripts-export-source-site-py">
"""Publish committed maintained Markdown for the static source browser."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import subprocess
import tarfile


def export(root, destination, revision='HEAD'):
    # Archive the committed allowlist, never local sessions, credentials or outputs.
    revision = subprocess.check_output(['git', 'rev-parse', '--verify', '--end-of-options', revision + '^{commit}'], cwd=root, text=True).strip()
    archive = subprocess.check_output(['git', 'archive', revision, '--', 'project.md', '.hickory-source'], cwd=root)
    documents = []
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
            (destination / url).write_text(json.dumps(dict(path=item.name, revision=revision, source=source), ensure_ascii=False) + '\n')
            documents.append(dict(path=item.name, title=title[1] if title else item.name, sha256=digest, url=url))
    if not any(d['path'] == 'project.md' for d in documents):
        raise ValueError('Committed source has no project.md')
    manifest = dict(version=1, revision=revision, entry='project.md', documents=sorted(documents, key=lambda d: d['path']))
    (destination / 'index.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--revision', default='HEAD')
    args = parser.parse_args()
    result = export(Path.cwd(), args.output, args.revision)
    print(f"Published {len(result['documents'])} Markdown sources at {result['revision']}")


if __name__ == '__main__':
    main()
</storycode:copy>

<storycode:file path="scripts/export-source-site.py"><storycode:paste select="#public-source-scripts-export-source-site-py"/></storycode:file>
</storycode:doc>
