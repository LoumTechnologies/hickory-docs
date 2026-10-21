"""Guarantees: python-execution-has-two-boundaries.md;
changeset-completion-is-negotiated-and-checked.md. No Hickory process or model API.
"""
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from hickory_agent_worker.actions import parse_action
from hickory_agent_worker.runtime import Broker, execute
from hickory_agent_worker.core import Agent, Config
from hickory_agent_worker.cli import ReplayModel, DevelopmentTools, ToolBridge
from hickory_agent_worker.hosted import monty, LIMITS


class Tools:
    def __init__(self, accepted=True):
        self.accepted = accepted
        self.completions = []

    def complete(self, summary):
        self.completions.append(summary)
        return {'accepted': self.accepted, 'error': 'No current accepted changeset'}


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


if __name__ == '__main__':
    unittest.main()
