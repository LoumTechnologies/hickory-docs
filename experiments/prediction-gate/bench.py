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
