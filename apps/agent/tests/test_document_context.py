"""Safety and information-equivalence witnesses for the document/context pilots."""
import hashlib
import unittest
from hickory_agent_worker.core import ModelResponse
from hickory_agent_worker.document_loop import DocumentAgent
from hickory_agent_worker.context_study import make_case, render_reading, score

class DocumentLoopTests(unittest.TestCase):
    def test_mutation_and_host_access_are_non_passing_even_with_plausible_output(self):
        from hickory_agent_worker.hosted import monty, LIMITS
        from hickory_agent_worker.local_experiment import evaluate
        with monty() as pool:
            with pool.checkout(limits=LIMITS) as session:
                mutation = evaluate(session, 'def solve(data):\n    data.sort()\n    return data', [([2,1],[1,2])])
                self.assertFalse(mutation['passed'])
                self.assertFalse(mutation['cases'][0]['input_unchanged'])
                host = evaluate(session, 'def solve(data):\n    return open("/etc/passwd").read()', [(None, '')])
                self.assertFalse(host['passed'])

    def test_failed_paths_leave_active_context_but_remain_in_journal(self):
        calls,events,predecessors=[],[],[]
        replies=iter(['incorrect candidate','correct candidate'])
        class Model:
            def complete(self,messages):
                calls.append(messages)
                return next(replies)
        def evaluate(document,previous):
            predecessors.append(previous)
            return {'accepted':document=='correct candidate','actual_failure':'negative balance'}
        result=DocumentAgent(Model(),evaluate,lambda kind,data:events.append((kind,data))).run('Original order matters')
        self.assertEqual(result.document,'correct candidate')
        self.assertEqual(predecessors,[hashlib.sha256(b'').hexdigest(),hashlib.sha256(b'incorrect candidate').hexdigest()])
        self.assertEqual(len(calls[1]),4)
        self.assertEqual(sum(m['role']=='system' for m in calls[1]),1)
        self.assertIn('Original order matters',calls[1][1]['content'])
        self.assertIn('negative balance',calls[1][3]['content'])
        self.assertTrue(any(data.get('text')=='incorrect candidate' for kind,data in events))

    def test_cutoff_and_model_success_claims_cannot_establish_acceptance(self):
        evaluations=[]
        class Model:
            def complete(self,messages): return ModelResponse('I passed every test','length')
        with self.assertRaisesRegex(RuntimeError,'iteration limit'):
            DocumentAgent(Model(),lambda *args:evaluations.append(args)).run('Task',max_iterations=2)
        self.assertEqual(evaluations,[])
        class Claim:
            def complete(self,messages): return 'Accepted: true'
        with self.assertRaisesRegex(RuntimeError,'iteration limit'):
            DocumentAgent(Claim(),lambda *args:{'accepted':False}).run('Task',max_iterations=1)

    def test_polishing_keeps_algorithm_and_refused_revision_keeps_predecessor(self):
        from pathlib import Path
        import tempfile
        from hickory_agent_worker.document_experiment import DocumentEvaluator
        from hickory_agent_worker.hosted import monty, LIMITS
        hick=Path('target/debug/hick').resolve()
        if not hick.exists(): self.skipTest('Build hick for reconstruction witness')
        def document(code, prose=''):
            return '<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n'+prose+'\n<hick:project/>\n<hick:file path="solution.py">'+code+'</hick:file>\n</hick:doc>'
        original=document('def solve(data):\n    return data\n')
        prose='# Preserve the input\n\nThe task returns its input without mutation. Returning the existing value directly preserves every element and its order. No intermediate transformation is needed, so the implementation also avoids introducing a copy or changing the representation.\n'
        changed=document('def solve(data):\n    return data + 0\n',prose)
        polished=document('# Preserve the value directly.\ndef solve(data):\n    return data\n',prose)
        fenced=document('```python\ndef solve(data):\n    return data\n```\n')
        replies=iter([fenced,original,changed,polished]); requests=[];events=[]
        class Model:
            def complete(self,messages):
                requests.append(messages)
                return next(replies)
        with tempfile.TemporaryDirectory() as scratch, monty() as pool:
            with pool.checkout(limits=LIMITS) as session:
                evaluator=DocumentEvaluator({'public':[(3,3)]},session,hick,Path(scratch),polish=True)
                result=DocumentAgent(Model(),evaluator,lambda kind,data:events.append((kind,data))).run('Return input unchanged',polish=True)
                html_heading=polished.replace('# Preserve the input','<h1>Preserve the input</h1>')
                heading_receipt=evaluator(html_heading,hashlib.sha256(polished.encode()).hexdigest())
                self.assertTrue(heading_receipt['accepted'])
                self.assertTrue(heading_receipt['source_output_equal'])
        self.assertEqual(result.iterations,4)
        self.assertIn('Polishing stage',requests[2][0]['content'])
        self.assertEqual(result.document,polished)
        self.assertIn(original,requests[3][2]['content'])
        self.assertNotIn(changed,requests[3][2]['content'])
        receipts=[data['receipt'] for kind,data in events if kind=='document-evaluation']
        self.assertIn('literal Python',receipts[0]['error'])
        self.assertTrue(receipts[2]['retain_previous'])
        self.assertFalse(receipts[2]['accepted'])
        self.assertTrue(receipts[3]['source_output_equal'])
        self.assertEqual(evaluator.code,'# Preserve the value directly.\ndef solve(data):\n    return data\n')

    def test_candidate_cannot_borrow_an_earlier_definition_or_case_state(self):
        from hickory_agent_worker.hosted import monty, LIMITS
        from hickory_agent_worker.local_experiment import evaluate
        with monty() as pool:
            with pool.checkout(limits=LIMITS) as session:
                session.feed_run('def solve(data):\n    return data')
                self.assertFalse(evaluate(session,'pass',[(3,3)])['passed'])
                self.assertFalse(evaluate(session,'def solve(data):\n    return data',[])['passed'])
                code='def solve(data):\n    global seen\n    try:\n        seen += 1\n    except NameError:\n        seen = 1\n    return seen'
                self.assertTrue(evaluate(session,code,[(0,1),(0,1)])['passed'])


class ContextPlacementTests(unittest.TestCase):
    def test_unprojected_context_keeps_host_scope_instructions(self):
        from hickory_agent_worker.core import reading_context
        raw='No open folder is authorized.\n{"buffers":[]}'
        self.assertEqual(reading_context(raw,'interleaved'),raw)

    def test_selected_projection_places_matching_evidence_without_exposing_hidden_buffers(self):
        import json
        from hickory_agent_worker.core import reading_context
        group={'state':'captured','evidence':['receipt-1'],'reading':{
            'source':{'uri':'notes.py','revision':'abc'},'scope':{'run':'actual-run'},
            'excerpts':[{'location':{'line':2},'text':'b'}],
            'annotations':[{'payload':{'value':4}}]}}
        context={'buffers':[{'id':'tab','path':'notes.py','content':'HIDDEN BUFFER'}],
                 'projection':{'version':1,'documents':[{'id':'tab','revision':'sha256:abc',
                    'segments':[{'line_start':1,'text':'a\nb\nc\n','omitted':False},
                                {'start':6,'end':20,'text':'','omitted':True}]}],
                    'observations':[group],'estimate':{'method':'heuristic'}}}
        structured=reading_context('Host context\n'+json.dumps(context),'structured')
        self.assertNotIn('HIDDEN BUFFER',structured)
        self.assertIn('Host context',structured)
        rendered=reading_context('Host context\n'+json.dumps(context),'interleaved')
        self.assertNotIn('HIDDEN BUFFER',rendered)
        self.assertLess(rendered.index('L2: b'),rendered.index('Captured evidence:'))
        self.assertLess(rendered.index('Captured evidence:'),rendered.index('L3: c'))
        self.assertIn('actual-run',rendered)
        self.assertIn('Omitted source range',rendered)
        group['reading']['source']['revision']='old'
        rendered=reading_context(json.dumps(context),'interleaved')
        self.assertIn('Unattached evidence',rendered)
        self.assertNotIn('Captured evidence:',rendered)
        group['reading']['source']['uri']='provider://aliased-source'
        group['source_buffer']={'id':'tab','revision':'sha256:abc'}
        rendered=reading_context(json.dumps(context),'interleaved')
        self.assertIn('Captured evidence:',rendered)
        group['state']='changed'
        rendered=reading_context(json.dumps(context),'interleaved')
        self.assertIn('Unattached evidence',rendered)

    def test_layouts_contain_identical_information_and_random_handles(self):
        case=make_case(17,'run')
        separate=render_reading(case['sources'],case['records'],'separate')
        interleaved=render_reading(case['sources'],case['records'],'interleaved')
        self.assertNotEqual(separate,interleaved)
        self.assertCountEqual(separate.splitlines(),interleaved.splitlines())
        self.assertEqual(len(separate.encode()),len(interleaved.encode()))
        for action in case['actions']:
            self.assertEqual(separate.count(__import__('json').dumps(action['handle'])),1)
            self.assertEqual(interleaved.count(__import__('json').dumps(action['handle'])),1)
        self.assertNotEqual(case['expected'],make_case(18,'run')['expected'])

    def test_stale_or_fabricated_handles_are_non_passing(self):
        import json
        case=make_case(8,'debug')
        stale=next(a['handle'] for a in case['actions'] if not a['enabled'])
        self.assertTrue(score(case,ModelResponse(json.dumps({'handle':stale})))['stale_selection'])
        self.assertFalse(score(case,ModelResponse('{"handle":"forged"}'))['correct'])
        self.assertFalse(score(case,ModelResponse(json.dumps({'handle':case['expected']}),'length'))['correct'])


if __name__=='__main__':
    unittest.main()


