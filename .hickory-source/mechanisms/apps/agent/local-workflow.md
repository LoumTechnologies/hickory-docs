<build:doc xmlns:build="http://www.hickorydocs.com/1.0">
# Continue from verified documents and measure local inference

For: maintainers changing verified document handoff and measuring local agent behavior.

## dependencies

Trusted imports connect this mechanism to the existing independent loop; no generated code gains host access.

<build:copy id="local-agent-312b58ba-dependencies">
"""Explicit local inference transport for the independent agent and focused experiments."""
from dataclasses import dataclass, asdict
import json
import time
import urllib.request
from urllib.parse import urlparse
from .core import ModelResponse

</build:copy>

## LocalSettings

Allocate context and output together, and validate numeric controls before loading a model. These defaults target a bounded CPU experiment; they are not claimed optimal for every model.

<build:copy id="local-agent-312b58ba-LocalSettings">
@dataclass(frozen=True)
class LocalSettings:
    context_tokens: int = 8192
    output_tokens: int = 768
    temperature: float = 0.2
    top_p: float = 0.95
    top_k: int = 20
    threads: int = 6
    think: bool = False
    seed: int = 42
    response_format: str = 'json'
    timeout_seconds: int = 900

    def validate(self):
        if not 1024 <= self.context_tokens <= 131072:
            raise ValueError('context_tokens must be between 1024 and 131072')
        if not 1 <= self.output_tokens < self.context_tokens:
            raise ValueError('output_tokens must leave room for input')
        if not 0 <= self.temperature <= 2 or not 0 < self.top_p <= 1:
            raise ValueError('Invalid sampling settings')
        if not 1 <= self.top_k <= 1000 or not 1 <= self.threads <= 256:
            raise ValueError('Invalid top_k or thread count')
        if not 1 <= self.timeout_seconds <= 14400:
            raise ValueError('timeout_seconds must be between 1 and 14400')
        if self.response_format not in ('json', 'text'):
            raise ValueError('response_format must be json or text')
        return self

</build:copy>

## OllamaModel

Select an installed loopback model and record its immutable digest and server version. Refuse cloud-backed configurations and obviously insufficient Linux memory. Native requests specify context, sampling, CPU threads and thinking explicitly. Phase timings come from Ollama; the input estimate is labeled heuristic and never silently removes original task text.

<build:copy id="local-agent-312b58ba-OllamaModel">
class OllamaModel:
    """Native local requests: explicit allocation, CPU execution and measured phases.

    No implicit model download, cloud fallback, credential lookup or shell command.
    The input estimate is a planning heuristic, not a tokenizer or proof of coverage.
    """
    def __init__(self, model, settings=None, base_url='http://127.0.0.1:11434'):
        parsed = urlparse(base_url)
        if parsed.scheme != 'http' or parsed.hostname not in ('127.0.0.1', 'localhost', '::1') or parsed.username:
            raise ValueError('Local experiments require a loopback HTTP Ollama endpoint')
        self.model, self.url = model, base_url.rstrip('/')
        self.settings = (settings or LocalSettings()).validate()
        self.calls = []
        self.identity = self._request('/api/show', {'model': model})
        self.identity['ollama_version'] = self._request('/api/version').get('version')
        installed = self._request('/api/tags').get('models', [])
        selected = next((item for item in installed if item.get('name') == model), {})
        self.identity['installed_digest'] = selected.get('digest')
        self.identity['installed_size'] = selected.get('size')
        # Refuse loading a larger model into an already busy Linux host. Existing
        # residency is credited only for this model, never for another process.
        from pathlib import Path
        memory = Path('/proc/meminfo')
        if memory.exists() and selected.get('size'):
            available = int(next(line.split()[1] for line in memory.read_text().splitlines()
                                 if line.startswith('MemAvailable:'))) * 1024
            resident = sum(item.get('size', 0) for item in self._request('/api/ps').get('models', [])
                           if item.get('name') == model)
            if available + resident < selected['size'] + 1_500_000_000:
                raise RuntimeError('Insufficient available memory for this model plus context headroom')
        if self.identity.get('remote_host') or model.endswith(':cloud'):
            raise ValueError('Cloud-backed models are not local experiments')

    def _request(self, path, body=None):
        request = urllib.request.Request(self.url + path,
            data=json.dumps(body).encode() if body is not None else None, headers={'Content-Type': 'application/json'})
        class NoRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, *args, **kwargs):
                raise ValueError('Local inference does not follow redirects')
        opener = urllib.request.build_opener(NoRedirect)
        with opener.open(request, timeout=self.settings.timeout_seconds) as response:
            raw = response.read(4_194_305)
        if len(raw) > 4_194_304:
            raise ValueError('Ollama response exceeds 4 MiB')
        return json.loads(raw)

    def complete(self, messages):
        settings = self.settings
        estimated = sum((len(m['content'].encode()) + 2) // 3 + 8 for m in messages)
        if estimated + settings.output_tokens > settings.context_tokens:
            raise RuntimeError('Estimated context budget exhausted; select or condense documents before inference')
        started = time.monotonic()
        body = {'model': self.model, 'messages': messages,
            'stream': False, 'think': settings.think, 'keep_alive': '30m',
            'options': {'num_ctx': settings.context_tokens, 'num_predict': settings.output_tokens,
                'num_thread': settings.threads, 'num_gpu': 0, 'temperature': settings.temperature,
                'top_p': settings.top_p, 'top_k': settings.top_k, 'seed': settings.seed}}
        if settings.response_format == 'json':
            body['format'] = 'json'
        result = self._request('/api/chat', body)
        elapsed = time.monotonic() - started
        metrics = {key: result.get(key) for key in ('load_duration', 'prompt_eval_count',
            'prompt_eval_duration', 'eval_count', 'eval_duration', 'total_duration', 'done_reason')}
        metrics.update(elapsed_seconds=elapsed, estimated_input_tokens=estimated,
                       settings=asdict(settings), model=self.model)
        self.calls.append(metrics)
        # An interrupted reasoning trace is never an action fallback.
        return ModelResponse(result.get('message', {}).get('content', ''),
                             result.get('done_reason'), metrics)

</build:copy>

<build:file path="apps/agent/src/hickory_agent_worker/local.py">
<build:paste select="#local-agent-312b58ba-dependencies" /><build:paste select="#local-agent-312b58ba-LocalSettings" /><build:paste select="#local-agent-312b58ba-OllamaModel" />
</build:file>

## dependencies

The task fixtures distinguish tempting incorrect shortcuts from required behavior: half-open intervals, chronological balances and first-appearance order. Public examples guide repair; held-out cases are assessed only after the agent stops. These small witnesses do not rank general coding ability.

<build:copy id="local-agent-7a982a28-dependencies">
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

</build:copy>

## digest

Identify exact candidate bytes so acceptance and later checks cannot drift to a different revision.

<build:copy id="local-agent-7a982a28-digest">
def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()

</build:copy>

## evaluate

Run candidate programs only in a separate Monty session, without tool callbacks. Compare actual values with trusted fixture expectations outside the interpreter; an exception or resource limit is non-passing.

A candidate without solve must not inherit a previous passing definition. Evaluate each example in a fresh Monty checkout with no host callbacks; this also prevents one example from seeding globals for the next. Compare the returned value and unchanged input outside the interpreter. An empty example set supplies no passing evidence.

<build:copy id="local-agent-7a982a28-evaluate">
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

</build:copy>

## ExperimentTools

Keep every candidate and public-check receipt in the experiment directory. Hickory first validates the complete output inventory and then reconstructs exactly one solution.py in a disposable root. Submission must reproduce the current candidate and have passing public checks. This development gate never impersonates human production review.

<build:copy id="local-agent-7a982a28-ExperimentTools">
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

</build:copy>

## run case

Use separate interpreters for tool snippets and candidate programs, preventing reentrant evaluation and accidental host access. Record every request, response, cutoff and timing. Score held-out behavior after completion so the model cannot optimize against hidden answers.

<build:copy id="local-agent-7a982a28-run-case">
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

</build:copy>

## main

Resume only an identical configuration, preserving interrupted attempts under distinct names. Compare append and verified-document handoff at each temperature and alternate their order across repetitions. The published report includes failures, rather than selecting only completed attempts.

<build:copy id="local-agent-7a982a28-main">
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

</build:copy>

<build:file path="apps/agent/src/hickory_agent_worker/local_experiment.py">
<build:paste select="#local-agent-7a982a28-dependencies" /><build:paste select="#local-agent-7a982a28-digest" /><build:paste select="#local-agent-7a982a28-evaluate" /><build:paste select="#local-agent-7a982a28-ExperimentTools" /><build:paste select="#local-agent-7a982a28-run-case" /><build:paste select="#local-agent-7a982a28-main" />
</build:file>

## dependencies

Trusted imports connect this mechanism to the existing independent loop; no generated code gains host access.

<build:copy id="local-agent-b2c89978-dependencies">
"""Focused witnesses for document handoff and local experiment boundaries."""
import json
import unittest
from unittest.mock import patch

from hickory_agent_worker.core import Agent, Config, ModelResponse
from hickory_agent_worker.context import MemoryContext
from hickory_agent_worker.local import LocalSettings, OllamaModel
from hickory_agent_worker.hosted import Tools as HostedTools

</build:copy>

## HandoffTests

A fresh request must retain the task, editor context and declared ancestry while removing exploratory messages from delivery. The tests also attempt to execute a truncated but valid-looking action, forge acceptance through a read, and reuse stale acceptance; none may bypass the host gate.

<build:copy id="local-agent-b2c89978-HandoffTests">
class HandoffTests(unittest.TestCase):
    def test_focused_profile_keeps_private_policy_and_valid_tool_example(self):
        from hickory_agent_worker.instructions import select_instructions, INSTRUCTIONS
        focused = select_instructions({'tools':[{'name':'read_doc'}]}, 'focused')
        self.assertLess(len(focused), len(INSTRUCTIONS))
        example = next(line.split('Example read: ')[1].rstrip('.') for line in focused.splitlines() if 'Example read: ' in line)
        self.assertIn('tool(', json.loads(example)['python'])
        private = select_instructions({'tools':[{'name':'seal_prediction'}]}, 'focused')
        self.assertEqual(private, INSTRUCTIONS)
        with self.assertRaises(ValueError): select_instructions({}, 'unknown')

    def test_handoff_keeps_task_and_ancestry_but_removes_exploration(self):
        class Tools:
            pending = False
            def take_handoff(self):
                if not self.pending:
                    return None
                self.pending = False
                return {'entry':'response.md','sources':{'response.md':'Verified construction'},'expected':'B'}
            def complete(self, summary):
                return {'accepted':True}
        tools, calls, events = Tools(), [], []
        context = MemoryContext()
        class Model:
            def complete(self, messages):
                calls.append(messages)
                return '{"python":"wrong_trail()"}' if len(calls)==1 else '{"done":"finished"}'
        def execute(code):
            tools.pending = True
            context.queue('Context update source-2: reread before relying on source-1')
            context.boundary(agent.messages)
            return {'lesson':'wrong trail rejected; submitted actual construction'}
        agent=Agent(Model(),tools,execute,lambda kind,data:events.append((kind,data)),context=context)
        agent.run(Config(prompt='Keep chronological order',context='Current editor selection'))
        fresh=json.dumps(calls[1])
        self.assertIn('Keep chronological order',fresh)
        self.assertIn('Current editor selection',fresh)
        self.assertIn('Verified construction',fresh)
        self.assertIn('Current stage:',calls[1][0]['content'])
        self.assertIn('If a consequential requirement is still unmet',calls[1][0]['content'])
        self.assertNotIn('Current stage:',calls[0][0]['content'])
        self.assertIn('Context update source-2',fresh)
        self.assertNotIn('wrong_trail()',fresh)
        self.assertTrue(any('wrong_trail()' in n['text'] for n in context.nodes.values()))
        derived=[n for n in context.nodes.values() if 'Verified construction' in n['text']]
        self.assertTrue(any(n['parents'] and n['coverage']=='opaque' for n in derived))
        self.assertTrue(any(kind=='context-handoff' for kind,data in events))

    def test_cutoff_never_executes_even_valid_looking_json(self):
        replies=iter([ModelResponse('{"python":"dangerous()"}','length'),ModelResponse('{"done":"done"}','stop')])
        class Model:
            def complete(self,messages): return next(replies)
        class Tools:
            def complete(self,summary): return {'accepted':True}
        executed=[]
        agent=Agent(Model(),Tools(),lambda code:executed.append(code))
        self.assertEqual(agent.run(Config(prompt='Task')), 'done')
        self.assertEqual(executed,[])
        self.assertTrue(any('model_response_truncated' in m['content'] for m in agent.messages))

    def test_stale_handoff_retains_history_and_completion_gate(self):
        class Tools:
            attempts=0
            def take_handoff(self):
                self.attempts+=1
                if self.attempts==2: raise RuntimeError('Project changed after submission')
            def complete(self,summary): return {'accepted':False,'error':'stale'}
        class Model:
            def complete(self,messages): return '{"done":"claim success"}'
        agent=Agent(Model(),Tools(),lambda code:None)
        with self.assertRaisesRegex(RuntimeError,'iteration limit'):
            agent.run(Config(prompt='Task',max_turns=2))
        self.assertTrue(any('handoff_refused' in m['content'] for m in agent.messages))
        self.assertEqual(sum(m['content']=='{"done":"claim success"}' for m in agent.messages),2)

    def test_worker_print_cannot_schedule_host_handoff_and_write_invalidates_it(self):
        class Broker:
            calls=[]
            def call(self,method,params):
                self.calls.append((method,params))
                if method=='handoff': raise RuntimeError('No current accepted changeset')
                if params['name']=='submit_changeset':
                    return {'content':[{'type':'text','text':'{"accepted":true}'}]}
                return {'content':[{'type':'text','text':'{"accepted":true}'}]}
        broker=Broker();tools=HostedTools(broker)
        tools.call('read_doc',{})
        self.assertIsNone(tools.take_handoff())
        tools.call('submit_changeset',{})
        with self.assertRaisesRegex(RuntimeError,'No current accepted'):
            tools.take_handoff()
        self.assertIsNone(tools.take_handoff())

</build:copy>

## LocalTransportTests

Inspect native request controls without loading a model. An interrupted reasoning trace must remain non-executable, oversized input must fail before generation, and invalid or remote configurations must fail before inference.

<build:copy id="local-agent-b2c89978-LocalTransportTests">
class LocalTransportTests(unittest.TestCase):
    def test_settings_and_cutoff_are_recorded_without_reasoning_fallback(self):
        seen=[]
        def request(model,path,body=None):
            seen.append((path,body))
            if path=='/api/show': return {'details':{'quantization_level':'Q4_K_M'}}
            if path=='/api/tags': return {'models':[]}
            if path=='/api/version': return {'version':'test'}
            if path=='/api/ps': return {'models':[]}
            return {'message':{'content':'','thinking':'Pretend this was an action'},
                    'done_reason':'length','eval_count':100,'eval_duration':2_000_000_000}
        with patch.object(OllamaModel,'_request',request):
            model=OllamaModel('fixture',LocalSettings(context_tokens=8192,output_tokens=128))
            response=model.complete([{'role':'user','content':'Task'}])
        self.assertEqual(response.text,'')
        self.assertEqual(response.finish_reason,'length')
        body=next(body for path,body in seen if path=='/api/chat')
        self.assertEqual(body['options']['num_ctx'],8192)
        self.assertEqual(body['options']['num_gpu'],0)
        self.assertEqual(body['options']['temperature'],0.2)
        self.assertEqual(response.metrics['eval_count'],100)

    def test_budget_fails_before_generation_and_never_drops_task(self):
        def request(model,path,body=None): return {'models':[]}
        with patch.object(OllamaModel,'_request',request):
            model=OllamaModel('fixture',LocalSettings(context_tokens=1024,output_tokens=128))
            with self.assertRaisesRegex(RuntimeError,'Estimated context budget'):
                model.complete([{'role':'user','content':'x'*4000}])
        self.assertEqual(model.calls,[])

    def test_local_mode_rejects_remote_or_invalid_configuration(self):
        with self.assertRaises(ValueError): LocalSettings(temperature=float('nan')).validate()
        with self.assertRaises(ValueError): LocalSettings(output_tokens=8192).validate()
        with self.assertRaisesRegex(ValueError,'loopback'):
            OllamaModel('fixture',base_url='https://example.com')
        with patch.object(OllamaModel,'_request',lambda *args:{'remote_host':'cloud'}):
            with self.assertRaisesRegex(ValueError,'Cloud-backed'):
                OllamaModel('fixture')


if __name__=='__main__':
    unittest.main()

</build:copy>

<build:file path="apps/agent/tests/test_local_workflow.py">
<build:paste select="#local-agent-b2c89978-dependencies" /><build:paste select="#local-agent-b2c89978-HandoffTests" /><build:paste select="#local-agent-b2c89978-LocalTransportTests" />
</build:file>

</build:doc>
