import json
import unittest
from hickory_agent_worker.core import Agent, Config, reading_context


class ReadingProjectionTests(unittest.TestCase):
    # Guarantee: docs/guarantees/agent/expanded-reading-projection.md
    def test_projection_never_supplies_full_edit_snapshot(self):
        value = {'editors': {'buffers': [{'id': 'draft', 'content': 'secret folded body',
                  'observations': ['unselected']}], 'projection': {'version': 1,
                  'documents': [], 'observations': []}}}
        result = json.loads(reading_context(json.dumps(value)))
        self.assertEqual(result['editors']['buffers'], [{'id': 'draft'}])
        self.assertEqual(value['editors']['buffers'][0]['content'], 'secret folded body')

    def test_legacy_and_explicit_empty_scope_are_distinct(self):
        legacy = {'buffers': [{'content': 'legacy'}]}
        self.assertEqual(json.loads(reading_context(json.dumps(legacy))), legacy)
        self.assertEqual(reading_context('opaque adapter text'), 'opaque adapter text')


    def test_actual_model_request_contains_projection_without_complete_buffer(self):
        requests = []
        class Model:
            def complete_request(self, bundle):
                requests.append(bundle)
                return '{"done":"ok"}'
        class Tools:
            def complete(self, summary):
                return {'accepted': True}
        context = json.dumps({'editors': {'buffers': [{'id': 'draft', 'content': 'HIDDEN-BODY'}],
            'projection': {'version': 1, 'documents': [{'id': 'draft', 'revision': 'captured-revision',
            'segments': [{'start': 0, 'end': 7, 'text': 'VISIBLE', 'omitted': False}]}], 'observations': []}}})
        Agent(Model(), Tools(), None).run(Config('Task', context=context))
        parts = requests[0]['request']['parts']
        supplied = '\n'.join(p['text'] for p in parts)
        self.assertNotIn('HIDDEN-BODY', supplied)
        self.assertIn('VISIBLE', supplied)
        self.assertIn('captured-revision', supplied)
