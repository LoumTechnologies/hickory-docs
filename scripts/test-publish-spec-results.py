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
