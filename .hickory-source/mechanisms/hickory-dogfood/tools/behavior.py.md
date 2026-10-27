<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/tools/behavior.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-03a0a7186a44"></a>
## `hickory-dogfood/tools/behavior.py`

Construct and verify the maintained literate project, preserving migration evidence separately from current verification.

The capture commands forward explicit entry-independent arguments to the native
adapter. `capture` may name new output paths, `edits` lists IDs or selects one
patch, and `accept`/`new-patch` require a selected capture. Keep `baseline`
separate from `generate`: bootstrap must establish identity without rewriting
products. Other historical wrapper code retains its explanation-review gaps.

The literal construction preserves embedded document-syntax patterns.

<storycode:entry path="hickory-dogfood/tools/behavior.py">
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
    # Several outputs may share a narrative; each output still has one construction.
    assert all((ROOT/page).is_file() for page in pages), 'A constructor source is missing'
    if (ROOT/'.git').exists():
        tracked=subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0')
        outputs={p for p in tracked if p and p!='project.md' and not p.startswith('.hickory-source/')}
        assert outputs<=set(m['constructors']), 'Tracked outputs without a constructor: '+str(sorted(outputs-set(m['constructors'])))
    original_guarantees={e['path'] for e in baseline['entries'] if e['path'].startswith('docs/guarantees/') and e['path'].endswith('.md')}
    assert original_guarantees=={r['origin'] for r in m['contracts'].values()}|set(m['evidence_records']), 'A guarantee/evidence file has no disposition'
    unresolved=[]
    constructions=collections.defaultdict(list)
    definitions=collections.defaultdict(list)
    selections=[]
    for page in [ROOT/'project.md',*(ROOT/'.hickory-source').rglob('*.md')]:
        if not page.is_file(): continue
        text=page.read_text()
        relative=page.relative_to(ROOT).as_posix()
        namespace=re.search(r'xmlns:([\w.-]+)="http://www\.hickorydocs\.com/1\.0"',text)
        editorial=text
        if namespace:
            prefix=re.escape(namespace[1])
            raw=re.compile(r'<'+prefix+r':(entry|copy)\b([^>]*)>.*?</'+prefix+r':\1>',re.S)
            for match in raw.finditer(text):
                attribute='path' if match[1]=='entry' else 'id'
                found=re.search(attribute+r'="([^"]+)"',match[2])
                assert found, 'Missing '+attribute+' in '+relative
                (constructions if match[1]=='entry' else definitions)[found[1]].append(relative)
            editorial=raw.sub('',text)
            for attrs in re.findall(r'<'+prefix+r':file\b([^>]*)>',editorial):
                found=re.search(r'path="([^"]+)"',attrs)
                assert found, 'File construction without path: '+relative
                constructions[found[1]].append(relative)
            selections+=re.findall(r'<'+prefix+r':paste\b[^>]*select="#([^"]+)"',editorial)
        editorial=re.sub(r'(?ms)^```[^\n]*\n.*?^```[^\n]*$', '', editorial)
        editorial=re.sub(r'`[^`\n]*`', '', editorial)
        for target in re.findall(r'\[[^\]\n]+\]\(([^)\s]+)\)',editorial):
            if target.startswith(('#','http:','https:','mailto:','/')):continue
            path=target.partition('#')[0]
            if not (page.parent/path).is_file():unresolved.append((relative,target))
    assert set(constructions)==set(m['constructors']), 'Actual construction inventory differs from the ownership map: '+str(sorted(set(constructions)^set(m['constructors'])))
    for path,locations in constructions.items():
        assert locations==[m['constructors'][path]['source']], 'Duplicate or misplaced construction: '+path+' '+str(locations)
    assert all(len(locations)==1 for locations in definitions.values()), 'A fragment has more than one definition'
    assert set(selections)<=set(definitions), 'Unresolved selected fragment: '+str(sorted(set(selections)-set(definitions)))
    narrative=json.loads((BASE/'narrative-map.json').read_text())
    for fragment in narrative['fragments']:
        assert definitions.get(fragment['id'])==[fragment['page']], 'Fragment ownership differs: '+fragment['id']
    print(f"{len(definitions)} single-owner fragments; {len(narrative['explanation_gaps'])} files retain explicit explanation review work.")
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
    commands.add_parser('baseline',help='Record an agreeing source/output baseline without rewriting products')
    commands.add_parser('check')
    capture=commands.add_parser('capture',help='Capture external output edits without changing concept owners')
    capture.add_argument('--path',action='append',default=[],dest='paths')
    edits=commands.add_parser('edits',help='List pending captures or display one patch')
    edits.add_argument('--id')
    accept=commands.add_parser('accept',help='Accept authored source that exactly reconstructs a capture')
    accept.add_argument('--id',required=True)
    proposal=commands.add_parser('new-patch',help='Author a new concept patch; keep it pending for review')
    proposal.add_argument('--id',required=True)
    proposal.add_argument('--document',required=True)
    proposal.add_argument('--patch-id',required=True)
    proposal.add_argument('--explanation',required=True)
    locate=commands.add_parser('locate');locate.add_argument('output')
    show=commands.add_parser('show');show.add_argument('guarantee')
    test=commands.add_parser('test');test.add_argument('guarantee');test.add_argument('--execute',action='store_true')
    args=parser.parse_args()
    if args.command=='audit':return audit()
    if args.command in ('generate','baseline','check','capture','edits','accept','new-patch'):
        engine=ROOT/'target/dogfood/release/hickory-behavior-engine'
        if not engine.is_file():raise ValueError('Build hickory-dogfood/tools/behavior-engine with the command in docs/developers/literate-workflow.md first.')
        extra=[]
        if args.command=='capture':extra=args.paths
        elif args.command=='edits':extra=[args.id] if args.id else []
        elif args.command=='accept':extra=[args.id]
        elif args.command=='new-patch':extra=[args.id,args.document,args.patch_id,args.explanation]
        result,_=run([engine,args.command,ROOT,*extra]);print(result.stdout.strip());return 0
    m=read_map()
    if args.command=='locate':
        record=m['constructors'].get(args.output)
        if record is None:raise ValueError('Unknown constructed output: '+args.output)
        print('Construction: '+record['source']+('#'+record['anchor'] if record.get('anchor') else ''))
        narrative=json.loads((BASE/'narrative-map.json').read_text())
        for fragment in narrative['fragments']:
            if fragment['output']==args.output:
                print(fragment['page']+'#'+fragment['id']+' — '+fragment['symbol']+' ('+fragment['basis']+')')
        return 0
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
</storycode:entry>


## Challenge narrative ownership

<a id="audit-witness-fixture"></a>
### Fixture

The fixture copies the actual audit command into an isolated miniature project. One source page deliberately contains two output constructions, so accepting it protects the new narrative ownership model.

<storycode:copy id="audit-witness-fixture">
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


AUDIT = Path(__file__).with_name("behavior.py")
NAMESPACE = "http://www.hickorydocs.com/1.0"


class BehaviorAuditTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="hickory-audit-witness-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.base = self.root / "hickory-dogfood"
        (self.base / "tools").mkdir(parents=True)
        shutil.copy2(AUDIT, self.base / "tools/behavior.py")
        self.source = ".hickory-source/behavior.md"
        (self.root / ".hickory-source").mkdir()
        self.prose = ('<s:doc xmlns:s="' + NAMESPACE + '">\n'
            '<s:copy id="decision">accepted\n</s:copy>\n'
            '<s:file path="one.py"><s:paste select="#decision"/></s:file>\n'
            '<s:entry path="two.py">literal\n</s:entry>\n</s:doc>\n')
        self.document = self.root / self.source
        self.document.write_text(self.prose)
        (self.root / "project.md").write_text("Source index\n")
        ownership = {"constructors": {name: {"source": self.source}
            for name in ("one.py", "two.py")}, "contracts": {},
            "evidence_records": [], "freeform_specs": [], "unmapped_implementation": []}
        (self.base / "behavior-map.json").write_text(json.dumps(ownership))
        (self.base / "baseline.json").write_text(json.dumps({"entries": [
            {"path": "one.py"}, {"path": "two.py"}]}))
        (self.base / "narrative-map.json").write_text(json.dumps({
            "fragments": [{"id": "decision", "page": self.source}],
            "explanation_gaps": {}}))

    def audit(self):
        return subprocess.run([sys.executable, str(self.base / "tools/behavior.py"), "audit"],
            cwd=self.root, capture_output=True, text=True)

</storycode:copy>
<a id="audit-witness-accept-multiple-outputs"></a>
### Accept multiple outputs

Two output files may share an explanation page. Acceptance must depend on unique constructions, rather than imposing one Markdown file per output.

<storycode:copy id="audit-witness-accept-multiple-outputs">
    def test_several_outputs_can_share_one_narrative(self):
        result = self.audit()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

</storycode:copy>
<a id="audit-witness-refuse-duplicate-output"></a>
### Refuse duplicate output

A second construction for the same output is ambiguous even when both occur on the mapped page. Reject it before treating the ownership map as complete.

<storycode:copy id="audit-witness-refuse-duplicate-output">
    def test_duplicate_output_is_refused(self):
        self.document.write_text(self.prose.replace("</s:doc>",
            '<s:entry path="one.py">duplicate</s:entry></s:doc>'))
        result = self.audit()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Duplicate or misplaced construction", result.stdout + result.stderr)

</storycode:copy>
<a id="audit-witness-refuse-duplicate-fragment"></a>
### Refuse duplicate fragment

Fragment identifiers are shared by the source closure. A second definition must fail rather than silently choosing the first or last body.

<storycode:copy id="audit-witness-refuse-duplicate-fragment">
    def test_duplicate_fragment_is_refused(self):
        self.document.write_text(self.prose.replace("</s:doc>",
            '<s:copy id="decision">another</s:copy></s:doc>'))
        result = self.audit()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("more than one definition", result.stdout + result.stderr)

</storycode:copy>
<a id="audit-witness-refuse-missing-fragment"></a>
### Refuse missing fragment

An unresolved paste means the narrative cannot construct its promised output. Refuse the missing definition even when output paths and the ownership ledger look complete.

<storycode:copy id="audit-witness-refuse-missing-fragment">
    def test_missing_selected_fragment_is_refused(self):
        self.document.write_text(self.prose.replace('select="#decision"', 'select="#absent"'))
        result = self.audit()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Unresolved selected fragment", result.stdout + result.stderr)
</storycode:copy>

### Assemble the focused audit witnesses

<storycode:file path="hickory-dogfood/tools/test_behavior_audit.py"><storycode:paste select="#audit-witness-fixture"/><storycode:paste select="#audit-witness-accept-multiple-outputs"/><storycode:paste select="#audit-witness-refuse-duplicate-output"/><storycode:paste select="#audit-witness-refuse-duplicate-fragment"/><storycode:paste select="#audit-witness-refuse-missing-fragment"/></storycode:file>

</storycode:doc>
