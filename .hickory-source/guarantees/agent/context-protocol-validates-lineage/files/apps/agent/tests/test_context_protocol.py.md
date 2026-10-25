<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/tests/test_context_protocol.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Context protocol validates declared byte lineage before integration](../../../../contract.md)

Captured SHA-256: `5522da28803bfaa251eb4fe1ccd136a1b17afc68f9a4c0291da6c38d3ff68f53`.

<book:entry path="apps/agent/tests/test_context_protocol.py">
"""Shared contract witnesses, independent of Monty, providers and agent policy."""
import json
from pathlib import Path
import unittest
from hickory_agent_worker import context_protocol as p

FIXTURES = Path(__file__).resolve().parents[3] / 'protocols/context-v1/fixtures.json'


def run_case(case):
    value, op = case['payload'], case['operation']
    if op == 'bundle':
        return p.validate_bundle(value)
    if op == 'negotiate':
        return p.negotiate(*value)
    if op == 'offset':
        return p.utf16_to_utf8(value['text'], value['offset'])
    if op == 'page':
        return p.validate_page(**value)
    if op == 'frame':
        return p.validate_frame(value['raw'].encode(), value['limit'], value['expected_id'])
    if op == 'resolve':
        return p.resolve_handle(**value)
    if op == 'replay':
        registry = p.Idempotency()
        return [registry.register(*pair) for pair in value]
    raise AssertionError(op)


class ContextProtocolTests(unittest.TestCase):
    # Guarantee: docs/guarantees/agent/context-protocol-validates-lineage.md
    def test_shared_conformance(self):
        for case in json.loads(FIXTURES.read_text()):
            with self.subTest(case=case['name']):
                if case['expected'] == 'ok':
                    result = run_case(case)
                    if 'result' in case:
                        self.assertEqual(result, case['result'])
                    if case['operation'] == 'bundle':
                        self.assertEqual(json.loads(json.dumps(result)), case['payload'])
                else:
                    with self.assertRaises(p.ProtocolFailure) as caught:
                        run_case(case)
                    self.assertEqual(caught.exception.code, case['expected'])

    def test_existing_broker_requires_matching_response(self):
        # Guarantee: docs/guarantees/agent/context-protocol-validates-lineage.md
        from io import StringIO
        from hickory_agent_worker.runtime import Broker
        output = StringIO()
        broker = Broker(StringIO('{"id":1,"result":{"events":[]}}\n'), output)
        self.assertEqual(broker.call('context.changes', {'cursor': 'c1'}), {'events': []})
        self.assertEqual(json.loads(output.getvalue()), {
            'id': 1, 'method': 'context.changes', 'params': {'cursor': 'c1'}})
        for frame in ('{"method":"context.changed"}\n', '{"id":2,"result":{}}\n'):
            with self.subTest(frame=frame):
                broker = Broker(StringIO(frame), StringIO())
                with self.assertRaisesRegex(RuntimeError, 'Mismatched broker response'):
                    broker.call('context.changes', {})
</book:entry>

</book:doc>
