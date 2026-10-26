<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Local refresh experiments preserve historical request evidence

For: Authors using an agent to change documents while retaining review control.

Given an explicitly queued update, the Python refresh policy supports appended
replacement passages, exact prior-read projection with a change notice, and
notification followed by a bounded reread. Scheduling remains a separate input.
Projection assembles a copy; original request bundles and caller history remain
unchanged. It replaces only exact role-matching plain text messages and leaves
tool-call envelopes intact. Source updates remain user notices; role/scope is
reported rather than promoted. Notices invalidate earlier dependent plans and
summaries without claiming to have corrected their reasoning.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/refresh-experiments-preserve-evidence.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-02ab2fac48b5"></a>
## `apps/agent/tests/test_refresh_experiment.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Local refresh experiments preserve historical request evidence](contract.md).

<a id="f-02ab2fac48b5-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-02ab2fac48b5-expression_statement">
"""Guarantee: docs/guarantees/agent/refresh-experiments-preserve-evidence.md."""
</storycode:copy>
<a id="f-02ab2fac48b5-unittest"></a>
### `unittest`

<storycode:copy id="f-02ab2fac48b5-unittest">
import unittest
</storycode:copy>
<a id="f-02ab2fac48b5-deepcopy"></a>
### `deepcopy`

<storycode:copy id="f-02ab2fac48b5-deepcopy">
from copy import deepcopy
</storycode:copy>
<a id="f-02ab2fac48b5-MemoryContext"></a>
### `MemoryContext`

<storycode:copy id="f-02ab2fac48b5-MemoryContext">
from hickory_agent_worker.context import MemoryContext
</storycode:copy>
<a id="f-02ab2fac48b5-RefreshPolicy"></a>
### `RefreshPolicy`

<storycode:copy id="f-02ab2fac48b5-RefreshPolicy">
from hickory_agent_worker.refresh import RefreshPolicy, Update
</storycode:copy>
<a id="f-02ab2fac48b5-run"></a>
### `run`

<storycode:copy id="f-02ab2fac48b5-run">
from hickory_agent_worker.refresh_experiment import run, compare_observations


</storycode:copy>
<a id="f-02ab2fac48b5-RefreshTests"></a>
### `RefreshTests`

<storycode:copy id="f-02ab2fac48b5-RefreshTests">
class RefreshTests(unittest.TestCase):
    def test_paired_schedules_share_tasks_and_preserve_receipts(self):
        report = run(2)
        self.assertEqual(len(report['runs']), 96)
        self.assertTrue(all(row['exposure_success'] for row in report['runs']))
        self.assertTrue(all(row['behavioral_success'] is None for row in report['runs']))
        self.assertTrue(all(row['reported_cost'] is None for row in report['runs']))
        self.assertEqual({row['encoding'] for row in report['runs']},
                         {'append', 'projection', 'reread'})

    def test_projection_never_promotes_role_or_rewrites_tool_messages(self):
        policy = RefreshPolicy('projection')
        context = MemoryContext(policy=policy)
        messages = [dict(role='system', content='old'),
                    dict(role='user', content='old', tool_call_id='call-1'),
                    dict(role='user', content='old')]
        frozen = deepcopy(messages)
        receipt = context.prepare([dict(role='user', content='old')])
        old = deepcopy(receipt)
        policy.queue_update(Update('source', 'old', 'new', 'rev2'))
        candidate = policy.assemble(context, messages)
        self.assertEqual(messages, frozen)
        self.assertEqual(receipt, old)
        parts = candidate['request']['parts']
        self.assertEqual(parts[0]['text'], 'old')
        self.assertEqual(parts[1]['text'], 'old')
        self.assertEqual(parts[2]['text'], 'new')
        self.assertEqual(parts[-1]['role'], 'user')

    def test_representation_dimension_is_fixed_policy_and_missing_metrics_null(self):
        report = compare_observations([dict(id='profile')],
                    lambda case, representation: dict(behavioral_success=True))
        self.assertEqual([r['representation'] for r in report],
                         ['coordinates', 'annotations-handles'])
        self.assertTrue(all(row['refresh_policy'] == 'fixed' for row in report))
        self.assertTrue(all(row['mapping_errors'] is None for row in report))

    def test_stale_driver_fails_even_when_fresh_context_is_present(self):
        def stale(manifest, source, task):
            return dict(answer='original stale decision', assumed_revision='revision-1',
                        edit_attempts=[dict(revision='revision-1')])
        report = run(2, stale)
        self.assertTrue(all(row['exposure_success'] for row in report['runs']))
        self.assertTrue(all(row['behavioral_success'] is False for row in report['runs']))
        self.assertTrue(all(row['stale_assumptions'] == 1 for row in report['runs']))
        self.assertTrue(all(row['stale_edit_attempts'] == 1 for row in report['runs']))
        self.assertTrue(all(len(row['driver_observations']) == 2 for row in report['runs']))

    def test_driver_observes_isolated_source_and_unsaved_input_without_getting_expected_answer(self):
        from pathlib import Path
        seen = set()
        def current(manifest, source, task):
            self.assertNotIn('expected', task)
            self.assertNotIn('answer', task)
            seen.add(str(source.parent))
            answer = task.get('unsaved_buffer')
            if answer is None:
                answer = source.read_text() if source.exists() else None
            return dict(answer=answer, assumed_revision='revision-2', edit_attempts=[])
        report = run(2, current)
        self.assertTrue(all(row['behavioral_success'] for row in report['runs']))
        self.assertEqual(len(seen), 96)
        self.assertTrue(all(row['reported_cost'] is None for row in report['runs']))

    def test_missing_driver_answer_cannot_pass_a_deleted_source(self):
        from hickory_agent_worker.refresh_experiment import replay, command_driver
        import sys
        driver = command_driver([sys.executable, '-c', 'print("{}")'])
        row = replay('append', 'manual', ('deletion', 'before', None, 'unavailable'), 0, driver)
        self.assertFalse(row['behavioral_success'])


</storycode:copy>
<a id="f-02ab2fac48b5-if_statement"></a>
### `if_statement`

<storycode:copy id="f-02ab2fac48b5-if_statement">
if __name__ == '__main__':
    unittest.main()
</storycode:copy>

### Assemble `apps/agent/tests/test_refresh_experiment.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/tests/test_refresh_experiment.py"><storycode:paste select="#f-02ab2fac48b5-expression_statement"/><storycode:paste select="#f-02ab2fac48b5-unittest"/><storycode:paste select="#f-02ab2fac48b5-deepcopy"/><storycode:paste select="#f-02ab2fac48b5-MemoryContext"/><storycode:paste select="#f-02ab2fac48b5-RefreshPolicy"/><storycode:paste select="#f-02ab2fac48b5-run"/><storycode:paste select="#f-02ab2fac48b5-RefreshTests"/><storycode:paste select="#f-02ab2fac48b5-if_statement"/></storycode:file>

## Implementation review

### Implementation

- [apps/agent/src/hickory_agent_worker/context.py](../python-requests-have-validated-context/contract.md#output-778ee53bd3ed)
- [apps/agent/src/hickory_agent_worker/refresh.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/refresh.py.md#output-1fafe6145c55)
- [apps/agent/src/hickory_agent_worker/refresh_experiment.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/refresh_experiment.py.md#output-2a254a85d13b)

### Tests and fixtures

- [apps/agent/tests/test_context_runtime.py](../python-requests-have-validated-context/contract.md#output-c807e1692e39)
- [apps/agent/tests/test_refresh_experiment.py](contract.md#output-02ab2fac48b5)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
