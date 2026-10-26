<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/prediction-publication.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2375d732dd97"></a>
## `scripts/prediction-publication.py`

<a id="f-2375d732dd97-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2375d732dd97-expression_statement">
"""Selection validation for host-exported, answer-withheld prediction history.

No comparison/promotion policy lives here. The ledger owns those verdicts.
"""
</storycode:copy>
<a id="f-2375d732dd97-json"></a>
### `json`

<storycode:copy id="f-2375d732dd97-json">
import json


</storycode:copy>
<a id="f-2375d732dd97-selected_history"></a>
### `selected_history`

<storycode:copy id="f-2375d732dd97-selected_history">
def selected_history(value):
    if not isinstance(value, dict) or value.get('format') != 'hickory-selected-prediction-history' or value.get('version') != 1:
        raise ValueError('Expected a selected prediction history export')
    if set(value) - {'format', 'version', 'selected_scenarios', 'omitted_events', 'omitted_attempts', 'omitted_context_observations', 'history', 'scope', 'debugger_contexts'}:
        raise ValueError('Unexpected prediction publication payload')
    if len(json.dumps(value)) > 8 * 1024 * 1024:
        raise ValueError('Selected prediction history exceeds the publication bound')
    selected = value.get('selected_scenarios')
    if not isinstance(selected, list) or not 1 <= len(selected) <= 64 or any(not isinstance(key, str) or len(key) != 64 or any(c not in '0123456789abcdef' for c in key) for key in selected):
        raise ValueError('Selected scenario identities are required')
    for key in ('omitted_events', 'omitted_attempts', 'omitted_context_observations'):
        if type(value.get(key)) is not int or value[key] < 0:
            raise ValueError('Explicit omission counts are required')
    history = value.get('history')
    if not isinstance(history, dict) or not isinstance(history.get('attempts'), list) or not isinstance(history.get('ordering'), list):
        raise ValueError('Ordered prediction history is unavailable')
    if set(history) - {'generation', 'attempts', 'ordering'}:
        raise ValueError('Unexpected history payload')
    allowed = {
        'sealed': {'kind', 'id', 'actor', 'seal', 'context', 'future_stop'},
        'started': {'kind', 'id', 'seal', 'replay', 'actor'},
        'finished': {'kind', 'attempt', 'actual'},
        'evidence_unavailable': {'kind', 'attempt', 'actor'},
        'interrupted': {'kind', 'attempt', 'actor', 'reason'},
        'observed': {'kind', 'attempt', 'scenario', 'actor', 'surface', 'identity'},
        'promoted': {'kind', 'attempt', 'actor', 'identity'},
    }
    if type(history.get('generation')) is not int or history['generation'] < 0:
        raise ValueError('A bounded history generation is required')
    seals, starts, finished = set(), {}, set()
    previous = 0
    for entry in history['ordering']:
        if set(entry) - {'sequence', 'previous', 'digest', 'event'}:
            raise ValueError('Unexpected event envelope payload')
        event = entry.get('event', {})
        sequence=entry.get('sequence')
        if type(sequence) is not int or not previous < sequence <= history['generation']:
            raise ValueError('Prediction event ordering is unavailable')
        previous=sequence
        if event.get('kind') not in allowed or set(event) - allowed[event['kind']]:
            raise ValueError('Private or unsupported prediction event payload')
        if event['kind'] == 'finished' and event.get('actual') != 'withheld from agents':
            raise ValueError('Actual target answers cannot be published in this projection')
        if event['kind'] == 'started':
            if event.get('seal') not in seals or event.get('id') in starts:
                raise ValueError('Prediction admission is missing its unique prior seal')
            starts[event.get('id')]=event['seal']
        if event['kind'] == 'finished':
            if event.get('attempt') not in starts or event.get('attempt') in finished:
                raise ValueError('Prediction settlement is missing its admission')
            finished.add(event['attempt'])
        if event['kind'] == 'sealed':
            if 'future_stop' in event:
                stop = event['future_stop']
                if not isinstance(stop, dict) or set(stop) != {'run', 'thread', 'frame', 'hit', 'prior_visible_digest'} or not isinstance(stop['run'], str) or not stop['run'] or any(type(stop[key]) is not int or stop[key] < 1 for key in ('thread','frame','hit')) or stop['hit'] >= 100 or not isinstance(stop['prior_visible_digest'], str) or len(stop['prior_visible_digest']) != 64 or any(c not in '0123456789abcdef' for c in stop['prior_visible_digest']):
                    raise ValueError('Only answer-free prior stop coordinates may be published')
            seal = event.get('seal', {})
            if set(seal) - {'scenario', 'identity', 'target', 'predicted', 'rationale'} or set(seal.get('identity', {})) - {'source', 'inputs', 'dependencies', 'environment'}:
                raise ValueError('Unexpected sealed prediction payload')
            if not isinstance(event.get('id'), str) or event['id'] in seals:
                raise ValueError('A unique sealed identity is required')
            seals.add(event['id'])
    summary_keys = {'attempt', 'scenario', 'seal', 'classification', 'prediction_matched', 'original_requirement_satisfied', 'observed', 'promoted', 'current_certification'}
    for attempt in history['attempts']:
        if set(attempt) - summary_keys or attempt.get('current_certification') is not False or attempt.get('scenario') not in selected:
            raise ValueError('Only historical selected attempt summaries may be published')
        if starts.get(attempt.get('attempt')) != attempt.get('seal') or any(attempt.get(key) is not None for key in ('prediction_matched', 'original_requirement_satisfied')) and attempt.get('attempt') not in finished:
            raise ValueError('Attempt comparison lacks admitted settlement evidence')
    if len(history['attempts']) != len(starts):
        raise ValueError('Selected attempt history is incomplete')
    for capture in value.get('debugger_contexts', []):
        if set(capture) - {'attempt', 'adapter', 'file', 'line', 'problem', 'moved', 'truncated', 'hits'}:
            raise ValueError('Private capture payload cannot be published')
        for hit in capture.get('hits', []):
            if set(hit) - {'index', 'context'} or set(hit.get('context') or {}) - {'run', 'thread', 'frame', 'function', 'document_line', 'source', 'source_line', 'breakpoint_bound'}:
                raise ValueError('Only answer-free debugger coordinates may be published')
            context=hit.get('context') or {}
            if any(context.get(key) is not None and not isinstance(context[key], str) for key in ('run','function','source')) or any(context.get(key) is not None and type(context[key]) is not int for key in ('thread','frame','document_line','source_line')) or 'breakpoint_bound' in context and type(context['breakpoint_bound']) is not bool:
                raise ValueError('Only scalar debugger coordinates may be published')
    return value


</storycode:copy>
<a id="f-2375d732dd97-reported_status"></a>
### `reported_status`

<storycode:copy id="f-2375d732dd97-reported_status">
def reported_status(history, source_hash):
    """Display the ledger's verdict; never recompare values or grant promotion."""
    seals = {entry['event']['id']: entry['event']['seal'] for entry in history['history']['ordering'] if entry['event']['kind'] == 'sealed'}
    matching = {}
    for attempt in history['history']['attempts']:
        if seals.get(attempt['seal'], {}).get('identity', {}).get('source') == source_hash:
            matching[attempt['scenario']] = attempt['classification']
    if set(matching) != set(history['selected_scenarios']):
        return 'unavailable'
    states = {'first_attempt_verified': 'passed', 'informed_prediction_verified': 'passed', 'verified_after_correction': 'passed', 'verified_after_prior_attempt': 'passed', 'mismatch': 'failed', 'requirement_failed': 'failed', 'execution_rejected': 'failed', 'stale': 'stale'}
    reported = [states.get(state, 'unavailable') for state in matching.values()]
    return next((state for state in ('unavailable', 'stale', 'failed') if state in reported), 'passed')
</storycode:copy>

### Assemble `scripts/prediction-publication.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/prediction-publication.py"><storycode:paste select="#f-2375d732dd97-expression_statement"/><storycode:paste select="#f-2375d732dd97-json"/><storycode:paste select="#f-2375d732dd97-selected_history"/><storycode:paste select="#f-2375d732dd97-reported_status"/></storycode:file>

</storycode:doc>
