import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch
spec = importlib.util.spec_from_file_location('publisher', Path(__file__).with_name('publish-spec-results.py'))
publisher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publisher)

class PublicationTests(unittest.TestCase):
    # Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
    def test_prediction_projection_withholds_command_answers_and_requires_reviewed_export(self):
        recorded = json.loads((Path(__file__).resolve().parents[1] / 'examples/security-authorization/predictions.json').read_text())
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = b'# Selected source\n'
            history = json.dumps(recorded).encode()
            (root/'spec.md').write_bytes(source); (root/'predictions.json').write_bytes(history)
            selection = dict(version=1, documents=[dict(path='spec.md', reviewed_output=True, prediction_history='predictions.json', reviewed_inputs={'spec.md':publisher.digest(source), 'predictions.json':publisher.digest(history)})])
            def run(command, **kwargs):
                self.fail('Recorded prediction publication must not execute the held-out target')
            with patch.object(publisher.subprocess,'check_output',return_value='a'*40):
                publisher.publish(root,selection,root/'out',Path('/hick'),run)
                index=json.loads((root/'out/index.json').read_text())
                snapshot=json.loads((root/'out'/index['documents'][0]['snapshot']).read_text())
                self.assertEqual(snapshot['prediction_history'], recorded)
                self.assertEqual(snapshot['cells'], [])
                self.assertNotIn('PRIVATE HELD-OUT ANSWER',json.dumps(snapshot))
                self.assertNotIn('stdout',json.dumps(snapshot['verification']['commands']))
                (root/'predictions.json').write_text('{}')
                with self.assertRaises(ValueError): publisher.publish(root,selection,root/'changed',Path('/hick'),run)

    def test_recorded_authorization_pair_keeps_the_expected_failure_and_exact_source_identities(self):
        examples=Path(__file__).resolve().parents[1] / 'examples/security-authorization'
        history=(examples/'predictions.json').read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); (root/'predictions.json').write_bytes(history)
            documents=[]
            for name,status in [('vulnerable','failed'),('patched','passed')]:
                source=(examples/f'{name}.md').read_bytes(); path=f'{name}.md';(root/path).write_bytes(source)
                documents.append(dict(path=path, reviewed_output=True, prediction_history='predictions.json', expected_status=status,
                    expected_status_reason='Synthetic cross-tenant predicate must fail before the patch' if status=='failed' else '',
                    reviewed_inputs={path:publisher.digest(source),'predictions.json':publisher.digest(history)}))
            with patch.object(publisher.subprocess,'check_output',return_value='a'*40):
                self.assertTrue(publisher.publish(root,dict(version=1,documents=documents),root/'out',Path('/hick'),lambda *_args,**_kwargs:self.fail('No target execution')))
                index=json.loads((root/'out/index.json').read_text())
                self.assertEqual([doc['status'] for doc in index['documents']],['failed','passed'])
                self.assertNotEqual(index['documents'][0]['source_hash'],index['documents'][1]['source_hash'])
                # Declaring an expected failure cannot waive unavailable source evidence.
                source=b'# Changed without matching prediction evidence\n';(root/'vulnerable.md').write_bytes(source)
                documents[0]['reviewed_inputs']['vulnerable.md']=publisher.digest(source)
                self.assertFalse(publisher.publish(root,dict(version=1,documents=documents),root/'stale',Path('/hick')))

    def test_future_stop_projection_retains_coordinates_without_prior_visible_state(self):
        recorded=json.loads((Path(__file__).resolve().parents[1] / 'examples/security-authorization/predictions.json').read_text())
        event=next(entry['event'] for entry in recorded['history']['ordering'] if entry['event']['kind']=='sealed')
        event['future_stop']=dict(run='retained-host-run',thread=1,frame=2,hit=1,prior_visible_digest='a'*64)
        publisher._prediction_module.selected_history(recorded)
        event['future_stop']['prior_visible']='PRIVATE PREVIOUS STATE'
        with self.assertRaises(ValueError): publisher._prediction_module.selected_history(recorded)

    def test_prediction_selection_refuses_private_actual_and_unknown_payload(self):
        recorded=json.loads((Path(__file__).resolve().parents[1] / 'examples/security-authorization/predictions.json').read_text())
        changed=json.loads(json.dumps(recorded))
        next(entry['event'] for entry in changed['history']['ordering'] if entry['event']['kind']=='finished')['actual']='PRIVATE ANSWER'
        with self.assertRaises(ValueError): publisher._prediction_module.selected_history(changed)
        with self.assertRaises(ValueError): publisher._prediction_module.selected_history({**recorded, 'private_evidence':'PRIVATE ANSWER'})
        missing=json.loads(json.dumps(recorded))
        missing['history']['ordering']=[entry for entry in missing['history']['ordering'] if entry['event']['kind']!='finished']
        with self.assertRaises(ValueError): publisher._prediction_module.selected_history(missing)
        if recorded.get('debugger_contexts'):
            context=json.loads(json.dumps(recorded))
            context['debugger_contexts'][0]['hits'][0]['values']=['PRIVATE TARGET']
            with self.assertRaises(ValueError): publisher._prediction_module.selected_history(context)
    def test_recordings_reuse_selected_reviewed_assets_without_implicit_sessions(self):
        video = 'examples/public-demo.webm'
        demo = dict(reviewed=True, capture_kind='native-provider', captured_revision='a'*40, recording_asset=video, cuts=['One declared cut'])
        self.assertEqual(publisher.demonstration(dict(demonstration=demo), {video:'b'*64})['recording_asset'], video)
        for change in (dict(reviewed=False), dict(capture_kind='browser-replay'), dict(recording_asset='sessions/private.webm'), dict(session_asset='sessions/private.md'), dict(captured_revision=123), dict(recording_asset=[])):
            with self.assertRaises(ValueError): publisher.demonstration(dict(demonstration={**demo, **change}), {video:'b'*64})
        self.assertIsNone(publisher.demonstration({}, {}))
    # Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
    def test_failed_run_publishes_details_hashes_and_missing_witness(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / 'spec.md').write_text('# Requirement\n'); (root / 'lock').write_text('deps')
            selection = dict(version=1, documents=[dict(path='spec.md', reviewed_output=True, dependencies=['lock'], reviewed_inputs={'spec.md':publisher.digest(b'# Requirement\n'),'lock':publisher.digest(b'deps')}, requirements=[dict(id='R1', text='budget', witnesses=[9])])])
            def run(command, **kwargs):
                self.assertNotIn('OPENAI_API_KEY', kwargs['env'])
                self.assertNotEqual(kwargs['cwd'], root)
                return SimpleNamespace(returncode=0 if command[1] == 'weave' else 3,
                    stdout=json.dumps([dict(blocks=[])]), stderr='expected 100; actual 160')
            with patch.object(publisher.subprocess, 'check_output', return_value='a'*40):
                self.assertFalse(publisher.publish(root, selection, root/'out', Path('/hick'), run))
            index = json.loads((root/'out/index.json').read_text())
            snapshot = json.loads((root/'out'/index['documents'][0]['snapshot']).read_text())
            self.assertEqual(index['documents'][0]['status'], 'failed')
            self.assertIn('actual 160', snapshot['verification']['commands'][-1]['stderr'])
            self.assertEqual(snapshot['source_hash'], publisher.digest(b'# Requirement\n'))
            self.assertEqual(snapshot['verification']['dependencies']['lock'], publisher.digest(b'deps'))
    # Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
    def test_explicit_selection_rejects_escape_private_input_and_unreviewed_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); (root/'spec.md').write_text('x'); (root/'.env').write_text('secret')
            for path in ('../escape', str(root/'spec.md'), '.env'):
                with self.assertRaises(ValueError): publisher.selected(root,path)
            with patch.object(publisher.subprocess,'check_output',return_value='b'*40):
                with self.assertRaises(ValueError): publisher.publish(root,dict(version=1,documents=[dict(path='spec.md')]),root/'out',Path('/hick'))

if __name__ == '__main__': unittest.main()
