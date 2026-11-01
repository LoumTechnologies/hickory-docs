"""Focused, resumable local-agent experiments. No production workspace edits.

Candidate programs run only in Monty. Hickory materializes literate candidates
inside disposable directories, then compares exact bytes. Public test receipts
are visible to the agent; held-out results are scored after it stops.
"""
import argparse
from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import time

from .core import Agent, Config
from .local import OllamaModel, LocalSettings
from .hosted import monty, LIMITS
from .runtime import execute
from .cli import ToolBridge


TASKS = [
    dict(id='intervals', requirement='Implement solve(data): merge overlapping half-open intervals. '
         'Input is a list of [start,end] integer pairs with start < end. Return sorted lists. '
         'Touching endpoints are NOT overlap. Do not mutate input. Empty input returns []. '
         'A previous rabbit trail used <= and wrongly merged touching intervals.',
         public=[([[1,3],[3,5]], [[1,3],[3,5]]), ([[5,8],[1,4],[2,6]], [[1,8]])],
         hidden=[([], []), ([[1,9],[2,3]], [[1,9]]), ([[-4,-1],[-1,2],[0,5]], [[-4,-1],[-1,5]])]),
    dict(id='ledger', requirement='Implement solve(data): data has balance (nonnegative integer) and '
         'entries (signed integers). Process entries IN THEIR ORIGINAL ORDER. Reject an entry if it '
         'would make balance negative; continue processing later entries. Return '
         '{"balance": final_balance, "rejected": zero_based_rejected_indices}. '
         'A previous rabbit trail sorted deposits first; that changes the required behavior.',
         public=[({'balance':0,'entries':[-3,5,-2]}, {'balance':3,'rejected':[0]}),
                 ({'balance':4,'entries':[-4,-1,2]}, {'balance':2,'rejected':[1]})],
         hidden=[({'balance':0,'entries':[]}, {'balance':0,'rejected':[]}),
                 ({'balance':1,'entries':[-2,-2,3,-4]}, {'balance':0,'rejected':[0,1]})]),
    dict(id='tokens', requirement='Implement solve(data): split an input string on whitespace, '
         'lowercase each token, and return [[token,count],...] in FIRST APPEARANCE order. '
         'Preserve punctuation within tokens. Empty or whitespace-only input returns []. '
         'A previous rabbit trail alphabetically sorted the result; preserve first appearance instead.',
         public=[('B a B', [['b',2],['a',1]]), (' Hi! hi HI! ', [['hi!',2],['hi',1]])],
         hidden=[('', []), (' \n\t', []), ('z Z a\nz', [['z',3],['a',1]])]),
]

def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()

def evaluate(session, code, cases):
    # The passed session belongs to the caller's action loop. Never run a
    # candidate there: a missing solve() must not borrow an earlier definition.
    # Each public or held-out case receives a fresh interpreter namespace.
    results = []
    with monty() as pool:
        for data, expected in cases:
            try:
                with pool.checkout(limits=LIMITS) as isolated:
                    actual, after = isolated.feed_run(code + '\n__case = ' + repr(data) + '\n__value = solve(__case)\n[__value, __case]',
                                                     external_lookup={})
                actual = json.loads(json.dumps(actual))
                unchanged = after == data
                results.append(dict(passed=actual == expected and unchanged, actual=actual, expected=expected, input_unchanged=unchanged))
            except Exception as exc:
                results.append(dict(passed=False, error=str(exc)[:2000]))
    return dict(passed=bool(results) and all(row['passed'] for row in results), cases=results)

class ExperimentTools:
    catalogue = {'tools': [
        {'name':'read_doc','description':'Read the original task and its public samples.',
         'inputSchema':{'type':'object','properties':{}}},
        {'name':'try_solution','description':'Evaluate a pure Python solve(data) implementation in an isolated Monty interpreter. Record a candidate literate document and actual public-check receipts. No production edits or approval.',
         'inputSchema':{'type':'object','properties':{'code':{'type':'string'},'explanation':{'type':'string'}},'required':['code','explanation']}},
        {'name':'changeset_status','description':'Read the current candidate fingerprint and complete literate submission template.',
         'inputSchema':{'type':'object','properties':{}}},
        {'name':'submit_changeset','description':'Verify an authored literate artifact reproduces the current candidate bytes. Requires passing public checks.',
         'inputSchema':{'type':'object','properties':{'expected':{'type':'string'},'entry':{'type':'string'},'sources':{'type':'object','additionalProperties':{'type':'string'}}},'required':['expected','entry','sources']}},
    ]}

    def __init__(self, task, session, hick, directory):
        self.task, self.session, self.hick, self.directory = task, session, str(hick), directory
        self.code, self.document, self.accepted = '', '', None
        self.candidates, self.checks, self.pending = [], None, False

    def materialize(self, sources, entry, expected_code):
        # Only this trusted wrapper invokes a CLI; no model snippets execute a shell.
        # Each evaluation gets a new root so rejected candidates cannot affect others.
        with tempfile.TemporaryDirectory(prefix='hickory-local-candidate-') as scratch:
            root = Path(scratch)
            for path, text in sources.items():
                relative = Path(path)
                if relative.is_absolute() or '..' in relative.parts or relative.suffix != '.md':
                    raise ValueError('Candidate sources must be contained Markdown paths')
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(text)
            # A candidate can construct only solution.py. Validate the inventory
            # before materializing, and compare all outputs rather than a chosen file.
            result = subprocess.run([self.hick,'project','generate',entry,'--root',str(root)],
                capture_output=True,text=True,timeout=30)
            if result.returncode or result.stdout.strip() != f'File ordinary {len(expected_code.encode())} bytes solution.py':
                raise ValueError('Candidate inventory differs: ' + (result.stdout + result.stderr)[-2000:])
            result = subprocess.run([self.hick,'project','materialize',entry,'--root',str(root)],
                capture_output=True,text=True,timeout=30)
            if result.returncode or (root/'solution.py').read_bytes() != expected_code.encode():
                raise ValueError('Literate reconstruction did not reproduce the candidate bytes')

    def call(self, name, arguments):
        if name == 'read_doc':
            return {'requirement':self.task['requirement'], 'public_samples':self.task['public']}
        if name == 'try_solution':
            self.accepted, self.pending = None, False
            code, explanation = arguments['code'], arguments['explanation']
            if len(code.encode()) > 12000 or len(explanation.encode()) > 8000:
                raise ValueError('Keep the focused candidate small')
            code = code.rstrip() + '\n'
            # A unique namespace prevents generated code or prose from closing a tag.
            prefix = 'candidate' + digest(code + explanation)[:12]
            document = (f'<{prefix}:doc xmlns:{prefix}="http://www.hickorydocs.com/1.0">\n'
                        f'# Successful construction\n\n{explanation}\n\n'
                        f'<{prefix}:project/>\n<{prefix}:file path="solution.py">\n'
                        f'{code}</{prefix}:file>\n</{prefix}:doc>\n')
            self.materialize({'response.md':document}, 'response.md', code)
            self.code, self.document = code, document
            self.checks = evaluate(self.session, code, self.task['public'])
            row = dict(expected=digest(code), code=code, document=document, public=self.checks)
            self.candidates.append(row)
            (self.directory / f'candidate-{len(self.candidates)}.json').write_text(json.dumps(row,indent=2)+'\n')
            return {'expected':digest(code), 'public_checks':self.checks,
                    'evidence':'executed-in-Monty; exact Hickory reconstruction; development fixture only'}
        if name == 'changeset_status':
            return {'content':[{'type':'text','text':json.dumps({'boundary':self.task['id'],
                'expected':digest(self.code),'entry':'response.md','template':self.document})}]}
        if name == 'submit_changeset':
            if arguments['expected'] != digest(self.code) or not self.checks or not self.checks['passed']:
                raise ValueError('Current public checks must pass before submission')
            self.materialize(arguments['sources'], arguments['entry'], self.code)
            self.accepted = {'entry':arguments['entry'],'sources':arguments['sources'],
                             'expected':digest(self.code),'public_checks':self.checks}
            self.pending = True
            return {'content':[{'type':'text','text':json.dumps({'accepted':True})}]}
        raise ValueError('Unknown experiment tool: '+name)

    def take_handoff(self):
        if not self.pending:
            return None
        self.pending = False
        if self.accepted is None or self.accepted['expected'] != digest(self.code):
            raise ValueError('Candidate changed since acceptance')
        return self.accepted

    def complete(self, summary):
        return {'accepted':self.accepted is not None,
                'evidence':'development-only; candidate public checks and exact reconstruction, not production review'}

def run_case(task, model, handoff, hick, directory, max_turns=10, instruction_profile='full'):
    directory.mkdir(parents=True,exist_ok=True)
    events = directory / 'events.jsonl'
    requests = directory / 'requests.jsonl'
    def record(kind, data):
        with events.open('a') as output:
            output.write(json.dumps({'kind':kind,'data':data})+'\n')
    class RecordedModel:
        def complete(self, messages):
            response = model.complete(messages)
            with requests.open('a') as output:
                output.write(json.dumps({'messages':messages,'response':response.text,
                    'finish_reason':response.finish_reason,'metrics':response.metrics})+'\n')
            return response
    started, call_start = time.monotonic(), len(model.calls)
    with monty() as pool, monty() as candidate_pool:
        with pool.checkout(limits=LIMITS) as snippets, candidate_pool.checkout(limits=LIMITS) as interpreter:
            tools = ExperimentTools(task, interpreter, hick, directory)
            bridge = ToolBridge(tools)
            agent = Agent(RecordedModel(), tools,
                lambda code: execute(snippets, bridge, code), record)
            error, summary = None, None
            try:
                summary = agent.run(Config(prompt='Begin with print(tool(\"read_doc\", {})). Implement solve(data) using try_solution with a concise explanation of the chosen construction and its constraints. Read changeset_status, submit its complete template with submit_changeset, then finish. '+task['requirement'],
                    tools=tools.catalogue, max_turns=max_turns, document_handoff=handoff,
                    instruction_profile=instruction_profile))
            except Exception as exc:
                error = str(exc)
            hidden = evaluate(interpreter, tools.code, task['hidden']) if tools.code else {'passed':False}
    calls = model.calls[call_start:]
    result = dict(task=task['id'], handoff=handoff, model=model.model, settings=asdict(model.settings),
        instruction_profile=instruction_profile, completed=error is None and tools.accepted is not None, hidden_passed=hidden['passed'],
        hidden=hidden, error=error, summary=summary, candidates=len(tools.candidates),
        elapsed_seconds=time.monotonic()-started, calls=calls,
        input_tokens=sum(c.get('prompt_eval_count') or 0 for c in calls),
        output_tokens=sum(c.get('eval_count') or 0 for c in calls),
        limits='Three small deterministic fixtures; no frontier comparison, production review or explanation-quality certification')
    (directory/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    return result

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--model',default='qwen3.5:latest')
    parser.add_argument('--hick',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--temperatures',type=float,nargs='+',default=[1.0,0.2])
    parser.add_argument('--tasks',nargs='+',default=[t['id'] for t in TASKS])
    parser.add_argument('--repetitions',type=int,default=1)
    parser.add_argument('--context-tokens',type=int,default=8192)
    parser.add_argument('--output-tokens',type=int,default=768)
    parser.add_argument('--max-turns',type=int,default=10)
    parser.add_argument('--modes',choices=['append','handoff'],nargs='+',default=['append','handoff'])
    parser.add_argument('--instruction-profile',choices=['full','focused'],default='full')
    args=parser.parse_args(argv)
    hick=args.hick.resolve(strict=True)
    args.output.mkdir(parents=True,exist_ok=True)
    configuration=dict(model=args.model,hick=str(hick),temperatures=args.temperatures,tasks=args.tasks,
        repetitions=args.repetitions,context_tokens=args.context_tokens,output_tokens=args.output_tokens,
        max_turns=args.max_turns,modes=args.modes,instruction_profile=args.instruction_profile)
    config_path=args.output/'configuration.json'
    if config_path.exists() and json.loads(config_path.read_text()) != configuration:
        raise ValueError('Use a new output directory when changing experiment configuration')
    config_path.write_text(json.dumps(configuration,indent=2)+'\n')
    results=[]
    for repetition in range(args.repetitions):
        for temperature in args.temperatures:
            model=OllamaModel(args.model, LocalSettings(context_tokens=args.context_tokens,
                output_tokens=args.output_tokens,temperature=temperature,seed=42+repetition))
            (args.output/'model.json').write_text(json.dumps(model.identity,indent=2)+'\n')
            for task in [t for t in TASKS if t['id'] in args.tasks]:
                # Alternate order to reduce a systematic warm-cache advantage.
                modes=args.modes if repetition % 2 == 0 else list(reversed(args.modes))
                for mode in modes:
                    directory=args.output/f'{task["id"]}-t{temperature}-{mode}-r{repetition}'
                    if (directory/'result.json').exists():
                        result=json.loads((directory/'result.json').read_text())
                    else:
                        if directory.exists():
                            directory.rename(directory.with_name(directory.name + '-interrupted-' + str(time.time_ns())))
                        result=run_case(task,model,mode=='handoff',hick,directory,args.max_turns,args.instruction_profile)
                    results.append(result)
                    (args.output/'report.json').write_text(json.dumps(results,indent=2)+'\n')
                    print(json.dumps({k:result[k] for k in ('task','handoff','completed','hidden_passed','elapsed_seconds','input_tokens','output_tokens','error')}),flush=True)
    return 0


if __name__=='__main__':
    raise SystemExit(main())


