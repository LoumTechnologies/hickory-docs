"""Guarantee: docs/guarantees/agent/refresh-experiments-preserve-evidence.md."""
import unittest
from copy import deepcopy
from hickory_agent_worker.context import MemoryContext
from hickory_agent_worker.refresh import RefreshPolicy, Update
from hickory_agent_worker.refresh_experiment import run, compare_observations


class RefreshTests(unittest.TestCase):
    def test_paired_schedules_share_tasks_and_preserve_receipts(self):
        report = run(2)
        self.assertEqual(len(report['runs']), 96)
        self.assertTrue(all(row['behavioral_success'] for row in report['runs']))
        self.assertTrue(all(row['reported_cost'] is None for row in report['runs']))
        self.assertEqual({row['encoding'] for row in report['runs']},
                         {'append', 'projection', 'reread'})

    def test_projection_never_promotes_role_or_rewrites_tool_messages(self):
        policy = RefreshPolicy('projection')
        context = MemoryContext(policy=policy)
        messages = [dict(role='system', content='old'),
                    dict(role='user', content='old', tool_call_id='call-1'),
                    dict(role='user', content='old')]
        frozen = deepcopy(messages)
        receipt = context.prepare([dict(role='user', content='old')])
        old = deepcopy(receipt)
        policy.queue_update(Update('source', 'old', 'new', 'rev2'))
        candidate = policy.assemble(context, messages)
        self.assertEqual(messages, frozen)
        self.assertEqual(receipt, old)
        parts = candidate['request']['parts']
        self.assertEqual(parts[0]['text'], 'old')
        self.assertEqual(parts[1]['text'], 'old')
        self.assertEqual(parts[2]['text'], 'new')
        self.assertEqual(parts[-1]['role'], 'user')

    def test_representation_dimension_is_fixed_policy_and_missing_metrics_null(self):
        report = compare_observations([dict(id='profile')],
                    lambda case, representation: dict(behavioral_success=True))
        self.assertEqual([r['representation'] for r in report],
                         ['coordinates', 'annotations-handles'])
        self.assertTrue(all(row['refresh_policy'] == 'fixed' for row in report))
        self.assertTrue(all(row['mapping_errors'] is None for row in report))


if __name__ == '__main__':
    unittest.main()
