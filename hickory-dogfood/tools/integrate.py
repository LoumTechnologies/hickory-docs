#!/usr/bin/env python3
"""Construct a guarantee-organized Markdown source tree from the frozen capture.

This is a one-time migration. The emitted Markdown becomes maintained source.
Reference discovery supplies navigation candidates, never semantic verification.
"""
import collections
import hashlib
import html
import json
import os
from pathlib import Path
import re

BASE = Path(__file__).resolve().parents[1]
SOURCE = BASE / 'workspace'
OUT = BASE / 'integrated'
BOOK = '.hickory-source'
OPEN = '<book:doc xmlns:book="http://www.hickorydocs.com/1.0">\n'
CLOSE = '\n</book:doc>\n'
if OUT.exists():
    raise SystemExit('Refusing to overwrite maintained integrated source.')
baseline = json.loads((BASE / 'baseline.json').read_text())
entries = {e['path']: e for e in baseline['entries']}
overrides = json.loads((BASE / 'tools/behavior-overrides.json').read_text())
text_files = {}
for path, entry in entries.items():
    raw = (SOURCE / path).read_bytes()
    if hashlib.sha256(raw).hexdigest() != entry['sha256']:
        raise SystemExit(f'Frozen source changed: {path}')
    try:
        text_files[path] = raw.decode()
    except UnicodeDecodeError:
        pass

AUDIENCE = {
 'agent': ('Authors using an agent to change documents while retaining review control.', 'Follow a proposed change from context selection through review, application and retained evidence.'),
 'authoring': ('Authors editing executable documents and the files those documents produce.', 'Start with the editing action you want to preserve, then inspect its source updates and refusal cases.'),
 'execution': ('Authors running documents on their own machine and reviewers assessing their side effects.', 'Read the execution boundary before running an unfamiliar document; inspect the refusal and cleanup paths.'),
 'verification': ('Reviewers deciding what an output, recording or prediction actually establishes.', 'Find the expected outcome and its falsifier before using a passing result as evidence.'),
 'literate-projects': ('Maintainers reorganizing source while preserving the complete generated program.', 'Construct the project from an empty tree, compare complete inventory, then review implementation and focused witnesses.'),
 'collaboration': ('Engineers using local sessions or pairing their own machines.', 'Read local persistence and peer access separately; inspect the grants before enabling remote execution.'),
 'lineage': ('Authors editing generated text and reviewers tracing a result to its source.', 'Start with the byte range being traced, then inspect mapping, ambiguity and refusal behavior.'),
 'language': ('Authors embedding code in Markdown and maintainers changing the document grammar.', 'Check how the prefix, delimiter and parser determine structure before changing construction syntax.'),
 'languages': ('Teams choosing a toolchain and writing generators in their existing language.', 'Check the measured language capability before depending on a generator or editor service.'),
 'editor-intelligence': ('Authors using completion, diagnostics and refactoring inside literate source.', 'Trace the requested language operation through the generated source map to its actual language server.'),
 'debugging': ('Authors debugging generated code while navigating its document source.', 'Choose the captured program and source identity before interpreting a stop or variable value.'),
 'terminal': ('Authors running interactive tools alongside documents without losing session state.', 'Find the terminal lifecycle or attention behavior, then follow the command and output paths.'),
 'embedding': ('Host developers embedding the document editor without a native execution service.', 'Identify the host responsibilities before assuming a browser view can execute native tools.'),
 'release': ('Maintainers publishing installers and people checking what a download contains.', 'Choose the platform and channel, then inspect the build provenance and installation witness.'),
 'analytics': ('Maintainers evaluating landing-page behavior and visitor privacy.', 'Check whether the page and event still exist before treating historical analytics claims as current.'),
 'landing': ('Prospective users checking what the homepage demonstrates and promises.', 'Follow the visible claim to the mechanism and the download that can reproduce it.'),
 'lenses': ('Authors reviewing history and choosing where an edit should land.', 'Identify the revision and writable destination before changing a history or comparison view.'),
 'integrations': ('Engineers connecting repository issues, reviews and locally installed tools.', 'Check repository identity and the selected local tool before trusting an external association.'),
 'search': ('Authors finding and replacing project text without missing results.', 'Start with the query and result completeness, then inspect indexing or replacement behavior.'),
 'shell': ('Authors arranging document windows while preserving their editing position.', 'Follow the placement interaction and its editor-state witness before changing geometry.'),
 'editing': ('Authors comparing current code with another revision while continuing to edit.', 'Identify the writable current side and inspect how the saved edit reaches source.'),
 'git': ('Maintainers committing source and checking the staged project.', 'Check the exact staged source and output inventory rather than borrowing working-tree bytes.')
}

def emit(path, body):
    dest = OUT / path
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes((OPEN + body.rstrip() + '\n' + CLOSE).encode())


def link(origin, target, label=None):
    relative = os.path.relpath(target, Path(origin).parent).replace(os.sep, '/')
    return f'[{label or target}]({relative})'


def code_role(path):
    if path.startswith(('docs/', '.claude/', '.codex/', '.grok/', '.skills/', '.instructions/', 'personas/')) or path.endswith('.md'):
        return 'history-and-guidance'
    if '/fixtures/' in path or '/__fixtures__/' in path:
        return 'fixture'
    name = Path(path).name
    if '/tests/' in path or '/e2e/' in path or re.search(r'(?:^test_|\.test\.|\.spec\.|_tests?\.rs$)', name):
        return 'test'
    if '/src/' in path or path.endswith(('.rs','.py','.ts','.tsx','.js','.mjs','.sh','.ps1','.go','.cs')):
        return 'implementation'
    return 'build-and-assets'


def resolve_references(text):
    found, missing = set(), set()
    for token in re.findall(r'(?:[\w.-]+/)+[\w.*{},/+-]+', text, re.ASCII):
        token = token.rstrip('.,/')
        variants = [token]
        match = re.search(r'\{([^{}]+)\}', token)
        if match:
            variants = [token[:match.start()] + part.strip() + token[match.end():] for part in match[1].split(',')]
        for name in variants:
            if name.startswith(('hick-', 'hickory-')):
                name = 'crates/' + name
            if name in entries:
                found.add(name)
            elif name.startswith(('crates/','apps/','code-models/','scripts/','docs/','protocols/')) and '.' in Path(name).name:
                missing.add(name)
    return found, missing


contracts = {}
history_records = []
for path in sorted(p for p in entries if p.startswith('docs/guarantees/') and p.endswith('.md')):
    key = path.removeprefix('docs/guarantees/')
    if key.startswith('audits/') or Path(key).name in ('implementation-evidence.md', 'comparison-integration-evidence.md'):
        history_records.append(path)
        continue
    text = text_files[path]
    title = next((line[2:] for line in text.splitlines() if line.startswith('# ')), Path(key).stem)
    domain = key.split('/')[0]
    normative = text.split('\n---\n', 1)[0]
    historical_start = re.search(r'^## (?:Historical checkpoint|Verification record|Assurance boundary)', normative, re.M)
    if historical_start:
        normative = normative[:historical_start.start()]
    normative = re.sub(r'^# [^\n]*\n', '', normative, count=1).strip()
    refs, missing = resolve_references(text)
    override = overrides.get(key, {})
    refs.update(override.get('implementation', []))
    refs.update(t['file'] for t in override.get('tests', []))
    for name in refs:
        if name not in entries:
            raise SystemExit(f'Curated source path absent: {name}')
    page = f'{BOOK}/guarantees/{key[:-3]}/contract.md'
    flags = []
    if re.search(r'supersed|amend|correction', text, re.I): flags.append('amendments-need-reconciliation')
    if len(normative.splitlines()) > 100: flags.append('contract-needs-splitting')
    if missing: flags.append('some-source-citations-do-not-resolve')
    if not any(code_role(p) in ('test','implementation') for p in refs): flags.append('no-existing-code-or-test-reference-resolved')
    record = dict(id=key, origin=path, page=page, domain=domain,
        title=override.get('title', title),
        disposition=override.get('disposition', 'rewritten' if override else 'migrated-for-review'),
        review='partially verified' if override and 'contract' in override else 'not verified',
        contract=override.get('contract', normative),
        references=sorted(refs), unresolved_references=sorted(missing), flags=flags,
        tests=override.get('tests', []), decision=override.get('decision'),
        reason=override.get('reason'), owned=[], annotations=[], override=override)
    contracts[key] = record

# Test comments establish traceability, not proof of behavioral coverage.
for path, text in text_files.items():
    if code_role(path) not in ('implementation', 'test'):
        continue
    for match in re.finditer(r'docs/guarantees/([\w./-]+\.md)', text):
        key = match[1]
        if key not in contracts:
            continue
        contracts[key]['references'].append(path)
        suffix = text[match.end():match.end()+3000]
        fn = re.search(r'\b(?:async\s+)?fn\s+(\w+)\s*\(', suffix)
        if fn:
            contracts[key]['annotations'].append(dict(file=path, name=fn[1], basis='nearby Rust function after guarantee-path annotation; not a semantic coverage claim'))

uses = collections.defaultdict(set)
for key, record in contracts.items():
    record['references'] = sorted(set(record['references']))
    for path in record['references']:
        if code_role(path) in ('implementation','test','fixture') and record['disposition'] != 'historical':
            uses[path].add(key)
    for test in record['tests']:
        if not re.search(r'\bfn\s+' + re.escape(test['name']) + r'\s*\(', text_files[test['file']]):
            raise SystemExit(f'Curated witness is absent: {test}')

constructors = {}
for path in entries:
    users = sorted(uses[path])
    if len(users) == 1:
        owner = users[0]
        page = Path(contracts[owner]['page']).parent.as_posix() + '/files/' + path + '.md'
        contracts[owner]['owned'].append(path)
    else:
        owner = 'shared' if users else 'support'
        page = f'{BOOK}/{owner}/files/{path}.md'
    constructors[path] = dict(source=page, owner=owner, used_by=users, role=code_role(path), sha256=entries[path]['sha256'])

for path, construction in constructors.items():
    raw = (SOURCE / path).read_bytes()
    try:
        payload = raw.decode()
        encoded = '\x00' in payload or '</book:entry' in payload
    except UnicodeDecodeError:
        encoded = True
    attrs = f'path="{html.escape(path, quote=True)}"'
    if entries[path]['kind'] == 'symlink': attrs += ' kind="symlink"'
    if entries[path]['executable']: attrs += ' executable="true"'
    if encoded:
        attrs += ' encoding="hex"'
        payload = '\n'.join(raw[i:i+64].hex() for i in range(0,len(raw),64))
    page = construction['source']
    audience = 'Maintainers changing this implementation or reviewing its effect on the linked behaviors.'
    if construction['role'] == 'history-and-guidance':
        intro = 'Retained output from the capture. Use the integrated contract pages for review status; statements inside this preserved file may describe an earlier product.'
    elif construction['role'] in ('test','fixture'):
        intro = 'This construction contains the actual witness or fixture source. A reference to a guarantee identifies review scope; only a recorded execution establishes that a check ran.'
    else:
        intro = 'Edit the entry below to change the generated file. Its bytes have one construction owner. Follow the affected guarantees before changing shared behavior.'
    body = f'# {path}\n\nFor: {audience}\n\n{intro}\n\n'
    if construction['used_by']:
        body += 'Read the affected contracts:\n\n' + '\n'.join('- ' + link(page,contracts[k]['page'],contracts[k]['title']) for k in construction['used_by']) + '\n\n'
    body += f'Captured SHA-256: `{entries[path]["sha256"]}`.\n\n'
    if encoded: body += 'The payload uses hex encoding. It has no byte-equal reverse edit map.\n\n'
    body += f'<book:entry {attrs}>\n{payload}</book:entry>\n'
    emit(page,body)

for key, record in contracts.items():
    page, override = record['page'], record['override']
    audience = override.get('audience', AUDIENCE.get(record['domain'],('Maintainers reviewing this behavior.',''))[0])
    body = f'# {record["title"]}\n\nFor: {audience}\n\n'
    if record['disposition'] == 'historical':
        body += record['reason'] + '\n\nThis page constructs no implementation. The complete earlier contract remains in ' + link(page, constructors[record['origin']]['source'], 'historical evidence') + '.\n'
    else:
        if override.get('example'): body += override['example'] + '\n\n'
        body += ('## Behavior to preserve\n\n' if override else '## Contract imported for review\n\n')
        body += record['contract'].replace('<!--', '&lt;!--') + '\n\n'
        if not override:
            body += 'This is the earlier contract with its dated verification history separated. Its amendments and implementation claims require reconciliation before it becomes an accepted replacement.\n\n'
        if override.get('mechanism'):
            body += '## Follow the implementation\n\n' + override['mechanism'] + '\n\n'
        else:
            body += '## Inspect the change\n\nRead the implementation candidates below, then inspect the tests that exercise this behavior. References were resolved from the earlier document and source annotations; they are navigation aids, not verified coverage.\n\n'
        for label, roles in [('Implementation',('implementation',)),('Tests and fixtures',('test','fixture'))]:
            found = [p for p in record['references'] if constructors[p]['role'] in roles]
            if found:
                body += f'### {label}\n\n'
                body += '\n'.join('- ' + link(page, constructors[p]['source'], p) + (' (shared)' if constructors[p]['owner'] == 'shared' else '') for p in found) + '\n\n'
        if not any(constructors[p]['role'] in ('implementation','test','fixture') for p in record['references']):
            body += 'No implementation or test path resolved from the earlier references. This is explicit coverage debt; inspect the complete earlier evidence before relying on the promise.\n\n'
        if record['tests']:
            body += '### Focused witnesses\n\n'
            body += '\n'.join(f'- `{test["name"]}` in ' + link(page,constructors[test['file']]['source'],test['file']) for test in record['tests']) + '\n\n'
        if override.get('limit'):
            body += '## What this does not establish\n\n' + override['limit'] + '\n\n'
        body += '## Construction owned here\n\n'
        if record['owned']:
            body += '\n'.join('- ' + link(page, constructors[p]['source'],p) for p in sorted(record['owned'])) + '\n\n'
            for path in sorted(record['owned']):
                local = os.path.relpath(constructors[path]['source'],Path(page).parent).replace(os.sep,'/')
                body += f'<book:include file="{local}" />\n'
        else:
            body += 'This behavior composes shared implementation. The shared catalog constructs each file once; the links above lead to its actual source.\n'
        body += '\n---\n\nLast LLM verification:\n\n'
        body += f'- Result: {record["review"]}. ' + ('Selected paths inspected; limits above remain.' if override else 'Contract migration and reference discovery only.') + '\n'
        body += '- Originating requirement: ' + link(page, constructors[record['origin']]['source'],record['origin']) + '.\n'
        if record['decision']: body += '- Interpretation: ' + record['decision'] + '\n'
        body += '- Dependencies: the capture hashes for every linked source are recorded in the behavior map. Changes invalidate this review.\n'
        body += '- Executed checks: see the run record in the stakeholder guide; listed witnesses are unrun unless recorded there.\n- Human judgment: no approval of this replacement contract recorded.\n'
        if record['flags']: body += '- Remaining review work: ' + '; '.join(record['flags']) + '.\n'
    emit(page,body)

# Shared implementation and preserved support have one constructor each.
for catalog in ('shared','support'):
    page=f'{BOOK}/{catalog}/catalog.md'
    items=[(p,c) for p,c in constructors.items() if c['owner']==catalog]
    body=f'# {"Shared behavior implementation" if catalog == "shared" else "Build inputs, assets and historical guidance"}\n\n'
    body+='For: maintainers tracing a change that affects several guarantees or reviewing required project inputs.\n\n'
    body+=('Each file below is used by several guarantee pages. Change it once, then review all listed affected contracts.\n\n' if catalog=='shared' else 'These inputs preserve the captured program. Unmapped code is listed explicitly in the review ledger; preserving it does not establish a behavioral contract.\n\n')
    for path,c in sorted(items):
        body+='- '+link(page,c['source'],path)+'\n'
    body+='\n'
    for path,c in sorted(items):
        local=os.path.relpath(c['source'],Path(page).parent).replace(os.sep,'/')
        body+=f'<book:include file="{local}" />\n'
    emit(page,body)

# Domain entry points follow a stakeholder's task; all contracts remain addressable.
for domain in sorted({r['domain'] for r in contracts.values()}):
    page=f'{BOOK}/read/{domain}.md'
    audience, procedure=AUDIENCE.get(domain,('Maintainers reviewing this domain.','Read the behavior, implementation and witnesses together.'))
    body=f'# {domain.replace("-"," ").capitalize()}\n\nFor: {audience}\n\n{procedure}\n\n'
    for record in sorted((r for r in contracts.values() if r['domain']==domain),key=lambda r:r['title']):
        body+='- '+link(page,record['page'],record['title'])+f' — {record["disposition"]}.\n'
    emit(page,body)

root_sources=[r['page'] for r in contracts.values()]+[f'{BOOK}/shared/catalog.md',f'{BOOK}/support/catalog.md']+[f'{BOOK}/read/{d}.md' for d in sorted({r['domain'] for r in contracts.values()})]
root = '# Change a behavior without losing its evidence\n\nFor: Hickory maintainers, authors and reviewers evaluating this guarantee-organized rewrite.\n\nStart with the behavior a stakeholder needs. Read its contract, follow the actual implementation and tests, then inspect its limits. The source closure constructs the same 2,829 files as the captured baseline.\n\n'
root += 'Choose a task:\n\n'
for label,domain in [('Write and edit a document','authoring'),('Reorganize the generated project','literate-projects'),('Review a result or prediction','verification'),('Trace or edit generated bytes','lineage'),('Review an agent change','agent'),('Decide what can execute','execution'),('Pair machines or inspect local persistence','collaboration'),('Publish or install a build','release')]:
    root+='- '+link('project.md',f'{BOOK}/read/{domain}.md',label)+'.\n'
root+='\nThe contract pages retain unresolved review work. Historical hosted contracts are labeled explicitly. Source references establish navigation; they do not make an unrun test pass or settle a product decision.\n\n'
root+='<book:project sources="'+' '.join(root_sources)+'" exclude="target/ node_modules/ apps/web/node_modules/ apps/web/dist/ apps/desktop/src-tauri/target/ apps/agent/.venv/" />\n'
emit('project.md',root)

# Every input has a visible disposition; no orphan source is silently dropped.
serial_contracts={k:{name:value for name,value in r.items() if name not in ('override','contract')} for k,r in contracts.items()}
freeform=[]
for path in sorted(p for p in entries if p.startswith('docs/specs/freeform/') and p.endswith('.md')):
    related=[key for key,r in contracts.items() if path in r['references']]
    freeform.append(dict(path=path, disposition='retained-as-design-history', source=constructors[path]['source'], related_guarantees=related, review='current intent requires reconciliation; not silently adopted'))
ledger=dict(format=1,baseline_revision=baseline['revision'],contracts=serial_contracts,constructors=constructors,
    evidence_records=history_records,freeform_specs=freeform,
    unmapped_implementation=[p for p,c in constructors.items() if c['owner']=='support' and c['role']=='implementation'],
    limitations=['Reference matching is a navigation aid, not semantic coverage.','Historical checks are not executions in this rewrite.','Product decisions and acceptance remain visible review work.'])
(BASE/'behavior-map.json').write_text(json.dumps(ledger,indent=2)+'\n')
print(f'Created {len(contracts)} behavior pages, {len(constructors)} unique file constructions, {len(freeform)} design-history dispositions.')
print(collections.Counter(r['disposition'] for r in contracts.values()))
print(collections.Counter(c['owner'] if c['owner'] in ('support','shared') else 'behavior-owned' for c in constructors.values()))
print(f'Unmapped implementation files: {len(ledger["unmapped_implementation"])}')
