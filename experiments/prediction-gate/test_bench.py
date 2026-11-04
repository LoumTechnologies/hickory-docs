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
