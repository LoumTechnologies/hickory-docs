<experiment:doc xmlns:experiment="http://www.hickorydocs.com/1.0">
# Evaluate predictions with a frozen external correctness check

For maintainers reproducing the output-adapter experiment and inspecting its
measurement boundaries. The [runbook](../../experiments/prediction-gate/README.md)
records the implementation findings, frozen plan, reproduction and blog outline.
This page owns the actual harness and focused integrity tests. It adds no product
save guarantee. Existing explanation gaps elsewhere remain in the narrative map.

## Bound candidate observations before exposing any model tools

Candidate Python runs in a separate interpreter with a timeout and a narrow AST/builtin vocabulary. Only JSON arguments cross the observation boundary. This denies supported file and introspection access; it is a trusted-fixture interpreter, not a hostile-code OS sandbox.

<experiment:copy id="prediction-experiment-bench-py-0">
#!/usr/bin/env python3
"""Bounded host for private-output prediction experiments; no model filesystem tools."""
import argparse
import ast
import base64
import difflib
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import random
import shlex
import subprocess
import sys
import tempfile
import time
import urllib.request

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]

def encoded(value):
    return base64.b64encode(json.dumps(value).encode()).decode()

def digest(value):
    return hashlib.sha256(value).hexdigest()

def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")

# No imports, introspection, arbitrary attributes or ambient builtins in candidates.
# This narrow language makes host-only cases unreachable, but is not an OS sandbox.
SAFE_BUILTINS = {k: v for k, v in vars(__import__('builtins')).items()
                 if k in {'min','max','len','sum','range','sorted','enumerate','zip',
                          'list','dict','set','tuple','int','bool','abs','all','any',
                          'reversed','iter','next','str','float','frozenset','map','filter'}}
METHODS = {'get','copy','append','pop','items','keys','values','clear','sort',
           'index','insert','remove','add','discard','extend','setdefault','count',
           'reverse','union','difference','intersection','update'}
FORBIDDEN = (ast.Import, ast.ImportFrom, ast.ClassDef, ast.Global, ast.Nonlocal,
             ast.With, ast.AsyncFunctionDef, ast.Await, ast.Yield, ast.Try,
             ast.Raise)

def evaluate(code, inputs):
    tree = ast.parse(code)
    if len(code.encode()) > 16000:
        raise ValueError('candidate too large')
    for n in ast.walk(tree):
        if isinstance(n, FORBIDDEN):
            raise ValueError('unsupported syntax')
        if isinstance(n, ast.Name) and n.id.startswith('_'):
            raise ValueError('private names forbidden')
        if isinstance(n, ast.Attribute) and n.attr not in METHODS:
            raise ValueError('unsupported attribute')
        if isinstance(n, ast.Constant) and isinstance(n.value, str) and '__' in n.value:
            raise ValueError('private strings forbidden')
    scope = {'__builtins__': SAFE_BUILTINS}
    exec(compile(tree, '<candidate>', 'exec'), scope)
    answers = []
    for args in inputs:
        try:
            answers.append(scope['run'](*args))
        except Exception as error:
            answers.append({'__candidate_error__':type(error).__name__})
    return answers

def observe(code, inputs):
    try:
        p = subprocess.run([sys.executable, '-I', str(Path(__file__).resolve()),
                            '--evaluate', encoded([code, inputs])],
                           capture_output=True, timeout=3, cwd=tempfile.gettempdir(),
                           env={'PATH': os.defpath})
        if p.returncode:
            return {'error': 'candidate_execution_failed'}
        actual = json.loads(p.stdout)
        result = {'actual':actual}
        if any(isinstance(value,dict) and '__candidate_error__' in value for value in actual):
            result['error'] = 'one_or_more_cases_failed'
        return result
    except (subprocess.TimeoutExpired, ValueError):
        return {'error': 'candidate_timeout_or_invalid_output'}

def score(code, cases):
    result = observe(code, [case[0] for case in cases])
    passed = [json.dumps(a,sort_keys=True) == json.dumps(c[1],sort_keys=True) for a, c in zip(result.get('actual', []), cases)]
    if len(passed) != len(cases):
        passed = [False] * len(cases)
    return dict(result, passed=passed, correct=all(passed))

# The model supplies Python only. The host owns the wrapper, question and inputs;
# an agent cannot replace a gate test or expectation with a constant print.
</experiment:copy>

## Keep the question and executable wrapper under host control

The host constructs the Hickory execution and exact identities. The model edits only the candidate implementation, so it cannot replace the public question or command wrapper. Bridge errors stay in evaluator receipts.

<experiment:copy id="prediction-experiment-bench-py-1">
def source_for(code, inputs):
    payload = encoded([code, inputs])
    command = shlex.join([sys.executable, '-I', str(Path(__file__).resolve()), '--evaluate', payload])
    return '<hick:container name="c" image="python" />\n<hick:exec container="c">' + command + '</hick:exec>'

def bridge(request):
    binary = ROOT / 'target/debug/examples/prediction_bench'
    p = subprocess.run([str(binary)], input=json.dumps(request), text=True,
                       capture_output=True, timeout=15, env={'PATH':os.defpath})
    if p.returncode:
        return {'error': 'gate_host_refused', 'host_diagnostic': p.stderr[-2000:]}
    return json.loads(p.stdout)

def gate_request(task, code, directory, attempt):
    inputs = [c[0] for c in task['public']]
    return dict(ledger=str(directory / 'ledger'), source=source_for(code, inputs),
                inputs=inputs, expected=[c[1] for c in task['public']],
                family='bench-v1-' + task['id'], request_id=str(attempt),
                dependencies={},
                environment={'runner':digest(Path(__file__).read_bytes()), 'python':digest(Path(sys.executable).read_bytes()),
                             'host':digest((sys.version + platform.platform()).encode())})

# Only public fields enter provider context. HTTP calls are independent fresh
# conversations per arm; no repository, solution module or receipt paths are sent.
</experiment:copy>

## Send only public context to fresh provider conversations

Live calls expose JSON text only. Simulation deliberately injects known mistakes and repairs to verify accounting; it supplies no experimental evidence about a model.

<experiment:copy id="prediction-experiment-bench-py-2">
def live_response(messages, args):
    if args.backend == 'codex':
        from codex_backend import respond
        return respond(messages, args)
    body = json.dumps(dict(model=args.model, messages=messages, temperature=0,
                           max_tokens=args.output_tokens_limit, response_format={'type':'json_object'})).encode()
    req = urllib.request.Request(args.provider_url, body,
          {'Content-Type':'application/json', 'Authorization':'Bearer ' + os.environ[args.key_env]})
    with urllib.request.urlopen(req, timeout=30) as response:
        data = json.load(response)
    usage = data.get('usage', {})
    return data['choices'][0]['message']['content'], usage

def simulated_response(task, condition, repeat, step):
    # Deliberately inject a wrong initial patch, feedback repair, and frozen wrong
    # prediction. Identical code schedule in both arms checks measurement plumbing.
    code = task['start'] if step == 0 else task['solution']
    prediction = [c[1] for c in task['public']]
    if repeat % 2:
        prediction = ['deliberately wrong']
    return json.dumps(dict(action='submit', code=code, prediction=prediction,
                           rationale='SIMULATED fixture; not a model inference')), {}

</experiment:copy>

## Start an isolated arm with the same budgets

Each arm begins with the same source and public task. The gate instruction is the intervention. Byte and call caps bound API use; usage counters remain distinct from this conservative budgeting proxy.

<experiment:copy id="prediction-experiment-bench-py-3">
def trial(task, condition, repeat, directory, args):
    directory.mkdir(parents=True)
    (directory/'initial.py').write_text(task['start'])
    current = task['start']
    saved = current
    events = []
    instruction = dict(task=task['instruction'], starting_code=current,
                       public_inputs=[c[0] for c in task['public']],
                       tools='Respond with JSON: action=test or submit, code=whole Python file; '
                             'or action=finish. submit is a save attempt. test returns public '
                             f'case pass/fail only. Up to {args.calls} calls, {args.seconds} seconds. '
                             'Only the restricted Python subset is executable; no imports, '
                             'private names, reflection or arbitrary attributes.',
                       condition=condition)
    if 'references' in task:
        instruction['references'] = task['references']
        instruction['benchmark_kind'] = task['benchmark_kind']
        instruction['allowed_builtins'] = sorted(SAFE_BUILTINS)
        instruction['allowed_methods'] = sorted(METHODS)
        instruction['unsupported_syntax'] = [node.__name__ for node in FORBIDDEN]
    if condition == 'gate':
        instruction['gate'] = ('On submit also supply prediction: exact JSON array of results '
              'for public_inputs, and rationale. Seal before observation. Original prediction '
              'must remain unchanged through revisions to earn certification. Save requires '
              'both prediction and original public requirement to pass. test also requires '
              'prediction/rationale and a seal before execution; test does not save.')
    messages = [{'role':'system','content':'Implement the task. Use only the JSON tools described.'},
                {'role':'user','content':json.dumps(instruction)}]
    write(directory/'initial-context.json', messages)
    start = time.monotonic()
    byte_budget = 0
    input_tokens = output_tokens = cached_tokens = calls = 0
    usage_known = True
    failure = None
</experiment:copy>

## Commit before observing, and retain rejected proposals

A submission in the gate arm is durably recorded and sealed before public execution or hidden scoring. Production promotion decides acceptance. Only public booleans and bounded gate feedback return to the model; hidden results remain in receipts. Errors remain failed attempts.

<experiment:copy id="prediction-experiment-bench-py-4">
    for step in range(args.calls):
        elapsed = time.monotonic()-start
        if elapsed >= args.seconds:
            failure = 'elapsed_limit'; break
        # UTF-8 bytes conservatively bound input tokens for the budget estimate.
        byte_budget += len(json.dumps(messages).encode())
        if byte_budget > args.input_bytes:
            failure = 'input_budget'; break
        before = time.monotonic()
        event = dict(step=step, context=messages.copy())
        try:
            calls += 1
            raw, usage = (simulated_response(task, condition, repeat, step) if args.simulated
                          else live_response(messages, args))
            event.update(response=raw, usage=usage)
            usage_known &= isinstance(usage.get('prompt_tokens'),int) and isinstance(usage.get('completion_tokens'),int)
            input_tokens += usage.get('prompt_tokens') or 0
            output_tokens += usage.get('completion_tokens') or 0
            cached_tokens += usage.get('cached_input_tokens') or 0
            action = json.loads(raw)
            if action['action'] == 'finish':
                event['finished'] = True
                events.append(event); break
            code = action['code']
            if not isinstance(code, str) or len(code.encode()) > 16000:
                raise ValueError('invalid code')
            event.update(action=action['action'], code=code, source_hash=digest(code.encode()),
                         patch=''.join(difflib.unified_diff(current.splitlines(True), code.splitlines(True),
                                                         fromfile='before.py', tofile='after.py')))
            current = code
            if action['action'] not in {'submit','test'}:
                raise ValueError('unknown action')
            seal = None
            if condition == 'gate':
                request = gate_request(task, code, directory, step)
                request.update(phase='seal', prediction=action['prediction'], rationale=action['rationale'])
                # Durable commitment precedes BOTH public execution and held-out scoring.
                write(directory/f'{step}-commit.json', request)
                seal = bridge(request)
                event['seal'] = seal
                if 'seal' in seal:
                    request.update(phase='verify', seal=seal['seal'])
                    event['gate'] = bridge(request)
                else:
                    event['gate'] = seal
            if condition == 'gate':
                summary = event.get('gate',{}).get('result',{}).get('summary',{})
                public = {'correct':summary.get('original_requirement_satisfied') is True,
                          'passed':None, 'unavailable':not bool(summary)}
            else:
                public = score(code, task['public'])
            feedback = {'public_passed':public['correct']}
            event['public'] = public
            event['heldout'] = score(code, task['hidden'])
            if condition == 'gate':
                feedback['gate'] = event['gate'].get('result', {'error':'gate_refused'})
            if action['action'] == 'submit':
                promotion = event.get('gate', {}).get('promotion') or {}
                accepted = condition == 'baseline' or bool(promotion.get('current_certification'))
                event.update(save_accepted=accepted, blocked_save=not accepted)
                if accepted:
                    saved = code
                if condition == 'gate':
                    feedback['gate'] = event['gate'].get('result', {'error':'gate_refused'})
                # Host-only diagnostics, heldout and expected answers NEVER enter messages.
                feedback['save_accepted'] = accepted
            event['feedback'] = feedback
            messages = messages + [{'role':'assistant','content':raw},
                                   {'role':'user','content':json.dumps(feedback)}]
            event['elapsed_seconds'] = time.monotonic()-before
            events.append(event)
            if action['action'] == 'submit' and public['correct'] and event['save_accepted']:
                break
        except Exception as error:
            if 'usage' not in event:
                failed_usage = getattr(error,'usage',{})
                event['usage'] = failed_usage
                usage_known &= isinstance(failed_usage.get('prompt_tokens'),int) and isinstance(failed_usage.get('completion_tokens'),int)
                input_tokens += failed_usage.get('prompt_tokens') or 0
                output_tokens += failed_usage.get('completion_tokens') or 0
                cached_tokens += failed_usage.get('cached_input_tokens') or 0
            event['failure'] = type(error).__name__ + ': ' + str(error)
            if 'code' in event and action.get('action') == 'submit' and 'heldout' not in event:
                event.update(heldout=score(event['code'],task['hidden']),
                             public=score(event['code'],task['public']),
                             save_accepted=False, blocked_save=True)
            events.append(event)
            failure = 'provider_or_protocol_failure'; break
        if time.monotonic()-start > args.seconds:
            failure = 'elapsed_limit'; break
    proposals = [e for e in events if 'heldout' in e]
</experiment:copy>

## Score first and accepted final implementations separately

The last proposed code may be correct while the accepted code remains buggy. Count those blocked saves and abandonments explicitly. Missing provider counters are null, preserving the difference between unavailable and free.

<experiment:copy id="prediction-experiment-bench-py-5">
    submissions = [e for e in proposals if e.get('action') == 'submit']
    final = score(saved, task['hidden'])
    first = proposals[0]['heldout'] if proposals else None
    baseline_score = score(task['start'], task['hidden'])
    preserved = [i for i, p in enumerate(baseline_score['passed']) if p]
    result = dict(schema=2, simulated=args.simulated, backend=args.backend, task=task['id'], repeat=repeat,
                  condition=condition, events=events, failure=failure,
                  first_attempt_correct=first['correct'] if first else False,
                  first_submission_correct=submissions[0]['heldout']['correct'] if submissions else False,
                  final=final, completed=any(e.get('save_accepted') and e['public']['correct'] for e in submissions),
                  abandoned=not any(e.get('save_accepted') and e['public']['correct'] for e in submissions),
                  regressions=sum(not final['passed'][i] for i in preserved),
                  regression_denominator=len(preserved),
                  blocked_saves=sum(e.get('blocked_save',False) for e in events),
                  incorrect_predictions=sum(e.get('gate',{}).get('result',{}).get('summary',{}).get('prediction_matched') is False for e in events),
                  retries=max(0,len(submissions)-1),
                  incorrect_code_passed=sum(e.get('save_accepted',False) and not e['heldout']['correct'] for e in submissions) if condition=='gate' else None,
                  correct_code_blocked=sum(e.get('blocked_save',False) and e['heldout']['correct'] for e in submissions),
                  model_calls=0 if args.simulated else None if args.backend=='codex' else calls,
                  cli_turn_attempts=calls if args.backend=='codex' and not args.simulated else 0, simulated_responses=calls if args.simulated else 0,
                  input_tokens=input_tokens if usage_known else None,
                  cached_input_tokens=cached_tokens if usage_known else None,
                  output_tokens=output_tokens if usage_known else None,
                  elapsed_seconds=time.monotonic()-start,
                  estimated_cost=(input_tokens*args.input_rate+output_tokens*args.output_rate)/1e6 if usage_known and args.backend=='api' else None,
                  estimated_credits=(max(0,input_tokens-cached_tokens)*50+cached_tokens*2.5+output_tokens*250)/1e6 if usage_known and args.backend=='codex' and args.model=='gpt-6.1-sol' else None)
    (directory/'final.py').write_text(saved)
    (directory/'final.patch').write_text(''.join(difflib.unified_diff(task['start'].splitlines(True),saved.splitlines(True),fromfile='initial.py',tofile='final.py')))
    write(directory/'record.json', result)
    return result

</experiment:copy>

## Retain every paired outcome in the analysis

The primary comparison uses final accepted code and includes failed sessions. Seeded paired resampling describes these fixed tasks; the exact discordance test and raw counts expose tiny-sample limits.

<experiment:copy id="prediction-experiment-bench-py-6">
# Paired intention-to-treat analysis retains timeouts and protocol failures.
def summarize(records, out):
    arms = {c:[r for r in records if r['condition']==c] for c in ['baseline','gate']}
    pairs = {}
    for r in records:
        pairs.setdefault((r['task'],r['repeat']),{})[r['condition']] = r
    differences = [int(p['gate']['final']['correct'])-int(p['baseline']['final']['correct']) for p in pairs.values()]
    rng = random.Random(711)
    boot = sorted(sum(rng.choices(differences,k=len(differences)))/len(differences) for _ in range(10000))
    wins, losses = differences.count(1), differences.count(-1)
    discordant = wins+losses
    pvalue = min(1, 2*sum(math.comb(discordant,k) for k in range(min(wins,losses)+1))/2**discordant) if discordant else 1
    analysis = dict(pairs=len(pairs), wins=wins, losses=losses,
                    paired_risk_difference=sum(differences)/len(differences),
                    paired_bootstrap_95=[boot[249],boot[9749]], exact_mcnemar_p=pvalue,
                    simulated=all(r['simulated'] for r in records),
                    per_task={task:{c:dict(n=len([r for r in rows if r['task']==task]),
                        final_correct=sum(r['final']['correct'] for r in rows if r['task']==task),
                        failures=sum(r['failure'] is not None for r in rows if r['task']==task))
                        for c,rows in arms.items()} for task in sorted({r['task'] for r in records})})
    admitted = sum('heldout' in e for r in records for e in r['events'])
    analysis['admitted_proposals'] = admitted
    analysis['primary_outcome_available'] = admitted > 0
    analysis['conclusion'] = ('INVALID RUN: no admitted model proposals; no effectiveness estimate' if not analysis['simulated'] and not admitted else
         'SIMULATED: no experimental evidence' if analysis['simulated'] else
         'success on this benchmark' if analysis['paired_risk_difference']>=.10 and boot[249]>0 else 'inconclusive; not evidence of no benefit')
    if not analysis['simulated'] and not admitted:
        for field in ['wins','losses','paired_risk_difference','paired_bootstrap_95','exact_mcnemar_p']:
            analysis[field] = None
    write(out/'analysis.json',analysis)
    lines = ['# '+analysis['conclusion'], '',
             '| Condition | N | First correct | Final correct | Completed | Regressions | Blocked | Wrong predictions | Retries | Abandoned |',
             '|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
    for c, rows in arms.items():
        sums = [len(rows),sum(r['first_attempt_correct'] for r in rows),sum(r['final']['correct'] for r in rows)]
        sums += [sum(r[k] for r in rows) for k in ['completed','regressions','blocked_saves','incorrect_predictions','retries','abandoned']]
        lines.append('| '+c+' | '+' | '.join(map(str,sums))+' |')
    lines += ['', '| Condition | False passes | Correct blocked | Calls | Tokens in/out | Seconds | Cost | Estimated credits |',
              '|---|---:|---:|---:|---|---:|---:|---:|']
    for c,rows in arms.items():
        known = all(r['estimated_cost'] is not None for r in rows)
        cost = str(round(sum(r['estimated_cost'] for r in rows),6)) if known else 'unavailable'
        credits = str(round(sum(r['estimated_credits'] for r in rows),6)) if all(r.get('estimated_credits') is not None for r in rows) else 'unavailable'
        tokens_known = all(r['input_tokens'] is not None and r['output_tokens'] is not None for r in rows)
        tokens = str(sum(r['input_tokens'] for r in rows))+'/'+str(sum(r['output_tokens'] for r in rows)) if tokens_known else 'unavailable'
        lines.append('| '+c+' | '+(str(sum(r['incorrect_code_passed'] or 0 for r in rows)) if c=='gate' else 'n/a')+' | '+str(sum(r['correct_code_blocked'] for r in rows))+' | '+('CLI turns: '+str(sum(r.get('cli_turn_attempts',0) for r in rows)) if any(r.get('cli_turn_attempts') for r in rows) else str(sum(r['model_calls'] or 0 for r in rows)))+' | '+tokens+' | '+str(round(sum(r['elapsed_seconds'] for r in rows),2))+' | '+cost+' | '+credits+' |')
    lines += ['',json.dumps(analysis,indent=2),'', 'Token/cost null means unavailable, never zero. See records.jsonl for per-task metrics and failures.']
    (out/'results.md').write_text('\n'.join(lines)+'\n')
    (out/'records.jsonl').write_text(''.join(json.dumps(r)+'\n' for r in records))
    return analysis

</experiment:copy>

## Freeze settings and refuse overwriting prior evidence

Explicit live mode requires an endpoint, model and credential. A new output directory and hashed manifest retain order, code and budget. Evaluator-only fixtures are imported after model-facing settings have been established.

<experiment:copy id="prediction-experiment-bench-py-7">
# A fresh output directory prevents accidental overwrites or selective reruns.
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evaluate', help=argparse.SUPPRESS)
    parser.add_argument('--out',type=Path)
    parser.add_argument('--simulated',action='store_true')
    parser.add_argument('--live',action='store_true')
    parser.add_argument('--suite',choices=['smoke','reactive'],default='smoke')
    parser.add_argument('--turn-seconds',type=int)
    parser.add_argument('--answer-bytes',type=int)
    parser.add_argument('--output-tokens-limit',type=int)
    parser.add_argument('--backend',choices=['api','codex'],default='api')
    parser.add_argument('--reasoning',choices=['low','medium','high'],default='low')
    parser.add_argument('--provider-url')
    parser.add_argument('--model')
    parser.add_argument('--key-env',default='BENCH_API_KEY')
    parser.add_argument('--repeats',type=int,default=2)
    parser.add_argument('--seed',type=int,default=20261103)
    parser.add_argument('--calls',type=int,default=6)
    parser.add_argument('--seconds',type=int)
    parser.add_argument('--input-bytes',type=int)
    parser.add_argument('--input-rate',type=float,default=2)
    parser.add_argument('--output-rate',type=float,default=8)
    args = parser.parse_args()
    reactive = args.suite == 'reactive'
    if args.seconds is None:args.seconds = 360 if reactive else 180
    if args.input_bytes is None:args.input_bytes = 96000 if reactive else 32000
    if args.turn_seconds is None:args.turn_seconds = 60 if reactive else 30
    if args.answer_bytes is None:args.answer_bytes = 32000 if reactive else 16000
    if args.output_tokens_limit is None:args.output_tokens_limit = 8192 if reactive else 2048
    if args.evaluate:
        code, inputs = json.loads(base64.b64decode(args.evaluate))
        if os.name == 'posix':
            import resource
            resource.setrlimit(resource.RLIMIT_CPU, (2,2))
            if sys.platform.startswith('linux'):
                resource.setrlimit(resource.RLIMIT_AS, (256*1024*1024,256*1024*1024))
        output = json.dumps(evaluate(code,inputs))
        if len(output.encode()) > 4*1024*1024:
            raise ValueError('observation too large')
        print(output); return
    if not args.out or args.out.exists() or args.simulated == args.live:
        parser.error('Choose --simulated or --live and a new --out directory')
    if not 1<=args.repeats<=100 or not 1<=args.calls<=6 or not 1<=args.seconds<=360:
        parser.error('Bounds: 1..100 repeats, 1..6 calls, 1..360 seconds')
    if not 1 <= args.input_bytes <= 96000 or min(args.input_rate,args.output_rate) < 0:
        parser.error('Input cap must be 1..96000 bytes; rates must be nonnegative')
    if not 1<=args.turn_seconds<=60 or not 1<=args.answer_bytes<=32000 or not 1<=args.output_tokens_limit<=8192:
        parser.error('Bounds: 1..60 seconds/turn, 1..32000 answer bytes, 1..8192 API output tokens')
    if args.live and not args.model:
        parser.error('Live mode needs an explicit --model')
    if args.live and args.backend=='api' and (not args.provider_url or not os.environ.get(args.key_env)):
        parser.error('API mode needs --provider-url and the configured credential variable')
    if args.live and args.backend=='codex':
        from codex_backend import preflight
        args.codex_version = preflight()
        args.repository_root = ROOT
        args.transport_counter = 0
        args.out = args.out.resolve()
    if reactive:
        from reactive_cases import TASKS
    else:
        from cases import TASKS
    args.out.mkdir(parents=True)
    schedule = [(t['id'],r,c) for t in TASKS for r in range(args.repeats) for c in ['baseline','gate']]
    random.Random(args.seed).shuffle(schedule)
    write(args.out/'manifest.json',dict(schema=3,protocol='reactive-v4' if reactive else 'codex-subscription-v3' if args.backend=='codex' else 'api-v1',
          settings={k:str(v) if isinstance(v,Path) else v for k,v in vars(args).items()},schedule=schedule,
          transport_sha256=digest((HERE/'codex_backend.py').read_bytes()) if args.backend=='codex' else None,
          benchmark_sha256=digest((HERE/('reactive_cases.py' if reactive else 'cases.py')).read_bytes()),runner_sha256=digest(Path(__file__).read_bytes()),
          bridge_sha256=digest((ROOT/'target/debug/examples/prediction_bench').read_bytes()),
          git_revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
          dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT)),
          estimated_max_cost=len(schedule)*(args.input_bytes*args.input_rate+args.calls*args.output_tokens_limit*args.output_rate)/1e6 if args.backend=='api' else None))
    records = []
    for order,(task_id,repeat,condition) in enumerate(schedule):
        task = next(t for t in TASKS if t['id']==task_id)
        record = trial(task,condition,repeat,args.out/f'{order:03}-{task_id}-{repeat}-{condition}',args)
        record['order']=order
        records.append(record)
    report = summarize(records,args.out)
    print(args.out/'results.md')
    if not args.simulated and not report['primary_outcome_available']:
        print('Invalid run: no model proposals were admitted. Inspect transport receipts; do not start the main experiment.',file=sys.stderr)
        raise SystemExit(2)

if __name__ == '__main__':
    main()
</experiment:copy>

<experiment:file path="experiments/prediction-gate/bench.py"><experiment:paste select="#prediction-experiment-bench-py-0"/><experiment:paste select="#prediction-experiment-bench-py-1"/><experiment:paste select="#prediction-experiment-bench-py-2"/><experiment:paste select="#prediction-experiment-bench-py-3"/><experiment:paste select="#prediction-experiment-bench-py-4"/><experiment:paste select="#prediction-experiment-bench-py-5"/><experiment:paste select="#prediction-experiment-bench-py-6"/><experiment:paste select="#prediction-experiment-bench-py-7"/></experiment:file>

## Freeze the fixtures before live trials

These host-only tasks exercise a helper/caller budget, failed reservations with state preservation, and cache lookup/cleanup at expiry. Each enumerates separate public and held-out inputs plus a known repair for integrity checks. The model receives neither this module nor those repairs.

<experiment:copy id="prediction-experiment-cases-py-0">
"""Frozen v1 fixtures. Host-only: never send this module to the model."""
TASKS = [
    dict(id="deadline", instruction="Fix run(durations, budget): use one total nonnegative budget across retries. Each duration consumes min(duration, remaining); stop at zero. Return elapsed. Preserve empty inputs and short work. Inputs are nonnegative integers.",
         start="def allowance(budget, elapsed):\n    return budget\ndef run(durations, budget):\n    elapsed = 0\n    for duration in durations:\n        elapsed += min(duration, allowance(budget, elapsed))\n    return elapsed\n",
         solution="def allowance(budget, elapsed):\n    return max(0, budget - elapsed)\ndef run(durations, budget):\n    elapsed = 0\n    for duration in durations:\n        elapsed += min(duration, allowance(budget, elapsed))\n    return elapsed\n",
         public=[[[[80,80],100],100],[[[],100],0]],
         hidden=[[[[20,30],100],50],[[[100,1],100],100],[[[1,2],0],0],[[[0,4,9],7],7]], regression=[0,2]),
    dict(id="inventory", instruction="Fix run(stock, operations). operations are [item, quantity] reservations. reserve must reject missing/insufficient stock without mutation, accept exact stock, subtract on success. Return [acceptance booleans, final stock]. Work on a copy of stock. Quantities are positive integers.",
         start="def reserve(stock, item, quantity):\n    stock[item] = stock.get(item, 0) - quantity\n    return stock[item] >= 0\ndef run(stock, operations):\n    stock = stock.copy()\n    accepted = [reserve(stock, item, quantity) for item, quantity in operations]\n    return [accepted, stock]\n",
         solution="def reserve(stock, item, quantity):\n    if stock.get(item, 0) < quantity:\n        return False\n    stock[item] -= quantity\n    return True\ndef run(stock, operations):\n    stock = stock.copy()\n    accepted = [reserve(stock, item, quantity) for item, quantity in operations]\n    return [accepted, stock]\n",
         public=[[[{"a":3},[["a",4],["a",3]]],[[False,True],{"a":0}]]],
         hidden=[[[{},[["x",1]]],[[False],{}]],[[{"a":2,"b":5},[["a",1],["a",2],["b",5]]],[[True,False,True],{"a":1,"b":0}]],[[{"a":2},[]],[[],{"a":2}]],[[{"a":2},[["a",2]]],[[True],{"a":0}]]], regression=[2,3]),
    dict(id="cache", instruction="Fix run(events, ttl). events are [time,key,value]; value=None means get, otherwise put. Return [get results, remaining sorted keys]. Cache entries expire at time >= insertion+ttl. get and final cleanup must agree; a put replaces the expiry. Times are nondecreasing integers, ttl nonnegative; values are integers. Missing get returns None.",
         start="def live(entry, now):\n    return entry[1] >= now\ndef run(events, ttl):\n    cache = {}\n    out = []\n    now = 0\n    for now, key, value in events:\n        if value is not None:\n            cache[key] = [value, now + ttl]\n        else:\n            entry = cache.get(key)\n            out.append(entry[0] if entry and live(entry, now) else None)\n    return [out, sorted(key for key in cache if live(cache[key], now))]\n",
         solution="def live(entry, now):\n    return entry[1] > now\ndef run(events, ttl):\n    cache = {}\n    out = []\n    now = 0\n    for now, key, value in events:\n        if value is not None:\n            cache[key] = [value, now + ttl]\n        else:\n            entry = cache.get(key)\n            out.append(entry[0] if entry and live(entry, now) else None)\n    return [out, sorted(key for key in cache if live(cache[key], now))]\n",
         public=[[[[[0,"a",7],[5,"a",None]],5],[[None],[]]]],
         hidden=[[[[[0,"a",7],[4,"a",None]],5],[[7],["a"]]],[[[[0,"a",7],[3,"a",8],[5,"a",None]],5],[[8],["a"]]],[[[[0,"a",7],[0,"a",None]],0],[[None],[]]],[[[],5],[[],[]]]], regression=[0,1,3]),
]
</experiment:copy>

<experiment:file path="experiments/prediction-gate/cases.py"><experiment:paste select="#prediction-experiment-cases-py-0"/></experiment:file>

## Check the fixture oracles and the real production gate

The focused witnesses run reference repairs, deny supported host introspection, and exercise unchanged-expectation repair, an after-observation prediction change, a false block and an overfit false pass. These are harness integrity checks, not broad product regression coverage.

<experiment:copy id="prediction-experiment-test_bench-py-0">
"""Focused experiment integrity witnesses, not a product regression suite."""
import json
from pathlib import Path
import tempfile
import unittest
import bench
from cases import TASKS

class Integrity(unittest.TestCase):
    def test_frozen_oracles_and_regression_denominators(self):
        for task in TASKS:
            self.assertTrue(bench.score(task['solution'],task['public'])['correct'])
            self.assertTrue(bench.score(task['solution'],task['hidden'])['correct'])
            self.assertFalse(bench.score(task['start'],task['hidden'])['correct'])
            self.assertEqual([i for i,p in enumerate(bench.score(task['start'],task['hidden'])['passed']) if p],task['regression'])

    def test_candidate_cannot_import_or_introspect_host(self):
        for code in ['import os\ndef run(): return os.environ',
                     'def run(): return ().__class__',
                     'def run(): return open("cases.py").read()',
                     'def run(): return globals()']:
            self.assertIn('error',bench.observe(code,[[]]))
        self.assertIn('error',bench.observe('def run():\n    while True: pass',[[]]))

    def test_real_gate_false_positive_false_negative_and_frozen_prediction(self):
        task = TASKS[0]
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            def run(code,prediction,step):
                req=bench.gate_request(task,code,root,step)
                req.update(phase='seal',prediction=prediction,rationale='integrity witness')
                sealed=bench.bridge(req)
                self.assertIn('seal',sealed)
                req.update(phase='verify',seal=sealed['seal'])
                return bench.bridge(req)
            # Correct code, wrong prediction: no certification.
            first=run(task['solution'],['wrong'],0)
            self.assertIsNone(first['promotion'])
            # Observing then replacing the original expectation cannot certify.
            second=run(task['solution'],[100,0],1)
            self.assertIsNone(second['promotion'])
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            # Overfit two public cases: production runner certifies but hidden fails.
            code='def run(durations, budget):\n    return 100 if durations else 0\n'
            result=run(code,[100,0],0)
            self.assertTrue(result['promotion']['current_certification'])
            self.assertFalse(bench.score(code,task['hidden'])['correct'])
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            # Unchanged prediction survives a real source repair.
            bad=run(task['start'],[100,0],0)
            fixed=run(task['solution'],[100,0],1)
            self.assertIsNone(bad['promotion'])
            self.assertEqual(fixed['result']['summary']['classification'],'verified_after_correction')
            self.assertTrue(fixed['promotion']['current_certification'])


class CodexTransportIntegrity(unittest.TestCase):
    def test_subscription_environment_and_native_tool_rejection(self):
        from unittest.mock import patch
        import codex_backend as codex
        with patch.dict('os.environ', {'OPENAI_API_KEY':'dummy','CODEX_API_KEY':'dummy','BENCH_API_KEY':'dummy'}):
            self.assertFalse({'OPENAI_API_KEY','CODEX_API_KEY','BENCH_API_KEY'} & codex.environment().keys())
        completed = json.dumps({'type':'turn.completed','usage':{'input_tokens':100,'output_tokens':10,'cached_input_tokens':20}})
        raw,usage = codex.parse_events(completed,'{"action":"finish"}')
        self.assertEqual(usage['prompt_tokens'],100)
        self.assertEqual(usage['billing'],'chatgpt_subscription')
        self.assertFalse(usage['exact_provider_call_count_known'])
        for event in [{'type':'item.started','item':{'type':'command_execution'}},
                      {'type':'item.completed','item':{'type':'mcp_tool_call'}},
                      {'type':'turn.failed'}]:
            with self.assertRaises(RuntimeError):
                codex.parse_events(json.dumps(event)+'\n'+completed,raw)

    def test_codex_startup_diagnostics_are_not_native_tool_calls(self):
        import codex_backend as codex
        diagnostics = [
            'Under-development features enabled: skip_host_skill_discovery. Under-development features are incomplete and may behave unpredictably.',
            'Code Mode is unavailable because code-mode host is disabled. Code mode will fail closed; enable `features.code_mode_host` and install `codex-code-mode-host`.']
        events = [{'type':'item.completed','item':{'type':'error','message':message}} for message in diagnostics]
        events += [{'type':'item.completed','item':{'type':'agent_message','text':'{}'}},
                   {'type':'turn.completed','usage':{'input_tokens':8780,'output_tokens':157}}]
        raw,usage = codex.parse_events('\n'.join(map(json.dumps,events)),'{}')
        self.assertEqual(raw,'{}')
        self.assertEqual(usage['startup_warnings'],diagnostics)
        self.assertEqual(usage['prompt_tokens'],8780)
        for invalid in [{'type':'item.completed','item':{'type':'error','message':'actual adapter failure'}},
                        {'type':'item.started','item':{'type':'command_execution','command':'cat cases.py'}},
                        {'type':'error','message':diagnostics[0]}]:
            with self.assertRaises(codex.TransportFailure) as failure:
                codex.parse_events('\n'.join(map(json.dumps,events+[invalid])),'{}')
            self.assertEqual(failure.exception.usage['completion_tokens'],157)

    def test_infrastructure_only_run_has_no_effectiveness_estimate(self):
        common = dict(task='dummy',repeat=0,events=[],simulated=False,
                      failure='provider_or_protocol_failure', final={'correct':False},
                      first_attempt_correct=False,completed=False,regressions=0,
                      blocked_saves=0,incorrect_predictions=0,retries=0,abandoned=True,
                      incorrect_code_passed=None,correct_code_blocked=0,model_calls=None,
                      cli_turn_attempts=1,estimated_cost=None,input_tokens=None,
                      output_tokens=None,elapsed_seconds=1)
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            bench.summarize([dict(common,condition=c) for c in ['baseline','gate']],root)
            report=json.loads((root/'analysis.json').read_text())
            self.assertEqual(report['pairs'],1)
            self.assertIsNone(report['paired_risk_difference'])
            self.assertIsNone(report['paired_bootstrap_95'])
            self.assertFalse(report['primary_outcome_available'])
            self.assertTrue(report['conclusion'].startswith('INVALID RUN'))

    def test_rejected_turn_retains_its_measured_usage(self):
        from types import SimpleNamespace
        from unittest.mock import patch
        import codex_backend as codex
        args=SimpleNamespace(simulated=False,backend='codex',model='gpt-6.1-sol',
                             calls=1,seconds=180,input_bytes=32000,input_rate=2,output_rate=8)
        error=codex.TransportFailure('unexpected native tool',
              {'prompt_tokens':100,'completion_tokens':10,'cached_input_tokens':20})
        with tempfile.TemporaryDirectory() as temp:
            with patch.object(bench,'live_response',side_effect=error):
                record=bench.trial(TASKS[0],'baseline',0,Path(temp)/'arm',args)
        self.assertEqual(record['failure'],'provider_or_protocol_failure')
        self.assertEqual(record['input_tokens'],100)
        self.assertEqual(record['output_tokens'],10)
        self.assertEqual(record['cached_input_tokens'],20)
        self.assertAlmostEqual(record['estimated_credits'],.00655)
        self.assertIsNone(record['model_calls'])
        self.assertEqual(record['cli_turn_attempts'],1)

    @unittest.skipUnless(__import__('sys').platform=='darwin','macOS transport')
    def test_child_cannot_read_fixture_even_with_a_known_path(self):
        import subprocess
        import codex_backend as codex
        with tempfile.TemporaryDirectory(prefix='hickory-confinement-test-') as temp:
            scratch=Path(temp)/'empty';scratch.mkdir()
            hidden=Path(temp)/'heldout.txt';hidden.write_text('HOST_ONLY_SENTINEL')
            policy=codex.confinement(bench.ROOT,bench.ROOT/'target',scratch)
            for path in [hidden,bench.HERE/'cases.py']:
                p=subprocess.run(['sandbox-exec','-p',policy,'/bin/cat',str(path)],capture_output=True)
                self.assertNotEqual(p.returncode,0)
                self.assertNotIn(b'HOST_ONLY_SENTINEL',p.stdout)
            public=scratch/'public.txt';public.write_text('PUBLIC')
            p=subprocess.run(['sandbox-exec','-p',policy,'/bin/cat',str(public)],capture_output=True)
            self.assertEqual(p.returncode,0)
            self.assertEqual(p.stdout,b'PUBLIC')

if __name__ == '__main__':
    unittest.main()
</experiment:copy>

<experiment:file path="experiments/prediction-gate/test_bench.py"><experiment:paste select="#prediction-experiment-test_bench-py-0"/></experiment:file>

## Use the existing runner rather than reimplementing the gate

The bridge captures source, input, dependency and environment identities from host fields. It creates the same output target the production adapter expects. The model has no direct access to these arguments.

<experiment:copy id="prediction-experiment-bridge-rs-0">
//! Experiment host adapter: use the production private-output runner and ledger.
use hick_literate::prediction::{OutputEncoding, OutputRequest, output_target, run_output};
use hick_observe::prediction::*;
use hickory_executor::LocalExecutor;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{io::{self, Read}, sync::Arc};

fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let v: Value = serde_json::from_str(&input)?;
    let ledger = Ledger::open(v["ledger"].as_str().unwrap())?;
    let source = v["source"].as_str().unwrap();
    let identity = Identity {
        source: hash(source.as_bytes()), inputs: digest(&v["inputs"] )?,
        dependencies: serde_json::from_value(v["dependencies"].clone())?,
        environment: serde_json::from_value(v["environment"].clone())?,
    };
    let actor = Actor { kind: "agent".into(), id: "benchmark-predictor".into() };
</experiment:copy>

## Seal once, then dispatch and request promotion

The seal phase runs before observation. The verify phase invokes production run_output with a fresh private executor and asks the existing ledger to promote the resulting attempt. A refused promotion never becomes an accepted benchmark save.

<experiment:copy id="prediction-experiment-bridge-rs-1">
    if v["phase"] == "seal" {
        let receipt = ledger.seal(ledger.history(None)?.generation, Seal {
            scenario: Scenario {
                family: v["family"].as_str().unwrap().into(), inputs: v["inputs"].clone(),
                requirement_reference: "Frozen benchmark public cases; exact JSON results".into(),
                requirement: Predicate::Equals(v["expected"].clone()), identity_known: true,
            }, identity: identity.clone(), target: output_target("c", 2, OutputEncoding::Json),
            predicted: Predicate::Equals(v["prediction"].clone()),
            rationale: v["rationale"].as_str().unwrap_or("").into(),
        }, actor)?;
        println!("{}", json!({"seal":receipt.id,"identity":identity}));
    } else {
        let result = run_output(&ledger, &OutputRequest {
            seal: v["seal"].as_str().unwrap().into(),
            request_id: v["request_id"].as_str().unwrap().into(), actor,
            identity: identity.clone(), encoding: OutputEncoding::Json,
        }, source, Arc::new(LocalExecutor::new()?)).await?;
        let promotion = ledger.promote(&result.summary.attempt, &identity,
            Actor {kind:"host".into(),id:"benchmark-host".into()}).ok();
        println!("{}", json!({"result":result,"promotion":promotion}));
    }
    Ok(())
}
</experiment:copy>

<experiment:file path="crates/hick-literate/examples/prediction_bench.rs"><experiment:paste select="#prediction-experiment-bridge-rs-0"/><experiment:paste select="#prediction-experiment-bridge-rs-1"/></experiment:file>

## Keep authentication in Codex and deny held-out file reads

The CLI owns its sign-in. Key overrides are removed, while Seatbelt denies repository, evaluator-copy and prior-session reads. Scratch contains only public prompt/response material.

<experiment:copy id="prediction-experiment-codex_backend-py-0">
"""Subscription-backed, tool-confined Codex transport for versioned experiments.

The CLI owns authentication. Never extract or forward its OAuth credentials.
"""
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile

CREDIT_RATES = {'gpt-6.1-sol': (50, 2.5, 250)}
DISABLED = ['shell_tool','unified_exec','apps','code_mode','code_mode_host',
            'multi_agent','multi_agent_v2','browser_use','browser_use_external',
            'computer_use','view_image','memories','skill_search','sleep_tool',
            'unbounded_connection_retries']


def environment():
    # Removing key overrides plus forced_login_method prevents API billing fallback.
    env = dict(os.environ)
    for name in ('OPENAI_API_KEY','CODEX_API_KEY','BENCH_API_KEY'):
        env.pop(name, None)
    return env


def confinement(root, output, scratch):
    if sys.platform != 'darwin' or not shutil.which('sandbox-exec'):
        raise RuntimeError('Codex benchmark confinement currently requires macOS sandbox-exec')
    home = Path.home()/'.codex'
    # Known evaluator copies and receipts are under the repo, temp roots or Codex
    # history. Deny those even if a built-in tool is accidentally still advertised.
    denied = {str(Path(p).resolve()) for p in (root, output, '/tmp', tempfile.gettempdir(),
              home/'sessions', home/'archived_sessions', home/'memories', home/'history.jsonl')}
    rules = ['(version 1)', '(allow default)']
    rules += ['(deny file-read-data (subpath '+json.dumps(p)+'))' for p in sorted(denied)]
    rules += ['(deny file-write* (subpath '+json.dumps(str(Path(p).resolve()))+'))' for p in (root,output)]
    rules += ['(deny file-read-data (regex '+json.dumps(str(home.resolve())+r'/state_.*\.sqlite.*')+'))']
    rules += ['(allow file-read-data (subpath '+json.dumps(str(Path(scratch).resolve()))+'))']
    return '\n'.join(rules)


</experiment:copy>

## Make both conditions use the same confined Codex configuration

Explicit model/effort and disabled native tools keep model access equal. The subscription transport fails closed outside macOS and never falls back to API-key billing.

<experiment:copy id="prediction-experiment-codex_backend-py-1">
def command(model, reasoning, scratch, root, output):
    binary = shutil.which('codex')
    if not binary:
        raise RuntimeError('Install Codex CLI and run codex login first')
    cmd = ['sandbox-exec','-p',confinement(root,output,scratch),binary,
           '-a','never','exec','--ignore-user-config','--ignore-rules',
           '--ephemeral','--skip-git-repo-check','--sandbox','read-only',
           '--json','--color','never','--cd',str(scratch), '--model',model,
           '-c','forced_login_method="chatgpt"',
           '-c','sqlite_home='+json.dumps(str(Path(scratch)/'state')),
           '-c','log_dir='+json.dumps(str(Path(scratch)/'logs')),
           '-c','model_reasoning_effort='+json.dumps(reasoning),
           '-c','web_search="disabled"', '-c','agents.enabled=false',
           '-c','skills.include_instructions=false', '-c','skills.bundled.enabled=false',
           '-c','project_doc_max_bytes=0', '-c','cloud.skills.enabled=false',
           '-c','features.skip_host_skill_discovery=true',
           '--output-last-message',str(Path(scratch)/'answer.json')]
    for name in DISABLED:
        cmd += ['--disable',name]
    return cmd + ['-']


</experiment:copy>

## Reject tool activity and qualify usage counters

A completed turn supplies measured token counts. Native tool events invalidate the trial, and a CLI turn is never misreported as a known count of hidden provider calls.

<experiment:copy id="prediction-experiment-codex_backend-py-2">
class TransportFailure(RuntimeError):
    """Keep measured usage even when a completed response cannot be admitted."""
    def __init__(self, message, usage):
        super().__init__(message)
        self.usage = usage


def event_usage(events):
    completed = [e for e in events if e.get('type') == 'turn.completed']
    if len(completed) != 1:
        return {}
    usage = completed[0].get('usage',{})
    return dict(prompt_tokens=usage.get('input_tokens'),
                completion_tokens=usage.get('output_tokens'),
                cached_input_tokens=usage.get('cached_input_tokens'),
                reasoning_output_tokens=usage.get('reasoning_output_tokens'),
                billing='chatgpt_subscription', cli_turns=1,
                exact_provider_call_count_known=False)


def parse_events(stdout, answer, max_answer_bytes=16000):
    events = [json.loads(line) for line in stdout.splitlines() if line.strip()]
    usage = event_usage(events)
    allowed = {'agent_message','reasoning'}
    warnings = []
    for event in events:
        if event.get('type','').startswith('item.'):
            item = event.get('item',{})
            message = item.get('message','')
            # Codex 0.166 emits these startup diagnostics as error ITEMS, despite
            # a successfully completed turn. Neither represents a tool invocation.
            known_warning = item.get('type') == 'error' and (
                message.startswith('Under-development features enabled: skip_host_skill_discovery. ')
                or message == 'Code Mode is unavailable because code-mode host is disabled. Code mode will fail closed; enable `features.code_mode_host` and install `codex-code-mode-host`.')
            if known_warning:
                warnings.append(message)
                continue
            if item.get('type') not in allowed:
                raise TransportFailure('Unexpected Codex tool/item; trial is invalid, not correctness evidence', usage)
        if event.get('type') in {'error','turn.failed'}:
            raise TransportFailure('Codex provider/turn failure; inspect host-only CLI receipts', usage)
    if not usage or not answer or len(answer.encode()) > max_answer_bytes:
        raise TransportFailure('Expected one bounded completed Codex turn', usage)
    usage['startup_warnings'] = warnings
    return answer, usage


</experiment:copy>

## Preserve transport receipts and terminate timed-out turns

Every response receives only replayed public context in a fresh empty thread. Host-only command, event, error and exit receipts survive failure. A timeout kills the process group but cannot certify provider cancellation.

<experiment:copy id="prediction-experiment-codex_backend-py-3">
def respond(messages, args):
    # A fresh directory/thread per response receives replayed PUBLIC history only.
    # No parent repository instructions, solution module or previous arm is loaded.
    receipt = args.out/'transport'/f'{args.transport_counter:04}'
    args.transport_counter += 1
    receipt.mkdir(parents=True, exist_ok=False)
    with tempfile.TemporaryDirectory(prefix='hickory-codex-turn-') as scratch:
        cmd = command(args.model,args.reasoning,Path(scratch),args.repository_root,args.out)
        (receipt/'command.json').write_text(json.dumps(cmd,indent=2)+'\n')
        prompt = ('You are the JSON-only backend for a controlled coding experiment. '
                  'The following array is the complete conversation. Continue it with '
                  'one assistant response containing only valid JSON, without Markdown. '
                  f'Use no native tools. Keep the answer within {args.answer_bytes} UTF-8 bytes.\n'+json.dumps(messages))
        (receipt/'prompt.txt').write_text(prompt)
        process = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True, env=environment(),
                                   cwd=scratch, start_new_session=True)
        timed_out = False
        try:
            stdout, stderr = process.communicate(prompt, timeout=args.turn_seconds)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            stdout, stderr = process.communicate()
        (receipt/'events.jsonl').write_text(stdout)
        (receipt/'stderr.txt').write_text(stderr)
        (receipt/'exit.json').write_text(json.dumps({'returncode':process.returncode,'timed_out':timed_out})+'\n')
        answer_path = Path(scratch)/'answer.json'
        answer = answer_path.read_text() if answer_path.exists() else ''
        (receipt/'answer.json').write_text(answer)
        if timed_out or process.returncode:
            try:
                events = [json.loads(line) for line in stdout.splitlines() if line.strip()]
            except ValueError:
                events = []
            raise TransportFailure('Codex transport unavailable or timed out; see transport receipts', event_usage(events))
        return parse_events(stdout,answer,args.answer_bytes)


</experiment:copy>

## Require the existing ChatGPT sign-in before a paid turn

The local status check refuses API-key authentication and records the installed CLI version without starting inference.

<experiment:copy id="prediction-experiment-codex_backend-py-4">
def preflight():
    if sys.platform != 'darwin':
        raise RuntimeError('Subscription transport currently requires macOS')
    binary = shutil.which('codex')
    if not binary:
        raise RuntimeError('Codex CLI is not installed')
    result = subprocess.run([binary,'login','status'],capture_output=True,text=True,
                            env=environment(), timeout=15)
    if result.returncode or 'ChatGPT' not in result.stdout+result.stderr:
        raise RuntimeError('Run codex login with ChatGPT; API-key authentication is not accepted')
    return subprocess.check_output([binary,'--version'],text=True).strip()
</experiment:copy>

<experiment:file path="experiments/prediction-gate/codex_backend.py"><experiment:paste select="#prediction-experiment-codex_backend-py-0"/><experiment:paste select="#prediction-experiment-codex_backend-py-1"/><experiment:paste select="#prediction-experiment-codex_backend-py-2"/><experiment:paste select="#prediction-experiment-codex_backend-py-3"/><experiment:paste select="#prediction-experiment-codex_backend-py-4"/></experiment:file>

## Keep the protocol beside the implementation

The runbook records the frozen outcomes and evidence limits. Embedded shell syntax is retained literally.

<experiment:entry path="experiments/prediction-gate/README.md">
# Evaluate prediction gating without confusing it with correctness

For Hickory maintainers deciding whether to fund a controlled model experiment.

The local harness exercises the existing private-output runner and durable ledger.
It adds a host policy that accepts a benchmark save only after certification.
The production mechanism itself permits source writes before prediction checking;
calling it a universal “cannot save code” guardrail overstates its current scope.
A valid subscription-backed smoke pilot completed on 2026-11-03: both conditions
passed 3/3 tasks on the first proposal, with no mismatches or blocked saves. It
established harness operation but showed no correctness difference on those tasks.
The user then requested the separate [reactive-v4 benchmark](reactive-plan.md).
Simulated records remain plumbing evidence only; do not pool benchmark versions.

## What the existing implementation does

An agent predicts an observable value for a host-selected question, using an
`equals`, `number_at_most`, or nested `fields` predicate and a rationale. A target
can be one execution's output, a debugger expression at a source span, frame and
hit count, or both together. The rationale is retained text; the checker does not
score its quality. See [predicate and target definitions](../../crates/hick-observe/src/prediction_types.rs)
and [private session setup](../../crates/hickory-cli/src/serve/private_predictions.rs).

The host owns the scenario, concrete inputs, original requirement, actor and
captured environment. Sealing records exact SHA-256 identities for source,
inputs, dependencies and environment before execution admission. A repair needs
a new source-bound seal. The original prediction and requirement survive source
revisions and sessions within the same recognized scenario. Reusing an admission
returns its earlier result rather than reexecuting. The
[ledger classification](../../crates/hick-observe/src/prediction.rs) compares the
actual value separately against the sealed prediction and original requirement.
The [output runner](../../crates/hick-literate/src/prediction.rs) also requires
accepted execution evidence; matching a prediction cannot override a failed
`hick:expect`. Debugger and combined adapters share the gate.

A mismatch, failed original requirement, rejected execution, changed prediction,
changed requirement, stale identities, replay, unknown provenance, interruption,
or unavailable observation cannot earn current certification. Corrections that
retain the original expectation can pass with a distinct after-correction label.
Retained debugger coaching also distinguishes prediction before launch from an
informed prediction of the next stop after earlier state was shown.
[Promotion](../../crates/hickory-cli/src/serve/private_predictions.rs) is a separate
host action that rechecks current source/runtime and required debugger questions.

The [source edit tool](../../crates/hickory-agent/src/tools/mod.rs) writes edited
Markdown after parse/weave checks, before prediction verification. The
[private verify tool](../../crates/hickory-agent/src/tools/prediction.rs) returns
bounded summaries and publishes no generated outputs even on success. Ordinary
[verified publication](../../crates/hickory-agent/src/tools/verify.rs) is another
host policy and explicitly is not private prediction mode. Private service status
labels application integration incomplete. This inspection does not establish a
universal filesystem save barrier.

## Can the model obtain the answer first?

Through the admitted private tools, it can read/edit source, seal and verify.
[Dispatch](../../crates/hickory-agent/src/tools/dispatch.rs) refuses generated
output/file reads and alternate execution; private startup avoids ordinary cached
transcripts. Raw values, diagnostics and execution files are withheld, and the
host keeps ledger storage private. Managed inspection records exposure, and
changing scenario names/source/session does not legitimately reset known history.
These controls depend on the host's scenario correspondence and isolation.
They are not an OS sandbox against a same-user process. External tools, an
incorrectly confined ACP adapter, filesystem access, old conversations or leaked
host metadata can defeat the intended withholding; the repository itself records
integration and adapter evidence limits in the
[current contract](../../.hickory-source/guarantees/verification/predictions-precede-managed-observation/contract.md).

The status endpoint serializes the full host question, including its requirement
predicate; an equality requirement therefore exposes its expected value to any
allowed consumer of that status. Whether it reaches a particular model depends
on that host surface. The expected behavior can also be explicit in source `expect`
text, task requirements, examples or coach question wording. For example the
[private UI fixture](../../scripts/test-private-prediction-ui.sh) states the total
100ms budget and second allowance 20ms in its requirement references. That is a
source inspection finding, not proof of leakage in every question. The API does
not require an exact numerical forecast: a very permissive `number_at_most`
predicate is allowed. Repeating a requirement bound can earn a prediction match
without explaining the trajectory. `fields` enforces exactly the named field set,
but still permits broad subpredicates.

Repeated known tasks invite memorization. Small fixed input sets invite hardcoded
solutions. Pass/fail summaries are an information channel even when actual values
are withheld. Repairs can fit those cases, and changed concrete inputs create
independent scenarios. Unknown prior correspondence must remain unknown rather
than silently earning first-attempt credit. The ledger resists changing the
original expectation after feedback; it cannot establish understanding. A correct
implementation can also be blocked indefinitely by a mistaken original prediction.

## Frozen experiment v1

Freeze this document, `cases.py`, runner and bridge hashes before the first live
call. The manifest records them, revision, dirty status, settings, randomized
order and executable hash. Preserve a snapshot of the working revision when dirty.
Do not change cases, scoring, exclusions or success criteria after live outcomes.
Repairs to experiment bugs require a new version and rerunning both conditions;
retain failed version records. The current task oracles are fully enumerated,
not randomly generated during scoring.

Primary outcome: the fraction of all scheduled sessions whose final accepted
implementation passes every held-out test. All scheduled failures, timeouts,
abandonments and protocol errors remain in the denominator. Use paired gate minus
baseline risk difference across the same task/repeat. Predefined success requires
an improvement of at least 10 percentage points and a paired bootstrap 95% lower
bound above zero. Otherwise report inconclusive, unless a separately designed
noninferiority/equivalence study can support a claim of no useful benefit.

Report raw numerators/denominators, first proposed patch correctness before any
test or submit feedback, completed tasks, regressions against initially passing hidden
checks, prediction mismatches, blocked saves, retries, abandonments, and both
false passes and false blocks relative to held-out correctness. “Completed” means
an accepted submission passes public checks; it does not imply held-out success.
First-submission correctness is recorded separately because a submission may
follow ordinary test feedback; transcripts disclose that exposure. A test action cannot certify or save code.

Three tasks cover a retry budget boundary across helper/caller functions, inventory
reservations with failed-operation state preservation, and cache expiry/replacement
with lookup/cleanup agreement. Each has buggy starting code, explicit requirements,
1–2 public cases and four held-out cases. The fixtures also specify initial
regression cases. Public and held-out inputs do not overlap. The small deterministic
suite is intentionally inspectable by evaluators and weak against memorization.

Both conditions get identical task text, starting code, model, temperature 0,
JSON tool protocol, restricted Python subset, public cases, six calls, 2,048 output
tokens per call, a cumulative 32,000 UTF-8 input-byte cap and 180-second loop budget.
Provider HTTP calls time out at 30 seconds, gate operations at 15 seconds and
candidate executions at 3 seconds; a session can overrun its loop deadline by one
in-flight operation. Input bytes conservatively budget tokens; provider usage
counters supply measured tokens. Missing usage and cost are null, never zero.

The baseline can test and submit, sees public case-group pass/fail, and can revise on the same
call budget. Its submissions save even on public failure. The gate additionally
requires a sealed exact result-array prediction and rationale before every test
or submission execution, returns the existing runner's bounded summary, and accepts saves only
on production ledger promotion. Both see the same public case-group pass/fail; the gate
also exposes its prediction status. Gate overhead counts against the shared time
budget. Asking for a prediction necessarily changes that instruction and response
payload; there are no other intentional differences.

Each arm is a fresh HTTP conversation and directory; no previous arm history is
sent. Execution order is shuffled with seed 20261103 across all task/repeat/arm
entries. Model weights/provider caching and external provider state cannot be
reset by this harness. Pair labels are replication blocks, not matched random
samples from the provider. Keep provider/model version fixed and record changes.
There is no shell, repository or file tool in either arm. Only public JSON context
is sent. Candidate AST admission excludes imports, reflection, private names and
arbitrary attributes; execution uses a separate interpreter with a timeout.
This restricted trusted-fixture evaluator is not suitable for hostile arbitrary
programs or an unconstrained coding agent.

Held-out cases, solutions, expected values, scores and host diagnostics stay in
the evaluator process/files and never enter model context. The evaluator imports
the fixture module; the model receives only selected public fields. A candidate
has no supported filesystem access. Stored trial artifacts contain held-out
answers: keep the entire output directory evaluator-only during future runs.
This harness evaluates the production **output adapter**, not debugger coaching,
ACP, editor UI, arbitrary projects or the complete ordinary AI coding workflow.
Generalizing to those surfaces requires a separate integrated experiment. The
user's requested harness is an explicit experiment; it does not replace the
in-app opt-in measurements for indexes and lineage.

## Trial count and cost before live use

First run the simulated pilot below. Then run a separate three-pair live plumbing
pilot (one repeat of each task) to check provider protocol/accounting; retain it
as pilot-only evidence. Main experiment: eight
repeats per task, 24 pairs, 48 independent sessions. Keep pilot and main output
directories separate. This small study estimates performance on three tasks;
replicates do not create 24 independent task types or strong generalization power.

At illustrative rates $2/M input and $8/M output, the conservative main cap is
48 × (32,000 × 2 + 6 × 2,048 × 8) / 1,000,000 = **$7.79**; a three-pair live pilot
adds at most **$0.97**, excluding provider-specific billing extras. These are
budget assumptions, not verified prices for any named model. Choose a fixed model
and replace the rate flags with its actual rates before approval or execution.
No credentials are read or paid requests made in simulated mode. Live mode is
explicit and requires a configured endpoint, model and credential variable.

## Reproduce

From the repository root, build only the experiment adapter and run focused checks:

```sh
cargo build --locked -p hick-literate --example prediction_bench
python3 -m unittest discover -s experiments/prediction-gate -p 'test_bench.py' -v
python3 experiments/prediction-gate/bench.py --simulated --out target/prediction-gate-pilot
```

The output directory must be new. It retains `manifest.json`, `records.jsonl`,
`analysis.json`, `results.md` and per-session initial/final code, final patch,
full contexts, each proposal patch, original committed prediction, production
ledger, check results, revisions and final held-out results. A gate commitment
file is written before any observations. The production ledger provides durable
ordering; the host records are review artifacts, not tamper-proof attestations.
There is no raw provider chain-of-thought capture; rationale is whatever the
model elects to return.

A compatible provider must implement chat-completions JSON requests and return
`choices[0].message.content` plus `usage.prompt_tokens/completion_tokens`. The
bridge uses the existing Rust private-output runner; no Python replica decides
whether a prediction matches. With the chosen provider/model and real rates:

```sh
export BENCH_API_KEY='YOUR_KEY'
python3 experiments/prediction-gate/bench.py --live \
  --provider-url "$BENCH_PROVIDER_URL" --model "$BENCH_MODEL" \
  --input-rate 2 --output-rate 8 --repeats 1 --out target/prediction-gate-live-pilot
# After validating the pilot protocol, use --repeats 8 and a new main directory.
```

A live model is not assumed available merely because the local Codex session
works. A provider protocol failure is retained, not automatically retried or
replaced with a simulated response. Hidden scores are never returned as repair
feedback. During a run inspect only evaluator-owned artifacts; do not feed them
back to the agent. Regenerate maintained files with `behavior.py generate`, then
`check` and `audit`. Broad regression and post-commit monitoring belong to the
maintainer (`just checks-status`).

## Use subscription allowance through signed-in Codex (protocol 3)

This macOS transport uses `codex exec` with your existing ChatGPT sign-in. It
forces ChatGPT authentication and removes API-key overrides from the child
process; it does not extract or repurpose OAuth tokens. No API key is required.
See [signed-in automation](https://learn.chatgpt.com/docs/non-interactive-mode)
and [subscription usage and credit rates](https://learn.chatgpt.com/docs/pricing).
Included usage and available ChatGPT credits follow your account's limits; this
transport does not claim access to your balance or permission to buy more credits.

From a normal Terminal with working provider connectivity:

```sh
codex login status
python3 experiments/prediction-gate/bench.py --live --backend codex \
  --model gpt-6.1-sol --reasoning low --repeats 1 \
  --out target/prediction-gate-subscription-pilot
```

If login status does not say ChatGPT, run `codex login` first. The output directory
must be new. Verify the three-pair live pilot for authentication, valid JSON,
usage counters, absence of native tools and no infrastructure failures before
running eight repeats in a separate main directory. Keep these records separate
from API-v1 and simulated runs. DNS/HTTP failures are infrastructure failures,
never evidence about prediction efficacy.

Protocol 3 repairs the protocol-2 transport parser; tasks, held-out oracles,
paired sample size and success criteria remain unchanged. The prior live pilot
is retained as an invalid infrastructure run, not reused as valid trial evidence. Both arms use the same explicit
model, reasoning effort, CLI version, wrapper instructions and confinement. Each
response starts a fresh ephemeral thread in an empty directory and receives only
replayed public conversation text. User config, repository instructions, memory,
apps, browser, shell, web and agent tools are disabled. Native tool/item events
invalidate the trial. macOS Seatbelt additionally denies reads of the repository,
receipts, temp roots containing other evaluator copies, and Codex session/history
files; the empty scratch directory is readable. Focused tests demonstrate denial
of known held-out paths. This is a scoped experiment quarantine, not a claim of
universal hostile-process isolation.

The CLI adds its own instruction/tool overhead and cannot apply v1's temperature
or 2,048-generation-token cap. Protocol 3 instead shares a six-turn limit, 30-second
transport timeout per turn, 180-second session loop budget, 32,000-byte replayed
input cap and a 16,000-byte accepted final-answer limit across arms. The answer
limit is checked after generation; it does not cap hidden reasoning or billing.
Terminating a timed-out CLI process is not a guarantee that provider processing
stops immediately. Provider-visible token usage includes CLI overhead and is
recorded from its completion event. Missing counters remain null. CLI turn attempts
are counted separately; opaque internal model-call/retry counts remain unknown.

At the published standard GPT-6.1 Sol rates (50 input, 2.5 cached input, 250 output
credits per million tokens), a planning scenario of 20,000 uncached input and
4,000 output tokens per turn is 2 credits/turn: up to 72 credits for 36 pilot
turns, or 576 credits for 288 main turns. This is an estimate, not an enforced
credit cap or a promise about included allowance. The actual turn count is often
smaller. No purchases or automatic API fallback are performed. `estimated_credits`
uses observed input/cache/output counts for this model; dollar cost remains null.
Source the published rate date when reporting, because rates can change.

### Protocol-2 pilot incident, 2026-11-03

The run in `target/prediction-gate-subscription-pilot-20261103-200027` completed
six real Codex turns, then rejected all six before admitting their proposals.
Codex 0.166 encodes the expected unstable-feature and disabled-Code-Mode startup
warnings as `item.completed` entries whose item type is `error`. The parser
mistook them for native tool activity. Its zero correctness counts and zero-width
interval are unusable as effectiveness evidence. No gate comparison ran.
The original directory remains unchanged; no answers are retroactively scored.

Raw completion events retain 52,509 input tokens and 854 output tokens (no cached
input reported), approximately 2.84 credits at the documented standard Sol rates.
That is an estimate, not a verified account debit. The old trial rows lost these
counters when parsing raised; the repair preserves available usage on rejected
turns. Protocol 3 accepts only the two identified startup diagnostics, retains
them in receipts, and continues to refuse unknown errors, failed turns and actual
native tool items. Infrastructure-only runs now receive an INVALID RUN label and
null effectiveness estimates. Parser integrity checks exercise warning admission,
native-tool rejection and usage retention without loading/scoring model answers.
Rerun both conditions in a fresh directory; do not combine this pilot with the
new one or proceed directly to the main experiment.

## Analysis and reporting limits

`results.md` reports counts and the paired risk difference. `analysis.json` adds
10,000 seeded paired percentile bootstrap samples and a two-sided exact McNemar
p-value from discordant pairs. Report all failures and per-task breakdowns from
JSONL beside those numbers. Percentile bootstrap intervals can collapse with
few/no discordances, especially in a tiny pilot; a zero-width interval then is
not proof of equivalence. Treat tasks as fixed; repeated-run intervals do not
estimate uncertainty over new task populations. Model API nondeterminism persists
at temperature zero. Token comparisons include prediction/context overhead.

False-pass means a certified proposal fails any held-out case; false-block means
a refused proposal passes all held-out cases. Those operational labels are
relative to this suite, not universal semantic judgments. Final accepted code can
differ from final proposed code. Compare first submission, revision sequence and
accepted final results to separate initial reasoning from feedback-assisted repair.
An incorrect original expectation can permanently block a later correct patch;
keep that failure rather than quietly resealing a new scenario.

Passing this benchmark demonstrates behavioral performance on these tasks. It
cannot prove the model possesses a mental model, establish causal cognitive
mechanisms, or show that real Hickory users benefit on representative work.

## Blog-post outline

1. Hypothesis: committing observable forecasts may improve implementation
   correctness; explicit alternatives are extra deliberation, filtering and test feedback.
2. Implementation: show seal → admission → withheld comparison → promotion,
   distinguish source saves from certification, and show requirement/prediction separation.
3. Experiment: publish the frozen hashes, task population, randomization, matched
   budgets, tools, held-out separation, sample size, prices and primary criterion.
4. Results: lead with real sample size and raw counts; show final/first-attempt
   accuracy, paired difference/uncertainty, costs, regressions and every failure.
   Keep simulated plumbing receipts in a separate clearly labeled appendix.
5. Examples: a retry-budget repair with unchanged prediction, an overfit constant
   that passes public checks but fails held-out tests, and correct code blocked
   by a mistaken original prediction. Use archived patches/receipts, not invented runs.
6. Limitations: three fixed Python tasks, constrained tools, visible requirement
   clues, provider state, feedback and selection effects, output-adapter scope,
   weak task generalization, and behavior versus claims about understanding.
7. Next study: preregister new tasks and integrated Hickory sessions before
   looking at their results; never retrofit success criteria to the winner.
</experiment:entry>

</experiment:doc>
