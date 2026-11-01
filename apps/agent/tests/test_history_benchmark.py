import copy
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
from hickory_agent_worker.core import ModelResponse
from hickory_agent_worker import history_benchmark as study

def fixture():
    return dict(id='sample',repository='nloum/sample',kind='completion',author=dict(author_date='2020-12-31T23:00:00Z',committer_date='2020-12-31T23:00:00Z'),answer='private int Answer(){return 4;}',answer_sha256=study.digest('private int Answer(){return 4;}'),before='private int Answer(){throw new Exception();}',usings='using System;\n',support='// read-only collaborator',assertions='// assertions',public_tests='// PUBLIC_EXAMPLE',hidden_tests='// SECRET_HELD_OUT',prompt='Complete the method',commit_message='SECRET_COMMIT_MESSAGE')

class HistoricalBenchmarkTests(unittest.TestCase):
    def test_model_reading_has_same_information_without_oracles(self):
        case=fixture()
        a=study.reading(case,case['before'],'CURRENT_FAILURE','literate')
        b=study.reading(case,case['before'],'CURRENT_FAILURE','separate')
        self.assertNotEqual(a,b)
        self.assertCountEqual(a.splitlines(),b.splitlines())
        self.assertEqual(len(a.encode()),len(b.encode()))
        for text in (a,b):
            self.assertIn('CURRENT_FAILURE',text)
            self.assertIn('PUBLIC_EXAMPLE',text)
            self.assertNotIn('SECRET_HELD_OUT',text)
            self.assertNotIn('SECRET_COMMIT_MESSAGE',text)
            self.assertNotIn(case['answer'],text)

    def test_public_program_never_embeds_held_out_checks(self):
        case=fixture()
        self.assertNotIn('SECRET_HELD_OUT',study.program(case,case['before'],'public'))
        self.assertIn('SECRET_HELD_OUT',study.program(case,case['answer'],'hidden'))
        with self.assertRaises(ValueError):study.program(case,'','unknown')

    def test_references_are_bound_to_fixture_and_discriminate_missing_code(self):
        case=fixture();data=dict(cases=[case]);row=dict(case='sample',eligible=True,fixture_sha256=study.fixture_fingerprint(case),reference_public=dict(passed=True),reference_hidden=dict(passed=True),before_public=dict(passed=False))
        study.validate_references(data,[row])
        changed=copy.deepcopy(case);changed['hidden_tests']+=' changed'
        with self.assertRaises(ValueError):study.validate_references(dict(cases=[changed]),[row])
        row['before_public']['passed']=True
        with self.assertRaises(ValueError):study.validate_references(data,[row])

    def test_dates_and_answer_integrity_are_checked_before_inference(self):
        data=dict(version=1,cutoff_exclusive='2021-01-01',cases=[fixture()])
        with tempfile.TemporaryDirectory() as root:
            path=Path(root)/'dataset.json';path.write_text(json.dumps(data))
            study.load_dataset(path)
            data['cases'][0]['author']['committer_date']='2021-01-01T00:00:00Z';path.write_text(json.dumps(data))
            with self.assertRaises(ValueError):study.load_dataset(path)
            data['cases'][0]=fixture();data['cases'][0]['answer']+=' changed';path.write_text(json.dumps(data))
            with self.assertRaises(ValueError):study.load_dataset(path)

    def test_malformed_documents_are_refused(self):
        valid='Explanation\n```csharp\npublic int Value(){return 4;}\n```\n'
        self.assertIn('return 4',study.candidate_method(valid))
        for text in ('public int Value(){return 4;}',valid+valid,'```csharp\n\n```','x'*32769):
            with self.assertRaises(ValueError):study.candidate_method(text)

    def test_truncated_reply_is_never_compiled_or_executed(self):
        case=fixture();data=dict(cases=[case]);rows=[dict(case='sample',eligible=True,fixture_sha256=study.fixture_fingerprint(case),reference_public=dict(passed=True),reference_hidden=dict(passed=True),before_public=dict(passed=False,stage='execute',run=dict(output='NotImplementedException')))]
        fake=SimpleNamespace(identity={},complete=lambda messages:ModelResponse('```csharp\nprivate int Answer(){return 4;}\n```','length',{}))
        with tempfile.TemporaryDirectory() as root,patch.object(study,'OllamaModel',return_value=fake),patch.object(study,'image_identity',return_value={}),patch.object(study,'grade') as grade,patch.object(study.subprocess,'run'):
            result=study.experiment(data,rows,SimpleNamespace(output=Path(root),models=['fake'],iterations=1,layout='literate'))
            grade.assert_not_called()
            self.assertFalse(result[0]['public_passed'])
            self.assertFalse(result[0]['hidden_passed'])
            self.assertIn('truncated',result[0]['calls'][0]['error'])


    def test_memory_reclamation_wait_preserves_the_headroom_guard(self):
        ready=object()
        error=RuntimeError('Insufficient available memory for this model plus context headroom')
        with patch.object(study,'OllamaModel',side_effect=[error,ready]) as constructor,patch.object(study.time,'monotonic',side_effect=[0,1]),patch.object(study.time,'sleep') as sleep:
            self.assertIs(study.open_model('local',object()),ready)
            self.assertEqual(constructor.call_count,2)
            sleep.assert_called_once_with(0.25)
        with patch.object(study,'OllamaModel',side_effect=error),patch.object(study.time,'monotonic',side_effect=[0,31]),patch.object(study.time,'sleep') as sleep:
            with self.assertRaises(RuntimeError):study.open_model('local',object())
            sleep.assert_not_called()

    def test_unrelated_model_errors_are_not_retried(self):
        with patch.object(study,'OllamaModel',side_effect=RuntimeError('different transport failure')) as constructor,patch.object(study.time,'sleep') as sleep:
            with self.assertRaisesRegex(RuntimeError,'different transport'):
                study.open_model('local',object())
            constructor.assert_called_once()
            sleep.assert_not_called()


if __name__=='__main__':
    unittest.main()


