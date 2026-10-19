"""Offline, paired replay benchmark; never opens a provider or sends telemetry.

Run: python -m hickory_agent_worker.refresh_experiment --output results.json
The deterministic witness is a transport/measurement test, not a model ranking.
"""
import argparse
from copy import deepcopy
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from time import perf_counter
from .context import MemoryContext
from .refresh import RefreshPolicy, Update


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


def replay(encoding, schedule, scenario, repetition):
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
        return dict(task=name, repetition=repetition, encoding=encoding,
                    scheduling=schedule, representation='unchanged-source',
                    manifests=[first, second], delivery_point='offline-boundary-2',
                    delivery_coverage='local assembly only; no provider receipt',
                    behavioral_success=success, stale_assumptions=None,
                    stale_edit_attempts=None, human_interventions=int(schedule == 'manual'),
                    supplied_utf8_bytes=len(text.encode()), input_tokens=None,
                    cached_tokens=None, uncached_tokens=None, output_tokens=None,
                    tool_calls=tools, latency_seconds=perf_counter() - start,
                    reported_cost=None, model='deterministic exposure witness',
                    metric_limitations='No model reasoning, provider usage or user effort measured')


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


def run(repetitions=3):
    if repetitions < 2:
        raise ValueError('paired evaluation requires at least two repetitions')
    return dict(version=1, limitations='Offline exposure witnesses do not rank model correctness, '
                'cost or ease of use. Paid providers require a separately configured driver/budget.',
                runs=[replay(encoding, schedule, scenario, repetition)
                      for repetition in range(repetitions) for scenario in SCENARIOS
                      for schedule in ('manual', 'automatic')
                      for encoding in ('append', 'projection', 'reread')])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--repetitions', type=int, default=3)
    args = parser.parse_args()
    args.output.write_text(json.dumps(run(args.repetitions), indent=2) + '\n')


if __name__ == '__main__':
    main()
