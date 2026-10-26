<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/agent/tests/test_worker.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9f33e52791d8"></a>
## `apps/agent/tests/test_worker.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Separate model snippets from trusted host operations](../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md), [Explicit UI action dispatch checks scope, state, policy and retries](../../../../guarantees/agent/ui-action-dispatch/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../guarantees/verification/predictions-precede-managed-observation/contract.md), [Private edit proposals open review before writing](../../../../guarantees/verification/private-proposals-wait-for-review/contract.md).

<a id="f-9f33e52791d8-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9f33e52791d8-expression_statement">
"""Guarantees: python-execution-has-two-boundaries.md;
changeset-completion-is-negotiated-and-checked.md. No Hickory process or model API.
"""
</storycode:copy>
<a id="f-9f33e52791d8-io"></a>
### `io`

<storycode:copy id="f-9f33e52791d8-io">
import io
</storycode:copy>
<a id="f-9f33e52791d8-json"></a>
### `json`

<storycode:copy id="f-9f33e52791d8-json">
import json
</storycode:copy>
<a id="f-9f33e52791d8-subprocess"></a>
### `subprocess`

<storycode:copy id="f-9f33e52791d8-subprocess">
import subprocess
</storycode:copy>
<a id="f-9f33e52791d8-sys"></a>
### `sys`

<storycode:copy id="f-9f33e52791d8-sys">
import sys
</storycode:copy>
<a id="f-9f33e52791d8-tempfile"></a>
### `tempfile`

<storycode:copy id="f-9f33e52791d8-tempfile">
import tempfile
</storycode:copy>
<a id="f-9f33e52791d8-unittest"></a>
### `unittest`

<storycode:copy id="f-9f33e52791d8-unittest">
import unittest
</storycode:copy>
<a id="f-9f33e52791d8-Path"></a>
### `Path`

<storycode:copy id="f-9f33e52791d8-Path">
from pathlib import Path

</storycode:copy>
<a id="f-9f33e52791d8-parse_action"></a>
### `parse_action`

<storycode:copy id="f-9f33e52791d8-parse_action">
from hickory_agent_worker.actions import parse_action
</storycode:copy>
<a id="f-9f33e52791d8-Broker"></a>
### `Broker`

<storycode:copy id="f-9f33e52791d8-Broker">
from hickory_agent_worker.runtime import Broker, execute
</storycode:copy>
<a id="f-9f33e52791d8-Agent"></a>
### `Agent`

<storycode:copy id="f-9f33e52791d8-Agent">
from hickory_agent_worker.core import Agent, Config
</storycode:copy>
<a id="f-9f33e52791d8-ReplayModel"></a>
### `ReplayModel`

<storycode:copy id="f-9f33e52791d8-ReplayModel">
from hickory_agent_worker.cli import ReplayModel, DevelopmentTools, ToolBridge
</storycode:copy>
<a id="f-9f33e52791d8-monty"></a>
### `monty`

<storycode:copy id="f-9f33e52791d8-monty">
from hickory_agent_worker.hosted import monty, LIMITS


</storycode:copy>
<a id="f-9f33e52791d8-Tools"></a>
### `Tools`

<storycode:copy id="f-9f33e52791d8-Tools">
class Tools:
    def __init__(self, accepted=True):
        self.accepted = accepted
        self.completions = []

    def complete(self, summary):
        self.completions.append(summary)
        return {'accepted': self.accepted, 'error': 'No current accepted changeset'}


</storycode:copy>
<a id="f-9f33e52791d8-WorkerTests"></a>
### `WorkerTests`

<storycode:copy id="f-9f33e52791d8-WorkerTests">
class WorkerTests(unittest.TestCase):
    def test_only_one_declared_python_or_done_action(self):
        self.assertEqual(parse_action('{"python":"1 + 2"}'), {'python': '1 + 2'})
        for raw in ['{"shell":"rm -rf ."}', '{"python":"x","done":"ok"}', '[]', '```python\nx\n```']:
            with self.assertRaises(ValueError):
                parse_action(raw)

    # Guarantee: docs/guarantees/agent/ui-action-dispatch.md
    def test_model_protocol_repair_does_not_misreport_tool_execution(self):
        seen, observations = [], []
        class Model:
            def complete(self, messages):
                if not observations:
                    observations.append(None)
                    return 'I will inspect now.\n```json\n{"python":"tool(\\"list_docs\\", {})"}\n```'
                observation = json.loads(messages[-1]['content'].split(': ', 1)[1])
                observations.append(observation)
                return '{"done":"No tools were executed; protocol repaired"}'
        tools = Tools()
        agent = Agent(Model(), tools, lambda code: seen.append(code))
        self.assertEqual(agent.run(Config('Inspect fixture', max_turns=2)),
                         'No tools were executed; protocol repaired')
        self.assertEqual(seen, [])
        self.assertEqual(observations[1]['kind'], 'model_protocol_error')
        self.assertFalse(observations[1]['tool_execution_started'])
        self.assertIn('ONLY one raw JSON object', observations[1]['repair'])
        self.assertEqual(len(tools.completions), 1)

    # Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
    def test_rejected_batch_runs_nothing_before_first_step_retry(self):
        seen = []
        class Model:
            calls = 0
            def complete(self, messages):
                self.calls += 1
                if self.calls == 1:
                    return '{"python":"read()"}\n{"python":"edit()"}'
                if self.calls == 2:
                    observation = json.loads(messages[-1]['content'].split(': ', 1)[1])
                    self_test.assertFalse(observation['tool_execution_started'])
                    self_test.assertIn('Every action in the rejected response is unexecuted', observation['repair'])
                    self_test.assertIn('first unfinished requested step', observation['repair'])
                    self_test.assertEqual(seen, [])
                    return '{"python":"read()"}'
                return '{"done":"Only the first requested step ran"}'
        self_test = self
        agent = Agent(Model(), Tools(), lambda code: seen.append(code))
        self.assertEqual(agent.run(Config('Read before proposing an edit', max_turns=3)),
                         'Only the first requested step ran')
        self.assertEqual(seen, ['read()'])

    def test_broker_rejects_wrong_id_and_oversized_frames(self):
        broker = Broker(io.StringIO('{"id":2,"result":"forged"}\n'), io.StringIO())
        with self.assertRaisesRegex(RuntimeError, 'Mismatched'):
            broker.call('inference', {})
        with self.assertRaisesRegex(RuntimeError, 'Oversized'):
            broker.send({'x': 'a' * 1_048_576})

    def test_monty_cannot_bypass_tool_review(self):
        with monty() as pool:
            with pool.checkout(limits={**LIMITS, 'max_memory': 2_000_000,
                                       'max_feed_duration_secs': 0.2}) as session:
                self.assertEqual(execute(session, None, '1 + 2')['result'], 3)
                for code in ["open('/etc/passwd').read()", "import subprocess\nsubprocess.run(['sh'])",
                             "eval(\"open('/etc/passwd').read()\")", "'x' * 10**10", 'while True:\n    pass']:
                    self.assertFalse(execute(session, None, code)['ok'], code)

    def test_python_selects_history_actions_and_stopping_without_a_host(self):
        records = []
        tools = Tools()
        class Model:
            def complete(self, messages):
                self.messages = list(messages)
                return json.dumps({'done': 'Independent answer'})
        model = Model()
        agent = Agent(model, tools, lambda code: self.fail('No snippet requested'),
                      lambda kind, data: records.append((kind, data)))
        self.assertEqual(agent.run(Config(prompt='Now', context='editor', document='note.md',
            prior_turns=[{'prompt': 'Earlier', 'answer': 'History'}])), 'Independent answer')
        self.assertIn('Monty', model.messages[0]['content'])
        self.assertIn('<hick:paste select=', model.messages[0]['content'])
        self.assertIn('read_output with_lineage', model.messages[0]['content'])
        self.assertIn('never recreate a dataset', model.messages[0]['content'])
        self.assertEqual([m['content'] for m in model.messages[1:3]], ['Earlier', 'History'])
        self.assertEqual(model.messages[-1]['content'], 'Now')
        self.assertEqual(tools.completions, ['Independent answer'])

    def test_rejected_completion_is_observed_and_python_repairs(self):
        tools = Tools(False)
        seen = []
        def snippet(code):
            seen.append(code)
            tools.accepted = True
            return {'ok': True, 'result': 'accepted artifact'}
        agent = Agent(ReplayModel([{'done': 'Premature'}, {'python': 'repair()'},
                                  {'done': 'Repaired'}]), tools, snippet)
        self.assertEqual(agent.run(Config('Task', max_turns=3)), 'Repaired')
        self.assertEqual(seen, ['repair()'])
        self.assertTrue(any('No current accepted' in m['content'] for m in agent.messages))

    def test_failed_tool_is_observed_not_success_evidence(self):
        with tempfile.TemporaryDirectory() as root, monty() as pool:
            tools = DevelopmentTools(root)
            with pool.checkout(limits=LIMITS) as session:
                agent = Agent(ReplayModel([{'python': "tool('verify', {})"}, {'done': 'Refused'}]),
                    tools, lambda code: execute(session, ToolBridge(tools), code))
                self.assertEqual(agent.run(Config('Task')), 'Refused')
                self.assertTrue(any('unavailable' in m['content'] for m in agent.messages))
                self.assertEqual(list(Path(root).iterdir()), [])

    def test_python_owns_retry_and_iteration_policy(self):
        class Flaky:
            calls = 0
            def complete(self, messages):
                self.calls += 1
                if self.calls == 1:
                    raise RuntimeError('Transient inference transport error')
                return '{"done":"ok"}'
        model = Flaky()
        self.assertEqual(Agent(model, Tools(), None).run(Config('Task')), 'ok')
        self.assertEqual(model.calls, 2)
        with self.assertRaisesRegex(RuntimeError, 'iteration limit'):
            Agent(ReplayModel([{'done': 'Unverified'}]), Tools(False), None).run(Config('Task', max_turns=1))

    def test_cancellation_prevents_model_and_tool_calls(self):
        with self.assertRaisesRegex(RuntimeError, 'cancelled'):
            Agent(None, None, None, cancelled=lambda: True).run(Config('Task'))
        class Model:
            def complete(self, messages):
                cancelled[0] = True
                return '{"python":"tool(1, 2)"}'
        cancelled = [False]
        with self.assertRaisesRegex(RuntimeError, 'cancelled'):
            Agent(Model(), None, lambda code: self.fail('Acted after cancellation'),
                  cancelled=lambda: cancelled[0]).run(Config('Task'))

    def test_injected_tool_client_and_print_receipts(self):
        class FakeTools(Tools):
            def call(self, name, arguments):
                self.request = name, arguments
                return {'receipt': 'observed fixture result'}
        tools = FakeTools()
        with monty() as pool:
            with pool.checkout(limits=LIMITS) as session:
                observations = []
                agent = Agent(ReplayModel([{'python': "print(42)\ntool('read_doc', {'doc':'fixture.md'})"},
                                           {'done': 'Used injected client'}]), tools,
                    lambda code: execute(session, ToolBridge(tools), code),
                    lambda kind, data: observations.append((kind, data)))
                self.assertEqual(agent.run(Config('Task')), 'Used injected client')
                self.assertEqual(tools.request, ('read_doc', {'doc': 'fixture.md'}))
                result = next(data for kind, data in observations if kind == 'observation')
                self.assertEqual(result['result'], {'receipt': 'observed fixture result'})
                self.assertEqual(result['printed'], '42\n')
                self.assertEqual(''.join(event['text'] for event in result['print_events']), '42\n')
                self.assertTrue(all(event['stream'] == 'stdout' for event in result['print_events']))

    def test_standalone_cli_uses_temporary_workspace_and_no_rust(self):
        with tempfile.TemporaryDirectory() as root:
            responses = Path(root) / 'responses.json'
            responses.write_text(json.dumps([{'python': "tool('list_docs', {})"}, {'done': 'Standalone works'}]))
            result = subprocess.run([sys.executable, '-m', 'hickory_agent_worker.cli',
                '--responses', str(responses), 'Inspect fixtures'], text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(), 'Standalone works')
            self.assertIn('no hosted confinement', result.stderr)
            self.assertEqual(list(Path(root).iterdir()), [responses])


</storycode:copy>
<a id="f-9f33e52791d8-if_statement"></a>
### `if_statement`

<storycode:copy id="f-9f33e52791d8-if_statement">
if __name__ == '__main__':
    unittest.main()
</storycode:copy>

### Assemble `apps/agent/tests/test_worker.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/tests/test_worker.py"><storycode:paste select="#f-9f33e52791d8-expression_statement"/><storycode:paste select="#f-9f33e52791d8-io"/><storycode:paste select="#f-9f33e52791d8-json"/><storycode:paste select="#f-9f33e52791d8-subprocess"/><storycode:paste select="#f-9f33e52791d8-sys"/><storycode:paste select="#f-9f33e52791d8-tempfile"/><storycode:paste select="#f-9f33e52791d8-unittest"/><storycode:paste select="#f-9f33e52791d8-Path"/><storycode:paste select="#f-9f33e52791d8-parse_action"/><storycode:paste select="#f-9f33e52791d8-Broker"/><storycode:paste select="#f-9f33e52791d8-Agent"/><storycode:paste select="#f-9f33e52791d8-ReplayModel"/><storycode:paste select="#f-9f33e52791d8-monty"/><storycode:paste select="#f-9f33e52791d8-Tools"/><storycode:paste select="#f-9f33e52791d8-WorkerTests"/><storycode:paste select="#f-9f33e52791d8-if_statement"/></storycode:file>

</storycode:doc>
