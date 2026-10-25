<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/tests/test_refresh_experiment.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Local refresh experiments preserve historical request evidence](../../../../contract.md)

Captured SHA-256: `054e44c3c6285c4c94900bf5bf1c3a312901daed143d564f74b6627b95d59bd6`.

<book:entry path="apps/agent/tests/test_refresh_experiment.py">
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
        self.assertTrue(all(row['exposure_success'] for row in report['runs']))
        self.assertTrue(all(row['behavioral_success'] is None for row in report['runs']))
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

    def test_stale_driver_fails_even_when_fresh_context_is_present(self):
        def stale(manifest, source, task):
            return dict(answer='original stale decision', assumed_revision='revision-1',
                        edit_attempts=[dict(revision='revision-1')])
        report = run(2, stale)
        self.assertTrue(all(row['exposure_success'] for row in report['runs']))
        self.assertTrue(all(row['behavioral_success'] is False for row in report['runs']))
        self.assertTrue(all(row['stale_assumptions'] == 1 for row in report['runs']))
        self.assertTrue(all(row['stale_edit_attempts'] == 1 for row in report['runs']))
        self.assertTrue(all(len(row['driver_observations']) == 2 for row in report['runs']))

    def test_driver_observes_isolated_source_and_unsaved_input_without_getting_expected_answer(self):
        from pathlib import Path
        seen = set()
        def current(manifest, source, task):
            self.assertNotIn('expected', task)
            self.assertNotIn('answer', task)
            seen.add(str(source.parent))
            answer = task.get('unsaved_buffer')
            if answer is None:
                answer = source.read_text() if source.exists() else None
            return dict(answer=answer, assumed_revision='revision-2', edit_attempts=[])
        report = run(2, current)
        self.assertTrue(all(row['behavioral_success'] for row in report['runs']))
        self.assertEqual(len(seen), 96)
        self.assertTrue(all(row['reported_cost'] is None for row in report['runs']))

    def test_missing_driver_answer_cannot_pass_a_deleted_source(self):
        from hickory_agent_worker.refresh_experiment import replay, command_driver
        import sys
        driver = command_driver([sys.executable, '-c', 'print("{}")'])
        row = replay('append', 'manual', ('deletion', 'before', None, 'unavailable'), 0, driver)
        self.assertFalse(row['behavioral_success'])


if __name__ == '__main__':
    unittest.main()
</book:entry>

</book:doc>
