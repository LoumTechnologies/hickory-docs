"""Guarantee: docs/guarantees/agent/python-requests-have-validated-context.md."""
import unittest
from copy import deepcopy
from hickory_agent_worker.context import MemoryContext, HostedContext, AppendUpdates
from hickory_agent_worker.context_protocol import validate_bundle, ProtocolFailure
from hickory_agent_worker.core import Agent, Config


class ContextRuntimeTests(unittest.TestCase):
    def test_custom_policy_identity_is_recorded(self):
        class CustomPolicy(AppendUpdates):
            name = 'experimental'
            version = '2'
        request = MemoryContext(policy=CustomPolicy()).prepare([])['request']
        self.assertEqual((request['policy'], request['policy_version']), ('experimental', '2'))

    def test_exact_unicode_roles_and_tamper_rejection(self):
        context = MemoryContext('branch-a')
        bundle = context.prepare([{'role': 'system', 'content': 'Rules\r\n😀'},
                                  {'role': 'user', 'content': 'é'}])
        self.assertEqual(bundle['request']['branch'], 'branch-a')
        modified = deepcopy(bundle)
        modified['request']['parts'][0]['text'] += '!'
        with self.assertRaises(ProtocolFailure):
            validate_bundle(modified)
        modified = deepcopy(bundle)
        modified['request']['parts'][0]['segments'] = []
        with self.assertRaises(ProtocolFailure):
            validate_bundle(modified)

    def test_update_during_inference_waits_for_next_boundary(self):
        context = MemoryContext()
        context.queue('Revision one')
        requests = []
        class Model:
            def complete_request(self, bundle):
                requests.append(deepcopy(bundle))
                if len(requests) == 1:
                    context.queue('Revision two')
                    return '{"python":"42"}'
                return '{"done":"ok"}'
        class Tools:
            def complete(self, summary):
                return {'accepted': True}
        Agent(Model(), Tools(), lambda code: {'result': 42}, context=context).run(Config('Task'))
        first = [p['text'] for p in requests[0]['request']['parts']]
        second = [p['text'] for p in requests[1]['request']['parts']]
        self.assertIn('Revision one', first)
        self.assertNotIn('Revision two', first)
        self.assertIn('Revision two', second)
        self.assertNotEqual(requests[0]['request']['id'], requests[1]['request']['id'])
        self.assertEqual(context.pending, [])

    def test_compaction_retains_declared_ancestry_without_exactness_claim(self):
        context = MemoryContext('child')
        parent = context.register('Original requirement')
        summary = context.register('Short summary', [parent['id']])
        bundle = context.prepare([{'role': 'user', 'content': 'Short summary'}])
        self.assertEqual(summary['coverage'], 'opaque')
        self.assertEqual(bundle['request']['parts'][0]['segments'][0]['node'], summary['id'])
        self.assertIn(parent, bundle['nodes'])

    def test_retry_pins_manifest_and_does_not_consume_late_updates(self):
        context = MemoryContext()
        seen = []
        class Model:
            def complete_request(self, bundle):
                seen.append(deepcopy(bundle))
                if len(seen) == 1:
                    context.queue('Changed during failed request')
                    raise RuntimeError('failed')
                return '{"done":"ok"}'
        class Tools:
            def complete(self, summary):
                return {'accepted': True}
        Agent(Model(), Tools(), None, context=context).run(Config('Task'))
        self.assertEqual(seen[0], seen[1])
        self.assertEqual(context.pending[0][0], 'Changed during failed request')

    def test_continuous_change_stream_gets_one_poll_per_boundary(self):
        class Broker:
            def __init__(self):
                self.polls = 0
            def call(self, method, params):
                cursor = {'epoch': 'epoch', 'branch': 'branch', 'position': self.polls}
                if method == 'context.hello':
                    return {'cursor': cursor}
                if method == 'context.changes':
                    self.polls += 1
                    cursor['position'] = self.polls
                    return {'next_cursor': cursor, 'has_more': True,
                            'events': [{'sequence': self.polls, 'state': 'changed'}]}
                raise AssertionError(method)
        broker = Broker()
        context = HostedContext(broker)
        messages = []
        context.boundary(messages)
        self.assertEqual(broker.polls, 1)
        self.assertEqual(len(messages), 1)
        self.assertIn('reread', messages[0]['content'])


if __name__ == '__main__':
    unittest.main()
