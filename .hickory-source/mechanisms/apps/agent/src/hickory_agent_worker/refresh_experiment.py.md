<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/agent/src/hickory_agent_worker/refresh_experiment.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2a254a85d13b"></a>
## `apps/agent/src/hickory_agent_worker/refresh_experiment.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Local refresh experiments preserve historical request evidence](../../../../../guarantees/agent/refresh-experiments-preserve-evidence/contract.md).

<a id="f-2a254a85d13b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2a254a85d13b-expression_statement">
"""Offline, paired replay benchmark; never opens a provider or sends telemetry.

Run: python -m hickory_agent_worker.refresh_experiment --output results.json
The deterministic witness is a transport/measurement test, not a model ranking.
"""
</storycode:copy>
<a id="f-2a254a85d13b-argparse"></a>
### `argparse`

<storycode:copy id="f-2a254a85d13b-argparse">
import argparse
</storycode:copy>
<a id="f-2a254a85d13b-deepcopy"></a>
### `deepcopy`

<storycode:copy id="f-2a254a85d13b-deepcopy">
from copy import deepcopy
</storycode:copy>
<a id="f-2a254a85d13b-json"></a>
### `json`

<storycode:copy id="f-2a254a85d13b-json">
import json
</storycode:copy>
<a id="f-2a254a85d13b-subprocess"></a>
### `subprocess`

<storycode:copy id="f-2a254a85d13b-subprocess">
import subprocess
</storycode:copy>
<a id="f-2a254a85d13b-Path"></a>
### `Path`

<storycode:copy id="f-2a254a85d13b-Path">
from pathlib import Path
</storycode:copy>
<a id="f-2a254a85d13b-TemporaryDirectory"></a>
### `TemporaryDirectory`

<storycode:copy id="f-2a254a85d13b-TemporaryDirectory">
from tempfile import TemporaryDirectory
</storycode:copy>
<a id="f-2a254a85d13b-perf_counter"></a>
### `perf_counter`

<storycode:copy id="f-2a254a85d13b-perf_counter">
from time import perf_counter
</storycode:copy>
<a id="f-2a254a85d13b-MemoryContext"></a>
### `MemoryContext`

<storycode:copy id="f-2a254a85d13b-MemoryContext">
from .context import MemoryContext
</storycode:copy>
<a id="f-2a254a85d13b-RefreshPolicy"></a>
### `RefreshPolicy`

<storycode:copy id="f-2a254a85d13b-RefreshPolicy">
from .refresh import RefreshPolicy, Update


</storycode:copy>
<a id="f-2a254a85d13b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-2a254a85d13b-expression_statement-2">
SCENARIOS = (
    ('requirement', 'Allow deletion', 'Require approval before deletion', 'changed'),
    ('instructions', 'Scope: root; allow shell', 'Scope: child; forbid shell', 'changed'),
    ('overlapping_reads', 'limit=10\nretry=2', 'limit=4\nretry=2', 'changed'),
    ('outside_read', 'limit=10', 'limit=10', 'changed_elsewhere'),
    ('rename', 'old.py: limit=10', 'new.py: limit=10', 'moved'),
    ('deletion', 'limit=10', None, 'unavailable'),
    ('unsaved_buffer', 'limit=10', 'limit=7', 'changed'),
    ('concurrent_edit', 'limit=10', 'limit=3', 'changed'),
)


</storycode:copy>
<a id="f-2a254a85d13b-replay"></a>
### `replay`

<storycode:copy id="f-2a254a85d13b-replay">
def replay(encoding, schedule, scenario, repetition, driver=None):
    name, before, after, state = scenario
    with TemporaryDirectory(prefix='hick-refresh-') as directory:
        path = Path(directory) / 'source.txt'
        path.write_text(before)
        policy = RefreshPolicy(encoding)
        context = MemoryContext(f'{name}-{repetition}', policy)
        original = [dict(role='user', content=before),
                    dict(role='assistant', content='Historical plan: use original requirement')]
        first = context.prepare(original)
        frozen = deepcopy(first)
        initial = driver(deepcopy(first), path, dict(task=name, phase='plan', budget_usd=0)) if driver else None
        if after is None:
            path.unlink()
        else:
            path.write_text(after)
        # Unsaved content is a distinct adapter head, never implicitly saved.
        buffer = after if name == 'unsaved_buffer' else None
        if buffer is not None:
            path.write_text(before)
        current = buffer if buffer is not None else (path.read_text() if path.exists() else None)
        update = Update(name, before, current, 'revision-2', state,
                        scope='child' if name == 'instructions' else 'conversation')
        start = perf_counter()
        # Scheduling is independent of encoding. Manual records an intervention;
        # the same captured update enters the next safe boundary in both arms.
        policy.queue_update(update)
        second = policy.assemble(context, original)
        tools = 0
        if encoding == 'reread':
            tools = 1
            messages = [dict(role=p['role'], content=p['text'])
                        for p in second['request']['parts']]
            messages.append(dict(role='user', content='Fresh bounded read: ' + json.dumps(current)))
            second = context.prepare(messages)
        text = '\n'.join(p['text'] for p in second['request']['parts'])
        # Independent expected-answer witness: the new requirement must be
        # observable, and invalidation cannot be silently omitted.
        success = ((current is None and 'null' in text) or
                   (current is not None and (current in text or json.dumps(current) in text)))
        success = success and 'require revalidation' in text
        assert first == frozen, 'historical manifest was mutated'
        if name == 'unsaved_buffer':
            assert path.read_text() == before, 'refresh saved an unsaved buffer'
        # Deliberately queue a newer edit after assembly: it cannot enter request 2.
        if name == 'concurrent_edit':
            policy.queue_update(Update(name, current, 'limit=999', 'revision-3'))
            assert 'limit=999' not in text
        result = driver(deepcopy(second), path, dict(task=name, phase='act',
                        unsaved_buffer=buffer, budget_usd=0)) if driver else None
        # The task witness checks the driver's returned decision, not source
        # presence. The expected answer is never included in its manifest.
        observed = result is not None
        action_success = ('driver_error' not in result and 'answer' in result and result['answer'] == current) if observed else None
        stale_assumptions = int(result.get('assumed_revision') == 'revision-1') if observed and 'assumed_revision' in result else None
        stale_attempts = sum(action.get('revision') == 'revision-1' for action in result.get('edit_attempts', [])) if observed and 'edit_attempts' in result else None
        return dict(task=name, repetition=repetition, encoding=encoding,
                    scheduling=schedule, representation='unchanged-source',
                    manifests=[first, second], delivery_point='offline-boundary-2',
                    delivery_coverage='local assembly only; no provider receipt',
                    exposure_success=success, behavioral_success=action_success,
                    driver_observations=[initial, result],
                    driver_coverage='driver-returned decisions; provider internals opaque' if observed else 'no behavioral driver',
                    stale_assumptions=stale_assumptions,
                    stale_edit_attempts=stale_attempts,
                    human_interventions=result.get('human_interventions') if observed else None,
                    scheduled_manual_updates=int(schedule == 'manual'),
                    supplied_utf8_bytes=len(text.encode()), input_tokens=result.get('input_tokens') if observed else None,
                    cached_tokens=result.get('cached_tokens') if observed else None,
                    uncached_tokens=result.get('uncached_tokens') if observed else None,
                    output_tokens=result.get('output_tokens') if observed else None,
                    tool_calls=result.get('tool_calls') if observed else None,
                    harness_read_calls=tools, usage_scope='act phase only; driver declared', latency_seconds=perf_counter() - start,
                    reported_cost=result.get('reported_cost') if observed else None,
                    model=result.get('model', 'driver unspecified') if observed else 'deterministic exposure witness',
                    metric_limitations='Driver usage/cost are declarations; local elapsed time is measured; human effort unmeasured')


</storycode:copy>
<a id="f-2a254a85d13b-compare_observations"></a>
### `compare_observations`

<storycode:copy id="f-2a254a85d13b-compare_observations">
def compare_observations(cases, driver):
    """Separate logs/debug/profile dimension; caller drives the same task twice.

    Drivers return observed behavioral_success, mapping_errors, tool_errors,
    correction_requests, stale_handle_refusals, user_effort and usage/latency.
    Missing measurements stay null rather than becoming invented zeroes.
    """
    metrics = ('behavioral_success', 'mapping_errors', 'tool_errors',
               'correction_requests', 'stale_handle_refusals', 'user_effort',
               'input_tokens', 'latency_seconds')
    return [dict(task=case['id'], representation=representation,
                 refresh_policy='fixed', **{key: result.get(key) for key in metrics})
            for case in cases for representation in ('coordinates', 'annotations-handles')
            for result in [driver(deepcopy(case), representation)]]


</storycode:copy>
<a id="f-2a254a85d13b-run"></a>
### `run`

<storycode:copy id="f-2a254a85d13b-run">
def run(repetitions=3, driver=None):
    if repetitions < 2:
        raise ValueError('paired evaluation requires at least two repetitions')
    return dict(version=1, limitations='Offline exposure witnesses do not rank model correctness, '
                'cost or ease of use. Paid providers require a separately configured driver/budget.',
                runs=[replay(encoding, schedule, scenario, repetition, driver)
                      for repetition in range(repetitions) for scenario in SCENARIOS
                      for schedule in ('manual', 'automatic')
                      for encoding in ('append', 'projection', 'reread')])


</storycode:copy>
<a id="f-2a254a85d13b-command_driver"></a>
### `command_driver`

<storycode:copy id="f-2a254a85d13b-command_driver">
def command_driver(argv):
    """Explicit local driver protocol. No provider configuration or credentials added.

    The subprocess receives exactly the candidate manifest and isolated source
    path. Output is one JSON decision. Nonzero exit and invalid/oversized output
    are failed evaluations, never success. Timeout bounds each invocation.
    """
    if not isinstance(argv, list) or not argv or not all(isinstance(a, str) for a in argv):
        raise ValueError('driver command must be a nonempty JSON argv array')
    def invoke(manifest, source, task):
        request = dict(manifest=manifest, source=str(source), task={**task, 'instructions': 'Report current source bytes as answer (null for deletion), using supplied unsaved_buffer when present. Return assumed_revision and edit_attempts when observable.'},
                       coverage='Only this supplied manifest is recorded; driver/provider internals opaque')
        try:
            result = subprocess.run(argv, input=json.dumps(request), text=True,
                                    capture_output=True, timeout=60, cwd=source.parent)
            if result.returncode:
                return dict(driver_error='nonzero exit', exit_code=result.returncode)
            if len(result.stdout.encode()) > 1_048_576:
                return dict(driver_error='oversized response')
            value = json.loads(result.stdout)
            if not isinstance(value, dict):
                raise ValueError('driver response must be an object')
            return value
        except (ValueError, subprocess.TimeoutExpired, OSError) as error:
            return dict(driver_error=str(error))
    return invoke


</storycode:copy>
<a id="f-2a254a85d13b-main"></a>
### `main`

<storycode:copy id="f-2a254a85d13b-main">
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--repetitions', type=int, default=3)
    parser.add_argument('--driver-command', help='Explicit local JSON argv; receives manifests on stdin. No paid provider is configured.')
    args = parser.parse_args()
    driver = command_driver(json.loads(args.driver_command)) if args.driver_command else None
    args.output.write_text(json.dumps(run(args.repetitions, driver), indent=2) + '\n')


</storycode:copy>
<a id="f-2a254a85d13b-if_statement"></a>
### `if_statement`

<storycode:copy id="f-2a254a85d13b-if_statement">
if __name__ == '__main__':
    main()
</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/refresh_experiment.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/refresh_experiment.py"><storycode:paste select="#f-2a254a85d13b-expression_statement"/><storycode:paste select="#f-2a254a85d13b-argparse"/><storycode:paste select="#f-2a254a85d13b-deepcopy"/><storycode:paste select="#f-2a254a85d13b-json"/><storycode:paste select="#f-2a254a85d13b-subprocess"/><storycode:paste select="#f-2a254a85d13b-Path"/><storycode:paste select="#f-2a254a85d13b-TemporaryDirectory"/><storycode:paste select="#f-2a254a85d13b-perf_counter"/><storycode:paste select="#f-2a254a85d13b-MemoryContext"/><storycode:paste select="#f-2a254a85d13b-RefreshPolicy"/><storycode:paste select="#f-2a254a85d13b-expression_statement-2"/><storycode:paste select="#f-2a254a85d13b-replay"/><storycode:paste select="#f-2a254a85d13b-compare_observations"/><storycode:paste select="#f-2a254a85d13b-run"/><storycode:paste select="#f-2a254a85d13b-command_driver"/><storycode:paste select="#f-2a254a85d13b-main"/><storycode:paste select="#f-2a254a85d13b-if_statement"/></storycode:file>

</storycode:doc>
