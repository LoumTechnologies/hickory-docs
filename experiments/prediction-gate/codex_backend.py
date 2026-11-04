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
