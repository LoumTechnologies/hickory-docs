<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/tools/behavior.py

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="hickory-dogfood/tools/behavior.py">
#!/usr/bin/env python3
"""Navigate behavior source, generate it, and execute named focused witnesses."""
import argparse
import collections
import hashlib
import gzip
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

BASE=Path(__file__).resolve().parents[1]
ROOT=BASE.parent
MAP=BASE/'behavior-map.json'


def read_map():
    return json.loads(MAP.read_text())


def run(argv, *, env=None):
    started=time.monotonic()
    result=subprocess.run(list(map(str,argv)),cwd=BASE.parent,env=env,capture_output=True,text=True)
    if result.returncode:
        raise RuntimeError((result.stdout+'\n'+result.stderr)[-6000:])
    return result, round(time.monotonic()-started,3)


def test_command(test):
    if test.get('runner') == 'vitest':
        return [sys.executable, str(BASE/'tools/frontend-witness.py'), test['file'], test['name']]
    parts=Path(test['file']).parts
    if len(parts)<4 or parts[0]!='crates':
        raise ValueError('No reviewed focused Rust command for this witness.')
    command=['cargo','test','--offline','--locked','--release','--manifest-path',str(ROOT/'Cargo.toml'),'-p',parts[1]]
    if parts[2]=='tests' and len(parts)==4:
        command+=['--test',Path(parts[3]).stem,test['name'],'--','--exact']
    elif parts[2]=='src':
        # Curated unit witnesses are declared inside the crate-root tests module.
        command+=['--lib','tests::'+test['name'],'--','--exact']
    else:
        raise ValueError('Witness needs an explicit test target.')
    return command


def audit():
    m=read_map()
    baseline=json.loads((BASE/'baseline.json').read_text())
    assert {e['path'] for e in baseline['entries']} <= set(m['constructors']),'Incomplete construction inventory'
    pages=[c['source'] for c in m['constructors'].values()]
    assert len(pages)==len(set(pages)), 'Two outputs share a construction file'
    assert all((ROOT/page).is_file() for page in pages), 'A constructor source is missing'
    if (ROOT/'.git').exists():
        tracked=subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0')
        outputs={p for p in tracked if p and p!='project.md' and not p.startswith('.hickory-source/')}
        assert outputs<=set(m['constructors']), 'Tracked outputs without a constructor: '+str(sorted(outputs-set(m['constructors'])))
    original_guarantees={e['path'] for e in baseline['entries'] if e['path'].startswith('docs/guarantees/') and e['path'].endswith('.md')}
    assert original_guarantees=={r['origin'] for r in m['contracts'].values()}|set(m['evidence_records']), 'A guarantee/evidence file has no disposition'
    unresolved=[]
    for page in [ROOT/'project.md',*(ROOT/'.hickory-source').rglob('*.md')]:
        if not page.is_file(): continue
        text=page.read_text()
        # Links quoted inside original file payloads belong to that generated file.
        editorial=re.sub(r'<([A-Za-z_][\w.-]*):entry\b.*?</\1:entry>', '', text, flags=re.S)
        editorial=re.sub(r'(?ms)^```[^\n]*\n.*?^```[^\n]*$', '', editorial)
        editorial=re.sub(r'`[^`\n]*`', '', editorial)
        for target in re.findall(r'\[[^\]\n]+\]\(([^)\s]+)\)',editorial):
            if target.startswith(('#','http:','https:','mailto:','/')):continue
            path=target.partition('#')[0]
            if not (page.parent/path).is_file():unresolved.append((str(page.relative_to(ROOT)),target))
    paragraph_coverage=None
    if m.get('reconciliation_version'):
        from reconciliation_audit import validate
        ledger=json.loads((BASE/'reconciliation/paragraph-ledger.json').read_text())
        originals=json.loads(gzip.decompress((BASE/'reconciliation/original-contracts.json.gz').read_bytes()))
        paragraph_coverage=validate(ledger,m['contracts'],lambda path:originals[path].encode())
    report=dict(paragraph_coverage=paragraph_coverage,contracts=len(m['contracts']),construction_files=len(pages),
        guarantee_dispositions=len(original_guarantees),design_dispositions=len(m['freeform_specs']),
        statuses=dict(collections.Counter(r['disposition'] for r in m['contracts'].values())),
        unresolved_editorial_links=unresolved,unmapped_implementation=m['unmapped_implementation'])
    print(f'{len(pages)} unique constructors; all {len(original_guarantees)} guarantee/evidence files and {len(m["freeform_specs"])} freeform specs have a disposition.')
    if unresolved:
        print(f'{len(unresolved)} editorial links need repair: {unresolved[:20]}')
        return 1
    print('All local editorial file links resolve. Heading fragments are not validated.')
    if paragraph_coverage:print(f'All {paragraph_coverage[1]} original paragraph ranges have a disposition across {paragraph_coverage[0]} contracts; semantic entailment is not asserted.')
    return 0


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    commands=parser.add_subparsers(dest='command',required=True)
    commands.add_parser('audit')
    commands.add_parser('generate')
    commands.add_parser('check')
    show=commands.add_parser('show');show.add_argument('guarantee')
    test=commands.add_parser('test');test.add_argument('guarantee');test.add_argument('--execute',action='store_true')
    args=parser.parse_args()
    if args.command=='audit':return audit()
    if args.command in ('generate','check'):
        engine=ROOT/'target/dogfood/release/hickory-behavior-engine'
        if not engine.is_file():raise ValueError('Build hickory-dogfood/tools/behavior-engine with the command in docs/developers/literate-workflow.md first.')
        result,_=run([engine,args.command,ROOT]);print(result.stdout.strip());return 0
    m=read_map()
    key=args.guarantee.removeprefix('docs/guarantees/')
    if key not in m['contracts']:raise ValueError(f'Unknown guarantee: {key}')
    record=m['contracts'][key]
    if args.command=='show':
        print((ROOT/record['page']).read_text());return 0
    if not record['tests']:
        raise ValueError('No curated executable witness registered. Source and annotation candidates are in behavior-map.json; inspect them before adding a command.')
    receipts=[]
    for test in record['tests']:
        argv=test_command(test)
        if not args.execute:
            import shlex
            print(shlex.join(argv));continue
        env=dict(os.environ,CARGO_TARGET_DIR=str(ROOT/'target/dogfood'))
        result,elapsed=run(argv,env=env)
        if test.get('runner') == 'vitest':
            markers=[line.removeprefix('WITNESS_RESULT ') for line in result.stdout.splitlines() if line.startswith('WITNESS_RESULT ')]
            observed=json.loads(markers[0]) if len(markers)==1 else {}
            passed=observed.get('passed')==1 and observed.get('failed')==0 and observed.get('name')==test['name'] and observed.get('file')==test['file']
        else:
            passed=bool(re.search(r'test result: ok\. 1 passed; 0 failed;',result.stdout))
        if not passed:
            raise ValueError('Expected exactly one passing named witness, got:\n'+result.stdout[-2000:])
        dependencies={p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in record['references'] if (ROOT/p).is_file()}
        receipts.append(dict(guarantee=key,witness=test,command=argv,result='passed',elapsed_seconds=elapsed,
            dependency_hashes=dependencies,output=result.stdout[-4000:],
            scope='One named existing witness; not full behavior or application certification.'))
        print(f'PASS {test["name"]} ({elapsed}s)')
    if args.execute:
        path=ROOT/'target/dogfood/focused-results.json'
        path.parent.mkdir(parents=True,exist_ok=True)
        previous=json.loads(path.read_text()) if path.exists() else []
        path.write_text(json.dumps(previous+receipts,indent=2)+'\n')
    return 0


if __name__=='__main__':
    try:sys.exit(main())
    except (ValueError,RuntimeError,AssertionError) as error:
        print(str(error),file=sys.stderr);sys.exit(1)
</maint:entry>
</maint:doc>
