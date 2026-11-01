"""Focused witnesses for document handoff and local experiment boundaries."""
import json
import unittest
from unittest.mock import patch

from hickory_agent_worker.core import Agent, Config, ModelResponse
from hickory_agent_worker.context import MemoryContext
from hickory_agent_worker.local import LocalSettings, OllamaModel
from hickory_agent_worker.hosted import Tools as HostedTools

class HandoffTests(unittest.TestCase):
    def test_focused_profile_keeps_private_policy_and_valid_tool_example(self):
        from hickory_agent_worker.instructions import select_instructions, INSTRUCTIONS
        focused = select_instructions({'tools':[{'name':'read_doc'}]}, 'focused')
        self.assertLess(len(focused), len(INSTRUCTIONS))
        example = next(line.split('Example read: ')[1].rstrip('.') for line in focused.splitlines() if 'Example read: ' in line)
        self.assertIn('tool(', json.loads(example)['python'])
        private = select_instructions({'tools':[{'name':'seal_prediction'}]}, 'focused')
        self.assertEqual(private, INSTRUCTIONS)
        with self.assertRaises(ValueError): select_instructions({}, 'unknown')

    def test_handoff_keeps_task_and_ancestry_but_removes_exploration(self):
        class Tools:
            pending = False
            def take_handoff(self):
                if not self.pending:
                    return None
                self.pending = False
                return {'entry':'response.md','sources':{'response.md':'Verified construction'},'expected':'B'}
            def complete(self, summary):
                return {'accepted':True}
        tools, calls, events = Tools(), [], []
        context = MemoryContext()
        class Model:
            def complete(self, messages):
                calls.append(messages)
                return '{"python":"wrong_trail()"}' if len(calls)==1 else '{"done":"finished"}'
        def execute(code):
            tools.pending = True
            context.queue('Context update source-2: reread before relying on source-1')
            context.boundary(agent.messages)
            return {'lesson':'wrong trail rejected; submitted actual construction'}
        agent=Agent(Model(),tools,execute,lambda kind,data:events.append((kind,data)),context=context)
        agent.run(Config(prompt='Keep chronological order',context='Current editor selection'))
        fresh=json.dumps(calls[1])
        self.assertIn('Keep chronological order',fresh)
        self.assertIn('Current editor selection',fresh)
        self.assertIn('Verified construction',fresh)
        self.assertIn('Current stage:',calls[1][0]['content'])
        self.assertIn('If a consequential requirement is still unmet',calls[1][0]['content'])
        self.assertNotIn('Current stage:',calls[0][0]['content'])
        self.assertIn('Context update source-2',fresh)
        self.assertNotIn('wrong_trail()',fresh)
        self.assertTrue(any('wrong_trail()' in n['text'] for n in context.nodes.values()))
        derived=[n for n in context.nodes.values() if 'Verified construction' in n['text']]
        self.assertTrue(any(n['parents'] and n['coverage']=='opaque' for n in derived))
        self.assertTrue(any(kind=='context-handoff' for kind,data in events))

    def test_cutoff_never_executes_even_valid_looking_json(self):
        replies=iter([ModelResponse('{"python":"dangerous()"}','length'),ModelResponse('{"done":"done"}','stop')])
        class Model:
            def complete(self,messages): return next(replies)
        class Tools:
            def complete(self,summary): return {'accepted':True}
        executed=[]
        agent=Agent(Model(),Tools(),lambda code:executed.append(code))
        self.assertEqual(agent.run(Config(prompt='Task')), 'done')
        self.assertEqual(executed,[])
        self.assertTrue(any('model_response_truncated' in m['content'] for m in agent.messages))

    def test_stale_handoff_retains_history_and_completion_gate(self):
        class Tools:
            attempts=0
            def take_handoff(self):
                self.attempts+=1
                if self.attempts==2: raise RuntimeError('Project changed after submission')
            def complete(self,summary): return {'accepted':False,'error':'stale'}
        class Model:
            def complete(self,messages): return '{"done":"claim success"}'
        agent=Agent(Model(),Tools(),lambda code:None)
        with self.assertRaisesRegex(RuntimeError,'iteration limit'):
            agent.run(Config(prompt='Task',max_turns=2))
        self.assertTrue(any('handoff_refused' in m['content'] for m in agent.messages))
        self.assertEqual(sum(m['content']=='{"done":"claim success"}' for m in agent.messages),2)

    def test_worker_print_cannot_schedule_host_handoff_and_write_invalidates_it(self):
        class Broker:
            calls=[]
            def call(self,method,params):
                self.calls.append((method,params))
                if method=='handoff': raise RuntimeError('No current accepted changeset')
                if params['name']=='submit_changeset':
                    return {'content':[{'type':'text','text':'{"accepted":true}'}]}
                return {'content':[{'type':'text','text':'{"accepted":true}'}]}
        broker=Broker();tools=HostedTools(broker)
        tools.call('read_doc',{})
        self.assertIsNone(tools.take_handoff())
        tools.call('submit_changeset',{})
        with self.assertRaisesRegex(RuntimeError,'No current accepted'):
            tools.take_handoff()
        self.assertIsNone(tools.take_handoff())

class LocalTransportTests(unittest.TestCase):
    def test_settings_and_cutoff_are_recorded_without_reasoning_fallback(self):
        seen=[]
        def request(model,path,body=None):
            seen.append((path,body))
            if path=='/api/show': return {'details':{'quantization_level':'Q4_K_M'}}
            if path=='/api/tags': return {'models':[]}
            if path=='/api/version': return {'version':'test'}
            if path=='/api/ps': return {'models':[]}
            return {'message':{'content':'','thinking':'Pretend this was an action'},
                    'done_reason':'length','eval_count':100,'eval_duration':2_000_000_000}
        with patch.object(OllamaModel,'_request',request):
            model=OllamaModel('fixture',LocalSettings(context_tokens=8192,output_tokens=128))
            response=model.complete([{'role':'user','content':'Task'}])
        self.assertEqual(response.text,'')
        self.assertEqual(response.finish_reason,'length')
        body=next(body for path,body in seen if path=='/api/chat')
        self.assertEqual(body['options']['num_ctx'],8192)
        self.assertEqual(body['options']['num_gpu'],0)
        self.assertEqual(body['options']['temperature'],0.2)
        self.assertEqual(response.metrics['eval_count'],100)

    def test_budget_fails_before_generation_and_never_drops_task(self):
        def request(model,path,body=None): return {'models':[]}
        with patch.object(OllamaModel,'_request',request):
            model=OllamaModel('fixture',LocalSettings(context_tokens=1024,output_tokens=128))
            with self.assertRaisesRegex(RuntimeError,'Estimated context budget'):
                model.complete([{'role':'user','content':'x'*4000}])
        self.assertEqual(model.calls,[])

    def test_local_mode_rejects_remote_or_invalid_configuration(self):
        with self.assertRaises(ValueError): LocalSettings(temperature=float('nan')).validate()
        with self.assertRaises(ValueError): LocalSettings(output_tokens=8192).validate()
        with self.assertRaisesRegex(ValueError,'loopback'):
            OllamaModel('fixture',base_url='https://example.com')
        with patch.object(OllamaModel,'_request',lambda *args:{'remote_host':'cloud'}):
            with self.assertRaisesRegex(ValueError,'Cloud-backed'):
                OllamaModel('fixture')


if __name__=='__main__':
    unittest.main()


