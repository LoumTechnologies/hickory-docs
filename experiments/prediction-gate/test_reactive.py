"""Focused integrity checks for the frozen reactive semantic extractions."""
import json
import unittest
import bench
from reactive_cases import TASKS, dcl_oracle, flatten_oracle, groups_oracle, union_oracle, where_oracle

class ReactiveIntegrity(unittest.TestCase):
    def test_correct_kernels_match_independent_oracles(self):
        for task in TASKS:
            with self.subTest(task=task['id']):
                cases=task['public']+task['hidden']
                result=bench.score(task['solution'],cases)
                self.assertEqual(len(result['passed']),14)
                self.assertTrue(result['correct'],result.get('error',result['passed']))

    def test_faults_are_detectable_and_long_cases_are_held_out(self):
        for task in TASKS:
            with self.subTest(task=task['id']):
                self.assertFalse(bench.score(task['start'],task['hidden'])['correct'])
                public={json.dumps(case[0],sort_keys=True) for case in task['public']}
                hidden={json.dumps(case[0],sort_keys=True) for case in task['hidden']}
                self.assertFalse(public & hidden)
                self.assertEqual(len(hidden),12)
                self.assertTrue(all(len(case[0][0])>=40 for case in task['hidden']))

    def test_alias_callbacks_use_creation_order_with_current_offsets(self):
        args=[['source','a',[1]],['attach',0,'x','a'],['attach',0,'y','a'],['insert','a',1,[2]]]
        actual=flatten_oracle(args)
        self.assertEqual(actual['snapshots'][-1],[1,2,1,2])
        self.assertEqual(actual['changes'][-2:],[[3,'add',2,[2]],[3,'add',1,[2]]])

    def test_retained_groups_and_owner_multiplicity(self):
        result=groups_oracle([['add','a',2],['remove','a'],['add','b',5]],3)
        self.assertEqual(result['snapshots'][1],[[2,1,[]]])
        self.assertEqual(result['snapshots'][-1],[[2,1,[['b',5]]]])
        self.assertEqual(sum(e[1]=='group' for e in result['changes']),1)
        actions=[['source','a',[]],['attach','x','a'],['attach','y','a'],['insert','a',5],
                 ['detach','x'],['dispose'],['delete','a',5]]
        result=union_oracle(actions)
        self.assertEqual(result['changes'],[[3,'add',5]])
        self.assertEqual(result['snapshots'][-3:],[[5],[5],[5]])
        self.assertEqual(result['subscribed'],[])

    def test_queue_precedes_quiescence_and_shutdown_precedes_queue(self):
        actions=[['enqueue',['add',1]],['enqueue',['complete',1]],['enqueue',['add',2]],
                 ['enqueue',['data',2,9]],['poll'],['poll'],['shutdown'],
                 ['enqueue',['data',2,10]],['poll']]
        result=dcl_oracle(actions,True)
        self.assertEqual(result['polls'],[['emit',[9]],['pending'],['done']])
        self.assertEqual(result['remaining'],1)
        self.assertEqual(result['latest'],[[2,9]])

    def test_quiescence_fault_is_detected_without_any_shutdown_action(self):
        actions=[['enqueue',['add',1]],['poll'],['enqueue',['complete',1]],
                 ['enqueue',['add',3]],['enqueue',['data',3,10]],['poll']]
        task=next(t for t in TASKS if t['id']=='dcl-queued-quiescence')
        expected=dcl_oracle(actions,True)
        self.assertEqual(expected['polls'],[['pending'],['emit',[10]]])
        self.assertFalse(bench.score(task['start'],[[[actions,True],expected]])['correct'])

    def test_index_shift_removes_visible_items_right_to_left(self):
        result=where_oracle([['insert',0,[['a',0],['b',1]]],['insert',0,[['x',1]]]],2)
        self.assertEqual(result['snapshots'],[[['a',0],['b',2]],[]])
        self.assertEqual(result['changes'][-2:],[[1,'remove',1,['b',2]],[1,'remove',0,['a',0]]])

    def test_case_errors_do_not_erase_other_outcomes_and_types_are_checked(self):
        result=bench.score('def run(x):\n    return 1 // x if x == 0 else x\n',[[[0],0],[[2],2]])
        self.assertEqual(result['passed'],[False,True])
        self.assertFalse(bench.score('def run():\n    return True\n',[[[],1]])['correct'])

if __name__=='__main__':
    unittest.main()
