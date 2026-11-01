"""Information parity, balance and bounded-resume witnesses for screening."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from hickory_agent_worker.core import ModelResponse
from hickory_agent_worker.factorial_study import balanced_case, conditions, context_blocks, main, render, schedule

class FactorialTests(unittest.TestCase):
    def test_target_positions_are_balanced_and_all_cells_are_scheduled(self):
        for kind in ('run', 'debug', 'profile'):
            positions = []
            for family in range(32):
                case = balanced_case(family, kind, 8)
                target = next(a for a in case['actions'] if a['handle'] == case['expected'])
                positions.append(next(i for i, s in enumerate(case['sources']) if s['path'] == target['path']))
            self.assertEqual([positions.count(i) for i in range(8)], [4] * 8)
        jobs = schedule(8, 8, 2)
        self.assertEqual(jobs, schedule(8, 8, 2))
        self.assertEqual(len(jobs), 8 * 3 * 2 * 16)
        for family in range(8):
            for kind in ('run', 'debug', 'profile'):
                for repeat in range(2):
                    self.assertCountEqual([j['condition'] for j in jobs if (j['family'], j['kind'], j['repeat']) == (family, kind, repeat)], range(16))
        self.assertNotEqual([j['condition'] for j in jobs[:16]], list(range(16)))

    def test_placement_and_format_keep_facts_but_pruning_removes_only_old_revisions(self):
        case = balanced_case(7, 'debug', 8)
        for prune in (False, True):
            baseline = None
            for condition in conditions():
                if condition['prune_history'] != prune:
                    continue
                blocks = context_blocks(case, condition)
                facts = sorted(json.dumps([k, d], sort_keys=True) for k, d in blocks)
                if baseline is None:
                    baseline = facts
                self.assertEqual(facts, baseline)
                reading = render(case, condition)
                self.assertEqual(reading.count(case['expected']), 1)
                self.assertNotIn('expected', reading)
                for kind, data in blocks:
                    if kind == 'context' and prune:
                        self.assertFalse(data['source_revision'].startswith('older-'))
                self.assertEqual(len(blocks), 24 if prune else 40)
        invalid = conditions()[0] | {'markdown': 1}
        with self.assertRaises(ValueError):
            context_blocks(case, invalid)

    def test_call_budget_and_resume_keep_completed_trials_and_record_errors(self):
        class Model:
            identity = {'installed_digest': 'fixed-test-model'}
            def complete(self, messages):
                raise RuntimeError('transport unavailable')
        with tempfile.TemporaryDirectory() as directory, patch('hickory_agent_worker.factorial_study.open_model', return_value=Model()) as opened, patch('hickory_agent_worker.factorial_study.subprocess.run'):
            args = ['--output', directory, '--families', '1', '--max-calls', '1']
            main(args)
            main(args)
            summary = json.loads((Path(directory) / 'summary.json').read_text())
            self.assertEqual(summary['completed'], 2)
            self.assertEqual(sum(c['transport_errors'] for c in summary['conditions'].values()), 2)
            self.assertEqual(opened.call_count, 2)
            with self.assertRaisesRegex(ValueError, 'protocol/source change'):
                main(args + ['--cells', '4'])

    def test_plan_only_never_loads_model_and_cutoffs_cannot_pass(self):
        from hickory_agent_worker.context_study import score
        with tempfile.TemporaryDirectory() as directory, patch('hickory_agent_worker.factorial_study.open_model') as opened:
            main(['--output', directory, '--plan-only'])
            opened.assert_not_called()
            self.assertEqual(json.loads((Path(directory) / 'summary.json').read_text())['planned'], 4608)
        case = balanced_case(2, 'run', 8)
        self.assertFalse(score(case, ModelResponse(json.dumps({'handle': case['expected']}), 'length'))['correct'])


if __name__ == '__main__':
    unittest.main()


