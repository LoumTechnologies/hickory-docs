<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Context protocol validates declared byte lineage before integration

For: Authors using an agent to change documents while retaining review control.

Given a self-contained context v1 witness, when it is checked by Rust
`hick_context_protocol::validate_bundle` or Python
`context_protocol.validate_bundle`, then unknown node references, changed text
hashes, modified request segments, coverage gaps/overlaps and false exact
transformation mappings are rejected with typed errors. Summaries and model/tool
outputs retain typed producer genealogy without claiming exact correspondence or
semantic support. Opaque lineage stays explicit. Immutable artifact descriptors
are checked without pretending the validator fetched their bytes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/context-protocol-validates-lineage.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-4922a914e9e6"></a>
## `apps/agent/src/hickory_agent_worker/context_protocol.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Context protocol validates declared byte lineage before integration](contract.md).

<a id="f-4922a914e9e6-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4922a914e9e6-expression_statement">
"""Opt-in context v1 contract. No filesystem, refresh policy or broker side effects."""
</storycode:copy>
<a id="f-4922a914e9e6-sha256"></a>
### `sha256`

<storycode:copy id="f-4922a914e9e6-sha256">
from hashlib import sha256
</storycode:copy>
<a id="f-4922a914e9e6-files"></a>
### `files`

<storycode:copy id="f-4922a914e9e6-files">
from importlib.resources import files
</storycode:copy>
<a id="f-4922a914e9e6-json"></a>
### `json`

<storycode:copy id="f-4922a914e9e6-json">
import json
</storycode:copy>
<a id="f-4922a914e9e6-unicodedata"></a>
### `unicodedata`

<storycode:copy id="f-4922a914e9e6-unicodedata">
import unicodedata
</storycode:copy>
<a id="f-4922a914e9e6-cast"></a>
### `cast`

<storycode:copy id="f-4922a914e9e6-cast">
from typing import cast

</storycode:copy>
<a id="f-4922a914e9e6-Draft202012Validator"></a>
### `Draft202012Validator`

<storycode:copy id="f-4922a914e9e6-Draft202012Validator">
from jsonschema import Draft202012Validator, validators
</storycode:copy>
<a id="f-4922a914e9e6-Bundle"></a>
### `Bundle`

<storycode:copy id="f-4922a914e9e6-Bundle">
from .context_types import Bundle, ErrorCode, Hello, Negotiated

</storycode:copy>
<a id="f-4922a914e9e6-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4922a914e9e6-expression_statement-2">
SCHEMA = json.loads(files(__package__).joinpath('context_schema.json').read_text())
</storycode:copy>
<a id="f-4922a914e9e6-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-4922a914e9e6-expression_statement-3">
MAX_FRAME = 1_048_576
</storycode:copy>
<a id="f-4922a914e9e6-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-4922a914e9e6-expression_statement-4">
MAX_PAGE = 256
</storycode:copy>
<a id="f-4922a914e9e6-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-4922a914e9e6-expression_statement-5">
WireValidator = validators.extend(Draft202012Validator, type_checker=
    Draft202012Validator.TYPE_CHECKER.redefine(
        "integer", lambda _checker, value: isinstance(value, int) and not isinstance(value, bool)))


</storycode:copy>
<a id="f-4922a914e9e6-ProtocolFailure"></a>
### `ProtocolFailure`

<storycode:copy id="f-4922a914e9e6-ProtocolFailure">
class ProtocolFailure(ValueError):
    def __init__(self, code: ErrorCode, message: str):
        self.code = code
        super().__init__(message)


</storycode:copy>
<a id="f-4922a914e9e6-require"></a>
### `require`

<storycode:copy id="f-4922a914e9e6-require">
def require(ok, code, message):
    if not ok:
        raise ProtocolFailure(code, message)


</storycode:copy>
<a id="f-4922a914e9e6-validate"></a>
### `validate`

<storycode:copy id="f-4922a914e9e6-validate">
def validate(value, name):
    require(name in SCHEMA['$defs'], 'invalid_schema', 'Unknown schema type')
    try:
        json.dumps(value, ensure_ascii=False, allow_nan=False).encode()
    except (ValueError, UnicodeError, TypeError) as exc:
        raise ProtocolFailure('invalid_schema', 'Invalid JSON scalar') from exc
    schema = {'$schema': SCHEMA['$schema'], '$defs': SCHEMA['$defs'],
              '$ref': '#/$defs/' + name}
    error = next(WireValidator(schema).iter_errors(value), None)
    require(error is None, 'invalid_schema', str(error))
    return value


</storycode:copy>
<a id="f-4922a914e9e6-digest"></a>
### `digest`

<storycode:copy id="f-4922a914e9e6-digest">
def digest(data):
    return sha256(data).hexdigest()


</storycode:copy>
<a id="f-4922a914e9e6-assembly_digest"></a>
### `assembly_digest`

<storycode:copy id="f-4922a914e9e6-assembly_digest">
def assembly_digest(request):
    return digest(json.dumps(request, ensure_ascii=False, sort_keys=True,
                             separators=(',', ':'), allow_nan=False).encode())


</storycode:copy>
<a id="f-4922a914e9e6-span_bytes"></a>
### `span_bytes`

<storycode:copy id="f-4922a914e9e6-span_bytes">
def span_bytes(text, span):
    raw = text.encode()
    start, end = span['start'], span['end']
    require(start <= end <= len(raw), 'invalid_offset', 'Invalid UTF-8 byte range')
    try:
        raw[:start].decode(); raw[start:end].decode()
    except UnicodeDecodeError as exc:
        raise ProtocolFailure('invalid_offset', 'Split UTF-8 code point') from exc
    return raw[start:end]


</storycode:copy>
<a id="f-4922a914e9e6-utf16_to_utf8"></a>
### `utf16_to_utf8`

<storycode:copy id="f-4922a914e9e6-utf16_to_utf8">
def utf16_to_utf8(text, offset):
    require(isinstance(offset, int) and not isinstance(offset, bool) and offset >= 0,
            'invalid_offset', 'Invalid UTF-16 offset')
    units = size = 0
    for char in text:
        if units == offset:
            return size
        units += len(char.encode('utf-16-le')) // 2
        size += len(char.encode())
    require(units == offset, 'invalid_offset', 'Split surrogate pair or out of range')
    return size


</storycode:copy>
<a id="f-4922a914e9e6-negotiate"></a>
### `negotiate`

<storycode:copy id="f-4922a914e9e6-negotiate">
def negotiate(host: Hello, worker: Hello) -> Negotiated:
    validate(host, 'Hello'); validate(worker, 'Hello')
    require(1 in host['versions'] and 1 in worker['versions'], 'version_skew', 'No context v1')
    capabilities = sorted(set(host['capabilities']) & set(worker['capabilities']))
    require('context.changes' in capabilities, 'capability_missing', 'Polling required')
    frame = min(host['max_frame_bytes'], worker['max_frame_bytes'], MAX_FRAME)
    page = min(host['max_page_items'], worker['max_page_items'], MAX_PAGE)
    require(frame > 0 and page > 0, 'frame_limit', 'Zero negotiated limit')
    return cast(Negotiated, dict(version=1, capabilities=capabilities,
        transport='cursor_polling', max_frame_bytes=frame, max_page_items=page))


</storycode:copy>
<a id="f-4922a914e9e6-validate_frame"></a>
### `validate_frame`

<storycode:copy id="f-4922a914e9e6-validate_frame">
def validate_frame(raw: bytes, limit=MAX_FRAME, expected_id=None):
    require(len(raw) <= min(limit, MAX_FRAME) and raw.endswith(b'\n') and b'\n' not in raw[:-1],
            'frame_limit', 'One bounded newline-terminated JSON frame required')
    try:
        value = json.loads(raw.decode('utf-8'), parse_constant=lambda _: (_ for _ in ()).throw(ValueError()))
    except (ValueError, UnicodeDecodeError) as exc:
        raise ProtocolFailure('invalid_schema', 'Invalid JSON frame') from exc
    require(isinstance(value, dict), 'invalid_schema', 'Frame must be object')
    try:
        json.dumps(value, ensure_ascii=False, allow_nan=False).encode()
    except (ValueError, UnicodeError) as exc:
        raise ProtocolFailure('invalid_schema', 'Invalid JSON scalar') from exc
    if expected_id is not None:
        require(type(value.get('id')) is type(expected_id) and value.get('id') == expected_id
                and ('result' in value) != ('error' in value),
                'mismatched_response', 'Unsolicited or mismatched response')
    return value


</storycode:copy>
<a id="f-4922a914e9e6-validate_page"></a>
### `validate_page`

<storycode:copy id="f-4922a914e9e6-validate_page">
def validate_page(page, epoch, branch):
    validate(page, 'ChangePage')
    a, b = page['from_cursor'], page['next_cursor']
    require(a['epoch'] == b['epoch'] == epoch and a['branch'] == b['branch'] == branch,
            'cursor_expired', 'Reconcile retained references after epoch/branch change')
    require(0 < page['limit'] <= MAX_PAGE and len(page['events']) <= page['limit'],
            'invalid_page', 'Invalid page limit')
    sequence = a['position']
    for event in page['events']:
        require(event['sequence'] > sequence, 'invalid_page', 'Events must advance in order')
        sequence = event['sequence']
    require(b['position'] == sequence and (not page['has_more'] or bool(page['events'])),
            'invalid_page', 'Cursor must describe last event; no empty continuation')
    return page


</storycode:copy>
<a id="f-4922a914e9e6-validate_handle"></a>
### `validate_handle`

<storycode:copy id="f-4922a914e9e6-validate_handle">
def validate_handle(handle):
    validate(handle, 'Handle')
    require(handle['span']['start'] <= handle['span']['end'], 'invalid_offset', 'Invalid handle span')
    required = {'location': (), 'stop': ('run', 'stop'), 'frame': ('run', 'stop', 'frame'),
                'invocation': ('run', 'invocation')}[handle['kind']]
    require(all(k in handle for k in required), 'invalid_schema', 'Missing handle execution scope')
    return handle


</storycode:copy>
<a id="f-4922a914e9e6-resolve_handle"></a>
### `resolve_handle`

<storycode:copy id="f-4922a914e9e6-resolve_handle">
def resolve_handle(handle, request):
    validate_handle(handle); validate(request, 'ResolveRequest')
    require(request['handle'] == handle['id'] and all(
        request['expected_source'][k] == handle['source'][k] for k in ('id', 'revision'))
        and all(request.get(k) == handle.get(k) for k in ('run', 'stop', 'frame', 'invocation')),
        'stale_handle', 'Rust must resolve against current revision and execution scope')
    return handle


</storycode:copy>
<a id="f-4922a914e9e6-Idempotency"></a>
### `Idempotency`

<storycode:copy id="f-4922a914e9e6-Idempotency">
class Idempotency:
    """Session-scoped key/fingerprint registry. A replay never starts transport again."""
    def __init__(self):
        self.entries = {}

    def register(self, key, fingerprint):
        if key in self.entries:
            require(self.entries[key] == fingerprint, 'idempotency_conflict', 'Key reused for different bytes')
            return False
        self.entries[key] = fingerprint
        return True


</storycode:copy>
<a id="f-4922a914e9e6-validate_bundle"></a>
### `validate_bundle`

<storycode:copy id="f-4922a914e9e6-validate_bundle">
def validate_bundle(value) -> Bundle:
    if isinstance(value, dict) and (type(value.get('version')) is not int or value.get('version') != 1):
        raise ProtocolFailure('version_skew', 'Unsupported context version')
    validate(value, 'Bundle')
    nodes = {}
    for node in value['nodes']:
        require(node['id'] not in nodes, 'invalid_schema', 'Duplicate immutable reference')
        require(('text' in node) != ('artifact' in node), 'invalid_schema', 'Exactly one content representation')
        for parent in node['parents']:
            require(parent in nodes, 'missing_reference', 'Parents precede children; no cycles')
        if 'text' in node:
            require(digest(node['text'].encode()) == node['digest'], 'digest_mismatch', 'Modified node bytes')
        else:
            require(node['digest'] == node['artifact']['digest'], 'digest_mismatch', 'Artifact digest mismatch')
        if 'source_span' in node:
            span = node['source_span']
            length = len(node['text'].encode()) if 'text' in node else node['artifact']['byte_length']
            require('source' in node and span['end'] >= span['start'] and span['end'] - span['start'] == length,
                    'invalid_offset', 'Source excerpt length mismatch')
        kind = node['derivation']
        exact = kind in ('selection', 'normalization', 'concatenation')
        if kind == 'root':
            require(not node['parents'] and not node['mappings'], 'invalid_mapping', 'Root has no derivation')
        elif kind == 'opaque':
            require(node['coverage'] == 'opaque' and not node['mappings'], 'invalid_mapping', 'Opaque coverage required')
        else:
            require(bool(node['parents']), 'missing_reference', 'Derivation needs genealogy')
        if kind in ('summary', 'model_output', 'tool_output'):
            require('receipt' in node and node['coverage'] != 'complete' and not node['mappings'],
                    'invalid_mapping', 'Declared genealogy is not exact or semantic support')
        if not exact:
            require(not node['mappings'], 'invalid_mapping', 'Only verified transformations carry mappings')
            if kind in ('redaction', 'rendering'):
                require(node['coverage'] != 'complete', 'invalid_mapping', 'Unverified transformation')
        end = 0
        for mapping in node['mappings']:
            require(mapping['parent'] in node['parents'], 'missing_reference', 'Mapping needs declared parent')
            parent = nodes[mapping['parent']]
            require('text' in parent and 'text' in node, 'invalid_mapping', 'Text mapping required')
            left = span_bytes(parent['text'], mapping['input'])
            right = span_bytes(node['text'], mapping['output'])
            if kind == 'normalization':
                left = unicodedata.normalize('NFC', left.decode().replace('\r\n', '\n')).encode()
            require(left == right and mapping['output']['start'] == end,
                    'invalid_mapping', 'Transformation bytes or ordered coverage mismatch')
            end = mapping['output']['end']
        if exact:
            require('text' in node and end == len(node['text'].encode()) and node['coverage'] == 'complete',
                    'invalid_mapping', 'Exact derivation must cover every output byte')
        nodes[node['id']] = node
    productions = {}
    for receipt in value['productions']:
        require(receipt['id'] not in productions, 'invalid_schema', 'Duplicate production receipt')
        require(receipt['output'] in nodes and receipt['response'] in nodes and
                all(p in nodes for p in receipt['inputs']), 'missing_reference', 'Unknown production reference')
        output = nodes[receipt['output']]
        require(output.get('receipt') == receipt['id'] and output['parents'] == receipt['inputs']
                and receipt['kind'] == ('tool' if output['derivation'] == 'tool_output' else 'model'),
                'invalid_mapping', 'Producer and declared genealogy differ')
        productions[receipt['id']] = receipt
    for node in nodes.values():
        if node['derivation'] in ('summary', 'model_output', 'tool_output'):
            require(node['receipt'] in productions and
                    productions[node['receipt']]['output'] == node['id'],
                    'missing_reference', 'Missing typed production receipt')
    request = value['request']
    slots = set()
    for part in request['parts']:
        require(part['slot'] not in slots, 'invalid_schema', 'Duplicate request slot')
        slots.add(part['slot'])
        require(('text' in part) != ('artifact' in part), 'invalid_schema', 'Exactly one request content representation')
        end = 0
        for segment in part['segments']:
            require(segment['node'] in nodes, 'missing_reference', 'Unknown request reference')
            node = nodes[segment['node']]
            require(segment['span']['start'] == end, 'uncovered_bytes', 'Coverage must be contiguous and ordered')
            if 'text' in part and 'text' in node:
                require(span_bytes(part['text'], segment['span']) == span_bytes(node['text'], segment['node_span']),
                        'modified_bytes', 'Request bytes differ from reference')
            else:
                require('artifact' in part and 'artifact' in node and part['artifact'] == node['artifact'],
                        'modified_bytes', 'Artifact descriptor differs')
                for span in (segment['span'], segment['node_span']):
                    require(span['start'] == 0 and span['end'] == node['artifact']['byte_length'],
                            'invalid_offset', 'Artifact reference covers entire artifact')
            end = segment['span']['end']
        length = len(part['text'].encode()) if 'text' in part else part['artifact']['byte_length']
        require(end == length, 'uncovered_bytes', 'Every host-controlled byte needs lineage')
    captures = set()
    for receipt in value['captures']:
        require(receipt['node'] in nodes, 'missing_reference', 'Unknown capture reference')
        require(receipt['id'] not in captures, 'invalid_schema', 'Duplicate capture receipt')
        captures.add(receipt['id'])
    states, last_receipts, keys = {}, {}, Idempotency()
    transitions = {'prepared': {'transport_started', 'failed', 'cancelled_uncertain'},
                   'transport_started': {'provider_confirmed', 'failed', 'cancelled_uncertain'}}
    for receipt in value['deliveries']:
        require(receipt['request'] == request['id'] and receipt['assembly_digest'] == assembly_digest(request),
                'modified_bytes', 'Receipt must bind exact host request assembly')
        key = receipt['idempotency_key']
        keys.register(key, receipt['assembly_digest'] + ':' + receipt['attempt'])
        old, state = states.get(key), receipt['state']
        require((old is None and state == 'prepared') or state == old or state in transitions.get(old, set()),
                'invalid_delivery', 'Invalid delivery transition')
        require(state != old or receipt == last_receipts[key],
                'invalid_delivery', 'Receipt replay must preserve every field')
        require(('provider_receipt' in receipt) == (state == 'provider_confirmed'),
                'invalid_delivery', 'Provider confirmation needs observable receipt')
        states[key] = state
        last_receipts[key] = receipt
    handles = {}
    for handle in value['handles']:
        require(handle['id'] not in handles, 'invalid_schema', 'Duplicate handle')
        validate_handle(handle)
        require(any(n.get('source', {}).get('id') == handle['source']['id'] and
                    n.get('source', {}).get('revision') == handle['source']['revision']
                    for n in nodes.values()), 'missing_reference', 'Handle source revision must be retained')
        handles[handle['id']] = handle
    for annotation in value['annotations']:
        require(annotation['evidence'] in nodes and all(p in nodes for p in annotation['parents']),
                'missing_reference', 'Annotation evidence is immutable context')
        if 'location' in annotation:
            require(annotation['location'] in handles, 'missing_reference', 'Unknown typed location handle')
            bound = handles[annotation['location']]
            require(all(k not in bound or annotation.get(k) == bound[k]
                        for k in ('run', 'stop', 'frame', 'invocation')),
                    'stale_handle', 'Annotation and location execution scopes differ')
        if annotation['kind'] in ('paused', 'values', 'expression'):
            require(all(k in annotation for k in ('run', 'stop', 'frame', 'invocation', 'location')),
                    'invalid_schema', 'Debugger annotation needs captured execution scope')
        if annotation['kind'] == 'profile':
            require('metric' in annotation and 'run' in annotation, 'invalid_schema', 'Profile needs measurement semantics')
    return cast(Bundle, value)
</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/context_protocol.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/context_protocol.py"><storycode:paste select="#f-4922a914e9e6-expression_statement"/><storycode:paste select="#f-4922a914e9e6-sha256"/><storycode:paste select="#f-4922a914e9e6-files"/><storycode:paste select="#f-4922a914e9e6-json"/><storycode:paste select="#f-4922a914e9e6-unicodedata"/><storycode:paste select="#f-4922a914e9e6-cast"/><storycode:paste select="#f-4922a914e9e6-Draft202012Validator"/><storycode:paste select="#f-4922a914e9e6-Bundle"/><storycode:paste select="#f-4922a914e9e6-expression_statement-2"/><storycode:paste select="#f-4922a914e9e6-expression_statement-3"/><storycode:paste select="#f-4922a914e9e6-expression_statement-4"/><storycode:paste select="#f-4922a914e9e6-expression_statement-5"/><storycode:paste select="#f-4922a914e9e6-ProtocolFailure"/><storycode:paste select="#f-4922a914e9e6-require"/><storycode:paste select="#f-4922a914e9e6-validate"/><storycode:paste select="#f-4922a914e9e6-digest"/><storycode:paste select="#f-4922a914e9e6-assembly_digest"/><storycode:paste select="#f-4922a914e9e6-span_bytes"/><storycode:paste select="#f-4922a914e9e6-utf16_to_utf8"/><storycode:paste select="#f-4922a914e9e6-negotiate"/><storycode:paste select="#f-4922a914e9e6-validate_frame"/><storycode:paste select="#f-4922a914e9e6-validate_page"/><storycode:paste select="#f-4922a914e9e6-validate_handle"/><storycode:paste select="#f-4922a914e9e6-resolve_handle"/><storycode:paste select="#f-4922a914e9e6-Idempotency"/><storycode:paste select="#f-4922a914e9e6-validate_bundle"/></storycode:file>

<a id="output-62037abb921c"></a>
## `apps/agent/src/hickory_agent_worker/context_types.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Context protocol validates declared byte lineage before integration](contract.md).

<a id="f-62037abb921c-expression_statement"></a>
### `expression_statement`

Generated by scripts/generate-context-protocol.py; do not edit.

<storycode:copy id="f-62037abb921c-expression_statement">
# Generated by scripts/generate-context-protocol.py; do not edit.
from __future__ import annotations
from typing import Literal, NotRequired, TypedDict
ErrorCode = Literal['invalid_schema', 'version_skew', 'capability_missing', 'frame_limit', 'missing_reference', 'digest_mismatch', 'invalid_mapping', 'uncovered_bytes', 'modified_bytes', 'invalid_offset', 'cursor_expired', 'invalid_page', 'idempotency_conflict', 'invalid_delivery', 'stale_handle', 'cancelled', 'mismatched_response']

</storycode:copy>
<a id="f-62037abb921c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-62037abb921c-expression_statement-2">
Operation = Literal['context.capture', 'context.derive', 'context.changes', 'context.compare', 'context.resolve', 'inference']

</storycode:copy>
<a id="f-62037abb921c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-62037abb921c-expression_statement-3">
DerivationKind = Literal['root', 'selection', 'normalization', 'concatenation', 'redaction', 'rendering', 'summary', 'model_output', 'tool_output', 'opaque']

</storycode:copy>
<a id="f-62037abb921c-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-62037abb921c-expression_statement-4">
DeliveryState = Literal['prepared', 'transport_started', 'provider_confirmed', 'failed', 'cancelled_uncertain']

</storycode:copy>
<a id="f-62037abb921c-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-62037abb921c-expression_statement-5">
HandleKind = Literal['location', 'stop', 'frame', 'invocation']

</storycode:copy>
<a id="f-62037abb921c-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-62037abb921c-expression_statement-6">
AnnotationKind = Literal['log', 'stdout', 'exception', 'breakpoint_requested', 'breakpoint_bound', 'paused', 'values', 'expression', 'profile', 'external_review']

</storycode:copy>
<a id="f-62037abb921c-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-62037abb921c-expression_statement-7">
Precision = Literal['exact', 'span', 'block', 'unknown']

</storycode:copy>
<a id="f-62037abb921c-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-62037abb921c-expression_statement-8">
ComparisonState = Literal['unchanged', 'moved', 'changed', 'ambiguous', 'unavailable', 'cannot_check', 'invalidated']

</storycode:copy>
<a id="f-62037abb921c-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-62037abb921c-expression_statement-9">
Coverage = Literal['complete', 'partial', 'opaque']

</storycode:copy>
<a id="f-62037abb921c-Span"></a>
### `Span`

<storycode:copy id="f-62037abb921c-Span">
class Span(TypedDict):
    start: int
    end: int
    unit: Literal['utf8_bytes']

</storycode:copy>
<a id="f-62037abb921c-Source"></a>
### `Source`

<storycode:copy id="f-62037abb921c-Source">
class Source(TypedDict):
    id: str
    revision: str
    locator: str
    kind: str

</storycode:copy>
<a id="f-62037abb921c-Artifact"></a>
### `Artifact`

<storycode:copy id="f-62037abb921c-Artifact">
class Artifact(TypedDict):
    digest: str
    media_type: str
    byte_length: int
    locator: str

</storycode:copy>
<a id="f-62037abb921c-Mapping"></a>
### `Mapping`

<storycode:copy id="f-62037abb921c-Mapping">
class Mapping(TypedDict):
    parent: str
    input: Span
    output: Span

</storycode:copy>
<a id="f-62037abb921c-Node"></a>
### `Node`

<storycode:copy id="f-62037abb921c-Node">
class Node(TypedDict):
    id: str
    digest: str
    text: NotRequired[str]
    artifact: NotRequired[Artifact]
    source: NotRequired[Source]
    source_span: NotRequired[Span]
    parents: list[str]
    derivation: DerivationKind
    coverage: Coverage
    mappings: list[Mapping]
    receipt: NotRequired[str]

</storycode:copy>
<a id="f-62037abb921c-Segment"></a>
### `Segment`

<storycode:copy id="f-62037abb921c-Segment">
class Segment(TypedDict):
    span: Span
    node: str
    node_span: Span

</storycode:copy>
<a id="f-62037abb921c-RequestPart"></a>
### `RequestPart`

<storycode:copy id="f-62037abb921c-RequestPart">
class RequestPart(TypedDict):
    slot: str
    role: str
    scope: str
    text: NotRequired[str]
    artifact: NotRequired[Artifact]
    segments: list[Segment]

</storycode:copy>
<a id="f-62037abb921c-Request"></a>
### `Request`

<storycode:copy id="f-62037abb921c-Request">
class Request(TypedDict):
    id: str
    branch: str
    policy: str
    policy_version: str
    observed_cursor: str
    parts: list[RequestPart]
    provider_coverage: Literal['host_controlled_only']

</storycode:copy>
<a id="f-62037abb921c-Hello"></a>
### `Hello`

<storycode:copy id="f-62037abb921c-Hello">
class Hello(TypedDict):
    versions: list[int]
    capabilities: list[Operation]
    transport: Literal['cursor_polling']
    max_frame_bytes: int
    max_page_items: int

</storycode:copy>
<a id="f-62037abb921c-Negotiated"></a>
### `Negotiated`

<storycode:copy id="f-62037abb921c-Negotiated">
class Negotiated(TypedDict):
    version: int
    capabilities: list[Operation]
    transport: Literal['cursor_polling']
    max_frame_bytes: int
    max_page_items: int

</storycode:copy>
<a id="f-62037abb921c-CaptureReceipt"></a>
### `CaptureReceipt`

<storycode:copy id="f-62037abb921c-CaptureReceipt">
class CaptureReceipt(TypedDict):
    id: str
    node: str
    captured_at: str
    state: Literal['captured']

</storycode:copy>
<a id="f-62037abb921c-DeliveryReceipt"></a>
### `DeliveryReceipt`

<storycode:copy id="f-62037abb921c-DeliveryReceipt">
class DeliveryReceipt(TypedDict):
    request: str
    attempt: str
    idempotency_key: str
    assembly_digest: str
    state: DeliveryState
    provider_receipt: NotRequired[str]

</storycode:copy>
<a id="f-62037abb921c-Cursor"></a>
### `Cursor`

<storycode:copy id="f-62037abb921c-Cursor">
class Cursor(TypedDict):
    epoch: str
    branch: str
    position: int

</storycode:copy>
<a id="f-62037abb921c-Change"></a>
### `Change`

<storycode:copy id="f-62037abb921c-Change">
class Change(TypedDict):
    sequence: int
    source: str
    old_revision: NotRequired[str]
    new_revision: NotRequired[str]
    state: ComparisonState

</storycode:copy>
<a id="f-62037abb921c-ChangePage"></a>
### `ChangePage`

<storycode:copy id="f-62037abb921c-ChangePage">
class ChangePage(TypedDict):
    from_cursor: Cursor
    next_cursor: Cursor
    events: list[Change]
    has_more: bool
    limit: int

</storycode:copy>
<a id="f-62037abb921c-Handle"></a>
### `Handle`

<storycode:copy id="f-62037abb921c-Handle">
class Handle(TypedDict):
    id: str
    kind: HandleKind
    source: Source
    span: Span
    run: NotRequired[str]
    stop: NotRequired[str]
    frame: NotRequired[str]
    invocation: NotRequired[str]

</storycode:copy>
<a id="f-62037abb921c-Metric"></a>
### `Metric`

<storycode:copy id="f-62037abb921c-Metric">
class Metric(TypedDict):
    name: str
    value: float
    unit: str
    clock: str
    semantics: str
    aggregation: str
    interval_start: str
    interval_end: str

</storycode:copy>
<a id="f-62037abb921c-Annotation"></a>
### `Annotation`

<storycode:copy id="f-62037abb921c-Annotation">
class Annotation(TypedDict):
    id: str
    kind: AnnotationKind
    provider: str
    provider_version: str
    evidence: str
    source_precision: Precision
    location: NotRequired[str]
    run: NotRequired[str]
    stop: NotRequired[str]
    frame: NotRequired[str]
    invocation: NotRequired[str]
    metric: NotRequired[Metric]
    coverage: Coverage
    parents: list[str]
    payload_schema: str
    payload: dict[str, object]

</storycode:copy>
<a id="f-62037abb921c-Bundle"></a>
### `Bundle`

<storycode:copy id="f-62037abb921c-Bundle">
class Bundle(TypedDict):
    version: Literal[1]
    nodes: list[Node]
    request: Request
    captures: list[CaptureReceipt]
    deliveries: list[DeliveryReceipt]
    handles: list[Handle]
    annotations: list[Annotation]
    productions: list[ProductionReceipt]

</storycode:copy>
<a id="f-62037abb921c-CaptureRequest"></a>
### `CaptureRequest`

<storycode:copy id="f-62037abb921c-CaptureRequest">
class CaptureRequest(TypedDict):
    idempotency_key: str
    source: Source
    span: NotRequired[Span]
    max_bytes: int

</storycode:copy>
<a id="f-62037abb921c-DeriveRequest"></a>
### `DeriveRequest`

<storycode:copy id="f-62037abb921c-DeriveRequest">
class DeriveRequest(TypedDict):
    idempotency_key: str
    node: Node

</storycode:copy>
<a id="f-62037abb921c-ChangesRequest"></a>
### `ChangesRequest`

<storycode:copy id="f-62037abb921c-ChangesRequest">
class ChangesRequest(TypedDict):
    cursor: Cursor
    references: list[str]
    limit: int

</storycode:copy>
<a id="f-62037abb921c-CompareRequest"></a>
### `CompareRequest`

<storycode:copy id="f-62037abb921c-CompareRequest">
class CompareRequest(TypedDict):
    old: str
    current: str

</storycode:copy>
<a id="f-62037abb921c-Comparison"></a>
### `Comparison`

<storycode:copy id="f-62037abb921c-Comparison">
class Comparison(TypedDict):
    old: str
    current: NotRequired[str]
    state: ComparisonState
    mappings: list[Mapping]

</storycode:copy>
<a id="f-62037abb921c-ResolveRequest"></a>
### `ResolveRequest`

<storycode:copy id="f-62037abb921c-ResolveRequest">
class ResolveRequest(TypedDict):
    handle: str
    expected_source: Source
    run: NotRequired[str]
    stop: NotRequired[str]
    frame: NotRequired[str]
    invocation: NotRequired[str]

</storycode:copy>
<a id="f-62037abb921c-ProtocolError"></a>
### `ProtocolError`

<storycode:copy id="f-62037abb921c-ProtocolError">
class ProtocolError(TypedDict):
    code: ErrorCode
    message: str
    request_id: NotRequired[str]
    retryable: bool

</storycode:copy>
<a id="f-62037abb921c-ProductionReceipt"></a>
### `ProductionReceipt`

<storycode:copy id="f-62037abb921c-ProductionReceipt">
class ProductionReceipt(TypedDict):
    id: str
    kind: str
    request: str
    response: str
    inputs: list[str]
    output: str
</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/context_types.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/context_types.py"><storycode:paste select="#f-62037abb921c-expression_statement"/><storycode:paste select="#f-62037abb921c-expression_statement-2"/><storycode:paste select="#f-62037abb921c-expression_statement-3"/><storycode:paste select="#f-62037abb921c-expression_statement-4"/><storycode:paste select="#f-62037abb921c-expression_statement-5"/><storycode:paste select="#f-62037abb921c-expression_statement-6"/><storycode:paste select="#f-62037abb921c-expression_statement-7"/><storycode:paste select="#f-62037abb921c-expression_statement-8"/><storycode:paste select="#f-62037abb921c-expression_statement-9"/><storycode:paste select="#f-62037abb921c-Span"/><storycode:paste select="#f-62037abb921c-Source"/><storycode:paste select="#f-62037abb921c-Artifact"/><storycode:paste select="#f-62037abb921c-Mapping"/><storycode:paste select="#f-62037abb921c-Node"/><storycode:paste select="#f-62037abb921c-Segment"/><storycode:paste select="#f-62037abb921c-RequestPart"/><storycode:paste select="#f-62037abb921c-Request"/><storycode:paste select="#f-62037abb921c-Hello"/><storycode:paste select="#f-62037abb921c-Negotiated"/><storycode:paste select="#f-62037abb921c-CaptureReceipt"/><storycode:paste select="#f-62037abb921c-DeliveryReceipt"/><storycode:paste select="#f-62037abb921c-Cursor"/><storycode:paste select="#f-62037abb921c-Change"/><storycode:paste select="#f-62037abb921c-ChangePage"/><storycode:paste select="#f-62037abb921c-Handle"/><storycode:paste select="#f-62037abb921c-Metric"/><storycode:paste select="#f-62037abb921c-Annotation"/><storycode:paste select="#f-62037abb921c-Bundle"/><storycode:paste select="#f-62037abb921c-CaptureRequest"/><storycode:paste select="#f-62037abb921c-DeriveRequest"/><storycode:paste select="#f-62037abb921c-ChangesRequest"/><storycode:paste select="#f-62037abb921c-CompareRequest"/><storycode:paste select="#f-62037abb921c-Comparison"/><storycode:paste select="#f-62037abb921c-ResolveRequest"/><storycode:paste select="#f-62037abb921c-ProtocolError"/><storycode:paste select="#f-62037abb921c-ProductionReceipt"/></storycode:file>

<a id="output-48176c4779fa"></a>
## `apps/agent/tests/test_context_protocol.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Context protocol validates declared byte lineage before integration](contract.md).

<a id="f-48176c4779fa-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-48176c4779fa-expression_statement">
"""Shared contract witnesses, independent of Monty, providers and agent policy."""
</storycode:copy>
<a id="f-48176c4779fa-json"></a>
### `json`

<storycode:copy id="f-48176c4779fa-json">
import json
</storycode:copy>
<a id="f-48176c4779fa-Path"></a>
### `Path`

<storycode:copy id="f-48176c4779fa-Path">
from pathlib import Path
</storycode:copy>
<a id="f-48176c4779fa-unittest"></a>
### `unittest`

<storycode:copy id="f-48176c4779fa-unittest">
import unittest
</storycode:copy>
<a id="f-48176c4779fa-context_protocol-as-p"></a>
### `context_protocol as p`

<storycode:copy id="f-48176c4779fa-context_protocol-as-p">
from hickory_agent_worker import context_protocol as p

</storycode:copy>
<a id="f-48176c4779fa-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-48176c4779fa-expression_statement-2">
FIXTURES = Path(__file__).resolve().parents[3] / 'protocols/context-v1/fixtures.json'


</storycode:copy>
<a id="f-48176c4779fa-run_case"></a>
### `run_case`

<storycode:copy id="f-48176c4779fa-run_case">
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


</storycode:copy>
<a id="f-48176c4779fa-ContextProtocolTests"></a>
### `ContextProtocolTests`

<storycode:copy id="f-48176c4779fa-ContextProtocolTests">
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
</storycode:copy>

### Assemble `apps/agent/tests/test_context_protocol.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/tests/test_context_protocol.py"><storycode:paste select="#f-48176c4779fa-expression_statement"/><storycode:paste select="#f-48176c4779fa-json"/><storycode:paste select="#f-48176c4779fa-Path"/><storycode:paste select="#f-48176c4779fa-unittest"/><storycode:paste select="#f-48176c4779fa-context_protocol-as-p"/><storycode:paste select="#f-48176c4779fa-expression_statement-2"/><storycode:paste select="#f-48176c4779fa-run_case"/><storycode:paste select="#f-48176c4779fa-ContextProtocolTests"/></storycode:file>

<a id="output-1c697d40a8e6"></a>
## `crates/hick-context-protocol/src/lib.rs`

Validate the host/worker wire contract. Selection and refresh policy remain in the Python worker rather than this validation layer.

Related contracts: [Context protocol validates declared byte lineage before integration](contract.md).

<a id="f-1c697d40a8e6-types"></a>
### `types`

Context v1 validation only. Python owns selection and refresh policy.
This crate performs no capture, persistence, provider delivery or authorization.

<storycode:copy id="f-1c697d40a8e6-types">
//! Context v1 validation only. Python owns selection and refresh policy.
//! This crate performs no capture, persistence, provider delivery or authorization.
mod types;
</storycode:copy>
<a id="f-1c697d40a8e6-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-1c697d40a8e6-use_declaration">
use serde_json::{Value, json};
</storycode:copy>
<a id="f-1c697d40a8e6-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-1c697d40a8e6-use_declaration-2">
use sha2::{Digest, Sha256};
</storycode:copy>
<a id="f-1c697d40a8e6-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-1c697d40a8e6-use_declaration-3">
use std::collections::{HashMap, HashSet};
</storycode:copy>
<a id="f-1c697d40a8e6-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-1c697d40a8e6-use_declaration-4">
pub use types::*;
</storycode:copy>
<a id="f-1c697d40a8e6-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-1c697d40a8e6-use_declaration-5">
use unicode_normalization::UnicodeNormalization;

</storycode:copy>
<a id="f-1c697d40a8e6-MAX_FRAME"></a>
### `MAX_FRAME`

<storycode:copy id="f-1c697d40a8e6-MAX_FRAME">
pub const MAX_FRAME: usize = 1_048_576;
</storycode:copy>
<a id="f-1c697d40a8e6-MAX_PAGE"></a>
### `MAX_PAGE`

<storycode:copy id="f-1c697d40a8e6-MAX_PAGE">
pub const MAX_PAGE: u64 = 256;
</storycode:copy>
<a id="f-1c697d40a8e6-SCHEMA"></a>
### `SCHEMA`

<storycode:copy id="f-1c697d40a8e6-SCHEMA">
pub const SCHEMA: &str = include_str!("../../../protocols/context-v1/schema.json");
</storycode:copy>
<a id="f-1c697d40a8e6-Failure"></a>
### `Failure`

<storycode:copy id="f-1c697d40a8e6-Failure">
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub code: ErrorCode,
    pub message: String,
}
</storycode:copy>
<a id="f-1c697d40a8e6-impl-std-fmt-Display-for-Failure"></a>
### `impl-std::fmt::Display for Failure`

<storycode:copy id="f-1c697d40a8e6-impl-std-fmt-Display-for-Failure">
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
</storycode:copy>
<a id="f-1c697d40a8e6-impl-std-error-Error-for-Failure"></a>
### `impl-std::error::Error for Failure`

<storycode:copy id="f-1c697d40a8e6-impl-std-error-Error-for-Failure">
impl std::error::Error for Failure {}
</storycode:copy>
<a id="f-1c697d40a8e6-Result"></a>
### `Result`

<storycode:copy id="f-1c697d40a8e6-Result">
pub type Result<T> = std::result::Result<T, Failure>;
</storycode:copy>
<a id="f-1c697d40a8e6-require"></a>
### `require`

<storycode:copy id="f-1c697d40a8e6-require">
fn require(ok: bool, code: ErrorCode, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Failure {
            code,
            message: message.into(),
        })
    }
}
</storycode:copy>
<a id="f-1c697d40a8e6-list"></a>
### `list`

<storycode:copy id="f-1c697d40a8e6-list">
fn list(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or(&[])
}
</storycode:copy>
<a id="f-1c697d40a8e6-has"></a>
### `has`

<storycode:copy id="f-1c697d40a8e6-has">
fn has(value: &Value, key: &str) -> bool {
    value.get(key).is_some()
}
</storycode:copy>
<a id="f-1c697d40a8e6-num"></a>
### `num`

<storycode:copy id="f-1c697d40a8e6-num">
fn num(value: &Value) -> u64 {
    value.as_u64().unwrap_or(0)
}
</storycode:copy>
<a id="f-1c697d40a8e6-text"></a>
### `text`

<storycode:copy id="f-1c697d40a8e6-text">
fn text(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}
</storycode:copy>
<a id="f-1c697d40a8e6-validate"></a>
### `validate`

Validate any generated wire type by its schema definition name.

<storycode:copy id="f-1c697d40a8e6-validate">
/// Validate any generated wire type by its schema definition name.
pub fn validate(value: &Value, name: &str) -> Result<()> {
    let mut schema: Value = serde_json::from_str(SCHEMA).expect("checked-in schema");
    require(
        schema["$defs"].get(name).is_some(),
        ErrorCode::InvalidSchema,
        "Unknown schema type",
    )?;
    schema["$ref"] = json!(format!("#/$defs/{name}"));
    let validator = jsonschema::validator_for(&schema).expect("valid checked-in schema");
    require(
        validator.is_valid(value) && types::deserializes(name, value),
        ErrorCode::InvalidSchema,
        "Schema validation failed",
    )
}
</storycode:copy>
<a id="f-1c697d40a8e6-digest"></a>
### `digest`

<storycode:copy id="f-1c697d40a8e6-digest">
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
</storycode:copy>
<a id="f-1c697d40a8e6-assembly_digest"></a>
### `assembly_digest`

Request objects have only integers, strings, arrays and records. serde_json's
ordered map encoding matches Python's sorted, compact, UTF-8 JSON encoding.

<storycode:copy id="f-1c697d40a8e6-assembly_digest">
/// Request objects have only integers, strings, arrays and records. serde_json's
/// ordered map encoding matches Python's sorted, compact, UTF-8 JSON encoding.
pub fn assembly_digest(request: &Value) -> String {
    digest(&serde_json::to_vec(request).expect("JSON value"))
}
</storycode:copy>
<a id="f-1c697d40a8e6-span_bytes"></a>
### `span_bytes`

<storycode:copy id="f-1c697d40a8e6-span_bytes">
fn span_bytes<'a>(s: &'a str, span: &Value) -> Result<&'a [u8]> {
    let (a, b) = (num(&span["start"]), num(&span["end"]));
    require(
        a <= b && b <= s.len() as u64,
        ErrorCode::InvalidOffset,
        "Invalid UTF-8 byte range",
    )?;
    require(
        s.is_char_boundary(a as usize) && s.is_char_boundary(b as usize),
        ErrorCode::InvalidOffset,
        "Split UTF-8 code point",
    )?;
    Ok(&s.as_bytes()[a as usize..b as usize])
}
</storycode:copy>
<a id="f-1c697d40a8e6-utf16_to_utf8"></a>
### `utf16_to_utf8`

<storycode:copy id="f-1c697d40a8e6-utf16_to_utf8">
pub fn utf16_to_utf8(s: &str, offset: u64) -> Result<usize> {
    let mut units = 0;
    for (byte, c) in s.char_indices() {
        if units == offset {
            return Ok(byte);
        }
        units += c.len_utf16() as u64;
    }
    require(
        units == offset,
        ErrorCode::InvalidOffset,
        "Split surrogate pair or out of range",
    )?;
    Ok(s.len())
}
</storycode:copy>
<a id="f-1c697d40a8e6-negotiate"></a>
### `negotiate`

<storycode:copy id="f-1c697d40a8e6-negotiate">
pub fn negotiate(host: &Value, worker: &Value) -> Result<Negotiated> {
    validate(host, "Hello")?;
    validate(worker, "Hello")?;
    require(
        list(&host["versions"]).contains(&json!(1))
            && list(&worker["versions"]).contains(&json!(1)),
        ErrorCode::VersionSkew,
        "No context v1",
    )?;
    let mut capabilities: Vec<Value> = list(&host["capabilities"])
        .iter()
        .filter(|c| list(&worker["capabilities"]).contains(c))
        .cloned()
        .collect();
    capabilities.sort_by(|a, b| text(a).cmp(text(b)));
    capabilities.dedup();
    require(
        capabilities.contains(&json!("context.changes")),
        ErrorCode::CapabilityMissing,
        "Polling required",
    )?;
    let frame = num(&host["max_frame_bytes"])
        .min(num(&worker["max_frame_bytes"]))
        .min(MAX_FRAME as u64);
    let page = num(&host["max_page_items"])
        .min(num(&worker["max_page_items"]))
        .min(MAX_PAGE);
    require(
        frame > 0 && page > 0,
        ErrorCode::FrameLimit,
        "Zero negotiated limit",
    )?;
    Ok(serde_json::from_value(json!({"version":1,"capabilities":capabilities,"transport":"cursor_polling","max_frame_bytes":frame,"max_page_items":page})).expect("negotiated type"))
}
</storycode:copy>
<a id="f-1c697d40a8e6-validate_frame"></a>
### `validate_frame`

<storycode:copy id="f-1c697d40a8e6-validate_frame">
pub fn validate_frame(raw: &[u8], limit: usize, expected_id: Option<&Value>) -> Result<Value> {
    require(
        raw.len() <= limit.min(MAX_FRAME)
            && raw.ends_with(b"\n")
            && !raw[..raw.len().saturating_sub(1)].contains(&b'\n'),
        ErrorCode::FrameLimit,
        "One bounded newline-terminated JSON frame required",
    )?;
    let value: Value = serde_json::from_slice(raw).map_err(|_| Failure {
        code: ErrorCode::InvalidSchema,
        message: "Invalid JSON frame".into(),
    })?;
    require(
        value.is_object(),
        ErrorCode::InvalidSchema,
        "Frame must be object",
    )?;
    if let Some(id) = expected_id {
        require(
            value.get("id") == Some(id) && (has(&value, "result") != has(&value, "error")),
            ErrorCode::MismatchedResponse,
            "Unsolicited or mismatched response",
        )?;
    }
    Ok(value)
}
</storycode:copy>
<a id="f-1c697d40a8e6-validate_page"></a>
### `validate_page`

<storycode:copy id="f-1c697d40a8e6-validate_page">
pub fn validate_page(page: &Value, epoch: &str, branch: &str) -> Result<ChangePage> {
    validate(page, "ChangePage")?;
    let (a, b) = (&page["from_cursor"], &page["next_cursor"]);
    require(
        text(&a["epoch"]) == epoch
            && text(&b["epoch"]) == epoch
            && text(&a["branch"]) == branch
            && text(&b["branch"]) == branch,
        ErrorCode::CursorExpired,
        "Reconcile after epoch/branch change",
    )?;
    let events = list(&page["events"]);
    require(
        num(&page["limit"]) > 0
            && num(&page["limit"]) <= MAX_PAGE
            && events.len() as u64 <= num(&page["limit"]),
        ErrorCode::InvalidPage,
        "Invalid page limit",
    )?;
    let mut sequence = num(&a["position"]);
    for event in events {
        require(
            num(&event["sequence"]) > sequence,
            ErrorCode::InvalidPage,
            "Events must advance in order",
        )?;
        sequence = num(&event["sequence"]);
    }
    require(
        num(&b["position"]) == sequence && (page["has_more"] != true || !events.is_empty()),
        ErrorCode::InvalidPage,
        "Cursor must describe last event",
    )?;
    Ok(serde_json::from_value(page.clone()).expect("validated page"))
}
</storycode:copy>
<a id="f-1c697d40a8e6-validate_handle"></a>
### `validate_handle`

<storycode:copy id="f-1c697d40a8e6-validate_handle">
pub fn validate_handle(handle: &Value) -> Result<()> {
    validate(handle, "Handle")?;
    require(
        num(&handle["span"]["start"]) <= num(&handle["span"]["end"]),
        ErrorCode::InvalidOffset,
        "Invalid handle span",
    )?;
    let required: &[&str] = match text(&handle["kind"]) {
        "stop" => &["run", "stop"],
        "frame" => &["run", "stop", "frame"],
        "invocation" => &["run", "invocation"],
        _ => &[],
    };
    require(
        required.iter().all(|k| has(handle, k)),
        ErrorCode::InvalidSchema,
        "Missing handle execution scope",
    )
}
</storycode:copy>
<a id="f-1c697d40a8e6-resolve_handle"></a>
### `resolve_handle`

<storycode:copy id="f-1c697d40a8e6-resolve_handle">
pub fn resolve_handle(handle: &Value, request: &Value) -> Result<Handle> {
    validate_handle(handle)?;
    validate(request, "ResolveRequest")?;
    require(
        request["handle"] == handle["id"]
            && ["id", "revision"]
                .iter()
                .all(|k| request["expected_source"][k] == handle["source"][k])
            && ["run", "stop", "frame", "invocation"]
                .iter()
                .all(|k| request.get(k) == handle.get(k)),
        ErrorCode::StaleHandle,
        "Rust must resolve current source and execution scope",
    )?;
    Ok(serde_json::from_value(handle.clone()).expect("validated handle"))
}
</storycode:copy>
<a id="f-1c697d40a8e6-Idempotency"></a>
### `Idempotency`

<storycode:copy id="f-1c697d40a8e6-Idempotency">
#[derive(Default)]
pub struct Idempotency {
    entries: HashMap<String, String>,
}
</storycode:copy>
<a id="f-1c697d40a8e6-impl-Idempotency"></a>
### `impl-Idempotency`

<storycode:copy id="f-1c697d40a8e6-impl-Idempotency">
impl Idempotency {
    /// True means new; false means replay. Persist with session receipts in #49/50.
    pub fn register(&mut self, key: &str, fingerprint: &str) -> Result<bool> {
        if let Some(old) = self.entries.get(key) {
            require(
                old == fingerprint,
                ErrorCode::IdempotencyConflict,
                "Key reused for different bytes",
            )?;
            return Ok(false);
        }
        self.entries.insert(key.into(), fingerprint.into());
        Ok(true)
    }
}
</storycode:copy>
<a id="f-1c697d40a8e6-validate_bundle"></a>
### `validate_bundle`

Validate a self-contained, topologically ordered root-to-request witness.
External artifact bytes are not fetched; only their immutable descriptors
are checked. This cannot establish authenticity or semantic support.

<storycode:copy id="f-1c697d40a8e6-validate_bundle">
/// Validate a self-contained, topologically ordered root-to-request witness.
/// External artifact bytes are not fetched; only their immutable descriptors
/// are checked. This cannot establish authenticity or semantic support.
pub fn validate_bundle(value: &Value) -> Result<Bundle> {
    require(
        value["version"] == 1,
        ErrorCode::VersionSkew,
        "Unsupported context version",
    )?;
    validate(value, "Bundle")?;
    let mut nodes: HashMap<&str, &Value> = HashMap::new();
    for node in list(&value["nodes"]) {
        require(
            !nodes.contains_key(text(&node["id"])),
            ErrorCode::InvalidSchema,
            "Duplicate immutable reference",
        )?;
        require(
            has(node, "text") != has(node, "artifact"),
            ErrorCode::InvalidSchema,
            "Exactly one content representation",
        )?;
        for parent in list(&node["parents"]) {
            require(
                nodes.contains_key(text(parent)),
                ErrorCode::MissingReference,
                "Parents precede children; no cycles",
            )?;
        }
        if has(node, "text") {
            require(
                digest(text(&node["text"]).as_bytes()) == text(&node["digest"]),
                ErrorCode::DigestMismatch,
                "Modified node bytes",
            )?;
        } else {
            require(
                node["digest"] == node["artifact"]["digest"],
                ErrorCode::DigestMismatch,
                "Artifact digest mismatch",
            )?;
        }
        if has(node, "source_span") {
            let span = &node["source_span"];
            let length = if has(node, "text") {
                text(&node["text"]).len() as u64
            } else {
                num(&node["artifact"]["byte_length"])
            };
            require(
                has(node, "source")
                    && num(&span["end"]) >= num(&span["start"])
                    && num(&span["end"]) - num(&span["start"]) == length,
                ErrorCode::InvalidOffset,
                "Source excerpt length mismatch",
            )?;
        }
        let kind = text(&node["derivation"]);
        let exact = matches!(kind, "selection" | "normalization" | "concatenation");
        match kind {
            "root" => require(
                list(&node["parents"]).is_empty() && list(&node["mappings"]).is_empty(),
                ErrorCode::InvalidMapping,
                "Root has no derivation",
            )?,
            "opaque" => require(
                node["coverage"] == "opaque" && list(&node["mappings"]).is_empty(),
                ErrorCode::InvalidMapping,
                "Opaque coverage required",
            )?,
            _ => require(
                !list(&node["parents"]).is_empty(),
                ErrorCode::MissingReference,
                "Derivation needs genealogy",
            )?,
        }
        if matches!(kind, "summary" | "model_output" | "tool_output") {
            require(
                has(node, "receipt")
                    && node["coverage"] != "complete"
                    && list(&node["mappings"]).is_empty(),
                ErrorCode::InvalidMapping,
                "Genealogy is not exact or semantic support",
            )?;
        }
        if !exact {
            require(
                list(&node["mappings"]).is_empty(),
                ErrorCode::InvalidMapping,
                "Only verified transformations carry mappings",
            )?;
            if matches!(kind, "redaction" | "rendering") {
                require(
                    node["coverage"] != "complete",
                    ErrorCode::InvalidMapping,
                    "Unverified transformation",
                )?;
            }
        }
        let mut end = 0;
        for mapping in list(&node["mappings"]) {
            require(
                list(&node["parents"]).contains(&mapping["parent"]),
                ErrorCode::MissingReference,
                "Mapping needs declared parent",
            )?;
            let parent = nodes[text(&mapping["parent"])];
            require(
                has(parent, "text") && has(node, "text"),
                ErrorCode::InvalidMapping,
                "Text mapping required",
            )?;
            let mut left = span_bytes(text(&parent["text"]), &mapping["input"])?.to_vec();
            let right = span_bytes(text(&node["text"]), &mapping["output"])?;
            if kind == "normalization" {
                left = std::str::from_utf8(&left)
                    .expect("validated UTF-8")
                    .replace("\r\n", "\n")
                    .nfc()
                    .collect::<String>()
                    .into_bytes();
            }
            require(
                left == right && num(&mapping["output"]["start"]) == end,
                ErrorCode::InvalidMapping,
                "Transformation bytes or ordered coverage mismatch",
            )?;
            end = num(&mapping["output"]["end"]);
        }
        if exact {
            require(
                has(node, "text")
                    && end == text(&node["text"]).len() as u64
                    && node["coverage"] == "complete",
                ErrorCode::InvalidMapping,
                "Exact derivation covers every output byte",
            )?;
        }
        nodes.insert(text(&node["id"]), node);
    }
    let mut productions = HashMap::new();
    for receipt in list(&value["productions"]) {
        require(
            !productions.contains_key(text(&receipt["id"])),
            ErrorCode::InvalidSchema,
            "Duplicate production receipt",
        )?;
        require(
            nodes.contains_key(text(&receipt["output"]))
                && nodes.contains_key(text(&receipt["response"]))
                && list(&receipt["inputs"])
                    .iter()
                    .all(|p| nodes.contains_key(text(p))),
            ErrorCode::MissingReference,
            "Unknown production reference",
        )?;
        let output = nodes[text(&receipt["output"])];
        require(
            output["receipt"] == receipt["id"]
                && output["parents"] == receipt["inputs"]
                && receipt["kind"]
                    == if output["derivation"] == "tool_output" {
                        "tool"
                    } else {
                        "model"
                    },
            ErrorCode::InvalidMapping,
            "Producer and declared genealogy differ",
        )?;
        productions.insert(text(&receipt["id"]), receipt);
    }
    for node in nodes.values() {
        if matches!(
            text(&node["derivation"]),
            "summary" | "model_output" | "tool_output"
        ) {
            require(
                productions
                    .get(text(&node["receipt"]))
                    .is_some_and(|r| r["output"] == node["id"]),
                ErrorCode::MissingReference,
                "Missing typed production receipt",
            )?;
        }
    }
    let request = &value["request"];
    let mut slots = HashSet::new();
    for part in list(&request["parts"]) {
        require(
            slots.insert(text(&part["slot"])),
            ErrorCode::InvalidSchema,
            "Duplicate request slot",
        )?;
        require(
            has(part, "text") != has(part, "artifact"),
            ErrorCode::InvalidSchema,
            "Exactly one request content representation",
        )?;
        let mut end = 0;
        for segment in list(&part["segments"]) {
            let node = nodes.get(text(&segment["node"])).ok_or_else(|| Failure {
                code: ErrorCode::MissingReference,
                message: "Unknown request reference".into(),
            })?;
            require(
                num(&segment["span"]["start"]) == end,
                ErrorCode::UncoveredBytes,
                "Coverage must be contiguous and ordered",
            )?;
            if has(part, "text") && has(node, "text") {
                require(
                    span_bytes(text(&part["text"]), &segment["span"])?
                        == span_bytes(text(&node["text"]), &segment["node_span"])?,
                    ErrorCode::ModifiedBytes,
                    "Request bytes differ from reference",
                )?;
            } else {
                require(
                    has(part, "artifact")
                        && has(node, "artifact")
                        && part["artifact"] == node["artifact"],
                    ErrorCode::ModifiedBytes,
                    "Artifact descriptor differs",
                )?;
                for span in [&segment["span"], &segment["node_span"]] {
                    require(
                        num(&span["start"]) == 0 && span["end"] == node["artifact"]["byte_length"],
                        ErrorCode::InvalidOffset,
                        "Artifact reference covers entire artifact",
                    )?;
                }
            }
            end = num(&segment["span"]["end"]);
        }
        let length = if has(part, "text") {
            text(&part["text"]).len() as u64
        } else {
            num(&part["artifact"]["byte_length"])
        };
        require(
            end == length,
            ErrorCode::UncoveredBytes,
            "Every host-controlled byte needs lineage",
        )?;
    }
    let mut captures = HashSet::new();
    for receipt in list(&value["captures"]) {
        require(
            nodes.contains_key(text(&receipt["node"])),
            ErrorCode::MissingReference,
            "Unknown capture reference",
        )?;
        require(
            captures.insert(text(&receipt["id"])),
            ErrorCode::InvalidSchema,
            "Duplicate capture receipt",
        )?;
    }
    let mut states = HashMap::new();
    let mut last_receipts = HashMap::new();
    let mut keys = Idempotency::default();
    for receipt in list(&value["deliveries"]) {
        require(
            receipt["request"] == request["id"]
                && text(&receipt["assembly_digest"]) == assembly_digest(request),
            ErrorCode::ModifiedBytes,
            "Receipt must bind exact request assembly",
        )?;
        let key = text(&receipt["idempotency_key"]);
        keys.register(
            key,
            &format!(
                "{}:{}",
                text(&receipt["assembly_digest"]),
                text(&receipt["attempt"])
            ),
        )?;
        let old = states.get(key).copied();
        let state = text(&receipt["state"]);
        let allowed = match old {
            None => state == "prepared",
            Some("prepared") => matches!(
                state,
                "prepared" | "transport_started" | "failed" | "cancelled_uncertain"
            ),
            Some("transport_started") => matches!(
                state,
                "transport_started" | "provider_confirmed" | "failed" | "cancelled_uncertain"
            ),
            Some(previous) => state == previous,
        };
        require(
            allowed,
            ErrorCode::InvalidDelivery,
            "Invalid delivery transition",
        )?;
        require(
            old != Some(state) || last_receipts.get(key).copied() == Some(receipt),
            ErrorCode::InvalidDelivery,
            "Receipt replay must preserve every field",
        )?;
        require(
            has(receipt, "provider_receipt") == (state == "provider_confirmed"),
            ErrorCode::InvalidDelivery,
            "Provider confirmation needs observable receipt",
        )?;
        states.insert(key, state);
        last_receipts.insert(key, receipt);
    }
    let mut handles = HashMap::new();
    for handle in list(&value["handles"]) {
        require(
            !handles.contains_key(text(&handle["id"])),
            ErrorCode::InvalidSchema,
            "Duplicate handle",
        )?;
        validate_handle(handle)?;
        require(
            nodes.values().any(|n| {
                n["source"]["id"] == handle["source"]["id"]
                    && n["source"]["revision"] == handle["source"]["revision"]
            }),
            ErrorCode::MissingReference,
            "Handle source revision must be retained",
        )?;
        handles.insert(text(&handle["id"]), handle);
    }
    for annotation in list(&value["annotations"]) {
        require(
            nodes.contains_key(text(&annotation["evidence"]))
                && list(&annotation["parents"])
                    .iter()
                    .all(|p| nodes.contains_key(text(p))),
            ErrorCode::MissingReference,
            "Annotation evidence is immutable context",
        )?;
        if has(annotation, "location") {
            require(
                handles.contains_key(text(&annotation["location"])),
                ErrorCode::MissingReference,
                "Unknown typed location handle",
            )?;
            let bound = handles[text(&annotation["location"])];
            require(
                ["run", "stop", "frame", "invocation"]
                    .iter()
                    .all(|k| !has(bound, k) || annotation.get(k) == bound.get(k)),
                ErrorCode::StaleHandle,
                "Annotation and location execution scopes differ",
            )?;
        }
        if matches!(
            text(&annotation["kind"]),
            "paused" | "values" | "expression"
        ) {
            require(
                ["run", "stop", "frame", "invocation", "location"]
                    .iter()
                    .all(|k| has(annotation, k)),
                ErrorCode::InvalidSchema,
                "Debugger annotation needs captured execution scope",
            )?;
        }
        if annotation["kind"] == "profile" {
            require(
                has(annotation, "metric") && has(annotation, "run"),
                ErrorCode::InvalidSchema,
                "Profile needs measurement semantics",
            )?;
        }
    }
    Ok(serde_json::from_value(value.clone()).expect("validated bundle matches generated types"))
}
</storycode:copy>

### Assemble `crates/hick-context-protocol/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-context-protocol/src/lib.rs"><storycode:paste select="#f-1c697d40a8e6-types"/><storycode:paste select="#f-1c697d40a8e6-use_declaration"/><storycode:paste select="#f-1c697d40a8e6-use_declaration-2"/><storycode:paste select="#f-1c697d40a8e6-use_declaration-3"/><storycode:paste select="#f-1c697d40a8e6-use_declaration-4"/><storycode:paste select="#f-1c697d40a8e6-use_declaration-5"/><storycode:paste select="#f-1c697d40a8e6-MAX_FRAME"/><storycode:paste select="#f-1c697d40a8e6-MAX_PAGE"/><storycode:paste select="#f-1c697d40a8e6-SCHEMA"/><storycode:paste select="#f-1c697d40a8e6-Failure"/><storycode:paste select="#f-1c697d40a8e6-impl-std-fmt-Display-for-Failure"/><storycode:paste select="#f-1c697d40a8e6-impl-std-error-Error-for-Failure"/><storycode:paste select="#f-1c697d40a8e6-Result"/><storycode:paste select="#f-1c697d40a8e6-require"/><storycode:paste select="#f-1c697d40a8e6-list"/><storycode:paste select="#f-1c697d40a8e6-has"/><storycode:paste select="#f-1c697d40a8e6-num"/><storycode:paste select="#f-1c697d40a8e6-text"/><storycode:paste select="#f-1c697d40a8e6-validate"/><storycode:paste select="#f-1c697d40a8e6-digest"/><storycode:paste select="#f-1c697d40a8e6-assembly_digest"/><storycode:paste select="#f-1c697d40a8e6-span_bytes"/><storycode:paste select="#f-1c697d40a8e6-utf16_to_utf8"/><storycode:paste select="#f-1c697d40a8e6-negotiate"/><storycode:paste select="#f-1c697d40a8e6-validate_frame"/><storycode:paste select="#f-1c697d40a8e6-validate_page"/><storycode:paste select="#f-1c697d40a8e6-validate_handle"/><storycode:paste select="#f-1c697d40a8e6-resolve_handle"/><storycode:paste select="#f-1c697d40a8e6-Idempotency"/><storycode:paste select="#f-1c697d40a8e6-impl-Idempotency"/><storycode:paste select="#f-1c697d40a8e6-validate_bundle"/></storycode:file>

<a id="output-540657819071"></a>
## `crates/hick-context-protocol/src/types.rs`

Validate the host/worker wire contract. Selection and refresh policy remain in the Python worker rather than this validation layer.

Related contracts: [Context protocol validates declared byte lineage before integration](contract.md).

<a id="f-540657819071-ErrorCode"></a>
### `ErrorCode`

Generated by scripts/generate-context-protocol.py; do not edit.

<storycode:copy id="f-540657819071-ErrorCode">
// Generated by scripts/generate-context-protocol.py; do not edit.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "invalid_schema")]
    InvalidSchema,
    #[serde(rename = "version_skew")]
    VersionSkew,
    #[serde(rename = "capability_missing")]
    CapabilityMissing,
    #[serde(rename = "frame_limit")]
    FrameLimit,
    #[serde(rename = "missing_reference")]
    MissingReference,
    #[serde(rename = "digest_mismatch")]
    DigestMismatch,
    #[serde(rename = "invalid_mapping")]
    InvalidMapping,
    #[serde(rename = "uncovered_bytes")]
    UncoveredBytes,
    #[serde(rename = "modified_bytes")]
    ModifiedBytes,
    #[serde(rename = "invalid_offset")]
    InvalidOffset,
    #[serde(rename = "cursor_expired")]
    CursorExpired,
    #[serde(rename = "invalid_page")]
    InvalidPage,
    #[serde(rename = "idempotency_conflict")]
    IdempotencyConflict,
    #[serde(rename = "invalid_delivery")]
    InvalidDelivery,
    #[serde(rename = "stale_handle")]
    StaleHandle,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "mismatched_response")]
    MismatchedResponse,
}
</storycode:copy>
<a id="f-540657819071-Operation"></a>
### `Operation`

<storycode:copy id="f-540657819071-Operation">
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    #[serde(rename = "context.capture")]
    ContextCapture,
    #[serde(rename = "context.derive")]
    ContextDerive,
    #[serde(rename = "context.changes")]
    ContextChanges,
    #[serde(rename = "context.compare")]
    ContextCompare,
    #[serde(rename = "context.resolve")]
    ContextResolve,
    #[serde(rename = "inference")]
    Inference,
}
</storycode:copy>
<a id="f-540657819071-DerivationKind"></a>
### `DerivationKind`

<storycode:copy id="f-540657819071-DerivationKind">
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationKind {
    #[serde(rename = "root")]
    Root,
    #[serde(rename = "selection")]
    Selection,
    #[serde(rename = "normalization")]
    Normalization,
    #[serde(rename = "concatenation")]
    Concatenation,
    #[serde(rename = "redaction")]
    Redaction,
    #[serde(rename = "rendering")]
    Rendering,
    #[serde(rename = "summary")]
    Summary,
    #[serde(rename = "model_output")]
    ModelOutput,
    #[serde(rename = "tool_output")]
    ToolOutput,
    #[serde(rename = "opaque")]
    Opaque,
}
</storycode:copy>
<a id="f-540657819071-DeliveryState"></a>
### `DeliveryState`

<storycode:copy id="f-540657819071-DeliveryState">
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryState {
    #[serde(rename = "prepared")]
    Prepared,
    #[serde(rename = "transport_started")]
    TransportStarted,
    #[serde(rename = "provider_confirmed")]
    ProviderConfirmed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled_uncertain")]
    CancelledUncertain,
}
</storycode:copy>
<a id="f-540657819071-HandleKind"></a>
### `HandleKind`

<storycode:copy id="f-540657819071-HandleKind">
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandleKind {
    #[serde(rename = "location")]
    Location,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "frame")]
    Frame,
    #[serde(rename = "invocation")]
    Invocation,
}
</storycode:copy>
<a id="f-540657819071-AnnotationKind"></a>
### `AnnotationKind`

<storycode:copy id="f-540657819071-AnnotationKind">
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationKind {
    #[serde(rename = "log")]
    Log,
    #[serde(rename = "stdout")]
    Stdout,
    #[serde(rename = "exception")]
    Exception,
    #[serde(rename = "breakpoint_requested")]
    BreakpointRequested,
    #[serde(rename = "breakpoint_bound")]
    BreakpointBound,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "values")]
    Values,
    #[serde(rename = "expression")]
    Expression,
    #[serde(rename = "profile")]
    Profile,
    #[serde(rename = "external_review")]
    ExternalReview,
}
</storycode:copy>
<a id="f-540657819071-Precision"></a>
### `Precision`

<storycode:copy id="f-540657819071-Precision">
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Precision {
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "span")]
    Span,
    #[serde(rename = "block")]
    Block,
    #[serde(rename = "unknown")]
    Unknown,
}
</storycode:copy>
<a id="f-540657819071-ComparisonState"></a>
### `ComparisonState`

<storycode:copy id="f-540657819071-ComparisonState">
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonState {
    #[serde(rename = "unchanged")]
    Unchanged,
    #[serde(rename = "moved")]
    Moved,
    #[serde(rename = "changed")]
    Changed,
    #[serde(rename = "ambiguous")]
    Ambiguous,
    #[serde(rename = "unavailable")]
    Unavailable,
    #[serde(rename = "cannot_check")]
    CannotCheck,
    #[serde(rename = "invalidated")]
    Invalidated,
}
</storycode:copy>
<a id="f-540657819071-Coverage"></a>
### `Coverage`

<storycode:copy id="f-540657819071-Coverage">
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Coverage {
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "partial")]
    Partial,
    #[serde(rename = "opaque")]
    Opaque,
}
</storycode:copy>
<a id="f-540657819071-Span"></a>
### `Span`

<storycode:copy id="f-540657819071-Span">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub start: u64,
    pub end: u64,
    pub unit: String,
}
</storycode:copy>
<a id="f-540657819071-Source"></a>
### `Source`

<storycode:copy id="f-540657819071-Source">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub revision: String,
    pub locator: String,
    pub kind: String,
}
</storycode:copy>
<a id="f-540657819071-Artifact"></a>
### `Artifact`

<storycode:copy id="f-540657819071-Artifact">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub digest: String,
    pub media_type: String,
    pub byte_length: u64,
    pub locator: String,
}
</storycode:copy>
<a id="f-540657819071-Mapping"></a>
### `Mapping`

<storycode:copy id="f-540657819071-Mapping">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub parent: String,
    pub input: Span,
    pub output: Span,
}
</storycode:copy>
<a id="f-540657819071-Node"></a>
### `Node`

<storycode:copy id="f-540657819071-Node">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<Artifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_span: Option<Span>,
    pub parents: Vec<String>,
    pub derivation: DerivationKind,
    pub coverage: Coverage,
    pub mappings: Vec<Mapping>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<String>,
}
</storycode:copy>
<a id="f-540657819071-Segment"></a>
### `Segment`

<storycode:copy id="f-540657819071-Segment">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    pub span: Span,
    pub node: String,
    pub node_span: Span,
}
</storycode:copy>
<a id="f-540657819071-RequestPart"></a>
### `RequestPart`

<storycode:copy id="f-540657819071-RequestPart">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestPart {
    pub slot: String,
    pub role: String,
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<Artifact>,
    pub segments: Vec<Segment>,
}
</storycode:copy>
<a id="f-540657819071-Request"></a>
### `Request`

<storycode:copy id="f-540657819071-Request">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub id: String,
    pub branch: String,
    pub policy: String,
    pub policy_version: String,
    pub observed_cursor: String,
    pub parts: Vec<RequestPart>,
    pub provider_coverage: String,
}
</storycode:copy>
<a id="f-540657819071-Hello"></a>
### `Hello`

<storycode:copy id="f-540657819071-Hello">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hello {
    pub versions: Vec<u64>,
    pub capabilities: Vec<Operation>,
    pub transport: String,
    pub max_frame_bytes: u64,
    pub max_page_items: u64,
}
</storycode:copy>
<a id="f-540657819071-Negotiated"></a>
### `Negotiated`

<storycode:copy id="f-540657819071-Negotiated">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Negotiated {
    pub version: u64,
    pub capabilities: Vec<Operation>,
    pub transport: String,
    pub max_frame_bytes: u64,
    pub max_page_items: u64,
}
</storycode:copy>
<a id="f-540657819071-CaptureReceipt"></a>
### `CaptureReceipt`

<storycode:copy id="f-540657819071-CaptureReceipt">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureReceipt {
    pub id: String,
    pub node: String,
    pub captured_at: String,
    pub state: String,
}
</storycode:copy>
<a id="f-540657819071-DeliveryReceipt"></a>
### `DeliveryReceipt`

<storycode:copy id="f-540657819071-DeliveryReceipt">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliveryReceipt {
    pub request: String,
    pub attempt: String,
    pub idempotency_key: String,
    pub assembly_digest: String,
    pub state: DeliveryState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_receipt: Option<String>,
}
</storycode:copy>
<a id="f-540657819071-Cursor"></a>
### `Cursor`

<storycode:copy id="f-540657819071-Cursor">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cursor {
    pub epoch: String,
    pub branch: String,
    pub position: u64,
}
</storycode:copy>
<a id="f-540657819071-Change"></a>
### `Change`

<storycode:copy id="f-540657819071-Change">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub sequence: u64,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_revision: Option<String>,
    pub state: ComparisonState,
}
</storycode:copy>
<a id="f-540657819071-ChangePage"></a>
### `ChangePage`

<storycode:copy id="f-540657819071-ChangePage">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangePage {
    pub from_cursor: Cursor,
    pub next_cursor: Cursor,
    pub events: Vec<Change>,
    pub has_more: bool,
    pub limit: u64,
}
</storycode:copy>
<a id="f-540657819071-Handle"></a>
### `Handle`

<storycode:copy id="f-540657819071-Handle">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Handle {
    pub id: String,
    pub kind: HandleKind,
    pub source: Source,
    pub span: Span,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocation: Option<String>,
}
</storycode:copy>
<a id="f-540657819071-Metric"></a>
### `Metric`

<storycode:copy id="f-540657819071-Metric">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub clock: String,
    pub semantics: String,
    pub aggregation: String,
    pub interval_start: String,
    pub interval_end: String,
}
</storycode:copy>
<a id="f-540657819071-Annotation"></a>
### `Annotation`

<storycode:copy id="f-540657819071-Annotation">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    pub id: String,
    pub kind: AnnotationKind,
    pub provider: String,
    pub provider_version: String,
    pub evidence: String,
    pub source_precision: Precision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<Metric>,
    pub coverage: Coverage,
    pub parents: Vec<String>,
    pub payload_schema: String,
    pub payload: serde_json::Value,
}
</storycode:copy>
<a id="f-540657819071-Bundle"></a>
### `Bundle`

<storycode:copy id="f-540657819071-Bundle">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub version: u64,
    pub nodes: Vec<Node>,
    pub request: Request,
    pub captures: Vec<CaptureReceipt>,
    pub deliveries: Vec<DeliveryReceipt>,
    pub handles: Vec<Handle>,
    pub annotations: Vec<Annotation>,
    pub productions: Vec<ProductionReceipt>,
}
</storycode:copy>
<a id="f-540657819071-CaptureRequest"></a>
### `CaptureRequest`

<storycode:copy id="f-540657819071-CaptureRequest">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureRequest {
    pub idempotency_key: String,
    pub source: Source,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    pub max_bytes: u64,
}
</storycode:copy>
<a id="f-540657819071-DeriveRequest"></a>
### `DeriveRequest`

<storycode:copy id="f-540657819071-DeriveRequest">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeriveRequest {
    pub idempotency_key: String,
    pub node: Node,
}
</storycode:copy>
<a id="f-540657819071-ChangesRequest"></a>
### `ChangesRequest`

<storycode:copy id="f-540657819071-ChangesRequest">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangesRequest {
    pub cursor: Cursor,
    pub references: Vec<String>,
    pub limit: u64,
}
</storycode:copy>
<a id="f-540657819071-CompareRequest"></a>
### `CompareRequest`

<storycode:copy id="f-540657819071-CompareRequest">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompareRequest {
    pub old: String,
    pub current: String,
}
</storycode:copy>
<a id="f-540657819071-Comparison"></a>
### `Comparison`

<storycode:copy id="f-540657819071-Comparison">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Comparison {
    pub old: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    pub state: ComparisonState,
    pub mappings: Vec<Mapping>,
}
</storycode:copy>
<a id="f-540657819071-ResolveRequest"></a>
### `ResolveRequest`

<storycode:copy id="f-540657819071-ResolveRequest">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolveRequest {
    pub handle: String,
    pub expected_source: Source,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocation: Option<String>,
}
</storycode:copy>
<a id="f-540657819071-ProtocolError"></a>
### `ProtocolError`

<storycode:copy id="f-540657819071-ProtocolError">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub retryable: bool,
}
</storycode:copy>
<a id="f-540657819071-ProductionReceipt"></a>
### `ProductionReceipt`

<storycode:copy id="f-540657819071-ProductionReceipt">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionReceipt {
    pub id: String,
    pub kind: String,
    pub request: String,
    pub response: String,
    pub inputs: Vec<String>,
    pub output: String,
}
</storycode:copy>
<a id="f-540657819071-deserializes"></a>
### `deserializes`

<storycode:copy id="f-540657819071-deserializes">
pub(crate) fn deserializes(name: &str, value: &serde_json::Value) -> bool {
    match name {
        "ErrorCode" => serde_json::from_value::<ErrorCode>(value.clone()).is_ok(),
        "Operation" => serde_json::from_value::<Operation>(value.clone()).is_ok(),
        "DerivationKind" => serde_json::from_value::<DerivationKind>(value.clone()).is_ok(),
        "DeliveryState" => serde_json::from_value::<DeliveryState>(value.clone()).is_ok(),
        "HandleKind" => serde_json::from_value::<HandleKind>(value.clone()).is_ok(),
        "AnnotationKind" => serde_json::from_value::<AnnotationKind>(value.clone()).is_ok(),
        "Precision" => serde_json::from_value::<Precision>(value.clone()).is_ok(),
        "ComparisonState" => serde_json::from_value::<ComparisonState>(value.clone()).is_ok(),
        "Coverage" => serde_json::from_value::<Coverage>(value.clone()).is_ok(),
        "Span" => serde_json::from_value::<Span>(value.clone()).is_ok(),
        "Source" => serde_json::from_value::<Source>(value.clone()).is_ok(),
        "Artifact" => serde_json::from_value::<Artifact>(value.clone()).is_ok(),
        "Mapping" => serde_json::from_value::<Mapping>(value.clone()).is_ok(),
        "Node" => serde_json::from_value::<Node>(value.clone()).is_ok(),
        "Segment" => serde_json::from_value::<Segment>(value.clone()).is_ok(),
        "RequestPart" => serde_json::from_value::<RequestPart>(value.clone()).is_ok(),
        "Request" => serde_json::from_value::<Request>(value.clone()).is_ok(),
        "Hello" => serde_json::from_value::<Hello>(value.clone()).is_ok(),
        "Negotiated" => serde_json::from_value::<Negotiated>(value.clone()).is_ok(),
        "CaptureReceipt" => serde_json::from_value::<CaptureReceipt>(value.clone()).is_ok(),
        "DeliveryReceipt" => serde_json::from_value::<DeliveryReceipt>(value.clone()).is_ok(),
        "Cursor" => serde_json::from_value::<Cursor>(value.clone()).is_ok(),
        "Change" => serde_json::from_value::<Change>(value.clone()).is_ok(),
        "ChangePage" => serde_json::from_value::<ChangePage>(value.clone()).is_ok(),
        "Handle" => serde_json::from_value::<Handle>(value.clone()).is_ok(),
        "Metric" => serde_json::from_value::<Metric>(value.clone()).is_ok(),
        "Annotation" => serde_json::from_value::<Annotation>(value.clone()).is_ok(),
        "Bundle" => serde_json::from_value::<Bundle>(value.clone()).is_ok(),
        "CaptureRequest" => serde_json::from_value::<CaptureRequest>(value.clone()).is_ok(),
        "DeriveRequest" => serde_json::from_value::<DeriveRequest>(value.clone()).is_ok(),
        "ChangesRequest" => serde_json::from_value::<ChangesRequest>(value.clone()).is_ok(),
        "CompareRequest" => serde_json::from_value::<CompareRequest>(value.clone()).is_ok(),
        "Comparison" => serde_json::from_value::<Comparison>(value.clone()).is_ok(),
        "ResolveRequest" => serde_json::from_value::<ResolveRequest>(value.clone()).is_ok(),
        "ProtocolError" => serde_json::from_value::<ProtocolError>(value.clone()).is_ok(),
        "ProductionReceipt" => serde_json::from_value::<ProductionReceipt>(value.clone()).is_ok(),
        _ => false,
    }
}
</storycode:copy>

### Assemble `crates/hick-context-protocol/src/types.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-context-protocol/src/types.rs"><storycode:paste select="#f-540657819071-ErrorCode"/><storycode:paste select="#f-540657819071-Operation"/><storycode:paste select="#f-540657819071-DerivationKind"/><storycode:paste select="#f-540657819071-DeliveryState"/><storycode:paste select="#f-540657819071-HandleKind"/><storycode:paste select="#f-540657819071-AnnotationKind"/><storycode:paste select="#f-540657819071-Precision"/><storycode:paste select="#f-540657819071-ComparisonState"/><storycode:paste select="#f-540657819071-Coverage"/><storycode:paste select="#f-540657819071-Span"/><storycode:paste select="#f-540657819071-Source"/><storycode:paste select="#f-540657819071-Artifact"/><storycode:paste select="#f-540657819071-Mapping"/><storycode:paste select="#f-540657819071-Node"/><storycode:paste select="#f-540657819071-Segment"/><storycode:paste select="#f-540657819071-RequestPart"/><storycode:paste select="#f-540657819071-Request"/><storycode:paste select="#f-540657819071-Hello"/><storycode:paste select="#f-540657819071-Negotiated"/><storycode:paste select="#f-540657819071-CaptureReceipt"/><storycode:paste select="#f-540657819071-DeliveryReceipt"/><storycode:paste select="#f-540657819071-Cursor"/><storycode:paste select="#f-540657819071-Change"/><storycode:paste select="#f-540657819071-ChangePage"/><storycode:paste select="#f-540657819071-Handle"/><storycode:paste select="#f-540657819071-Metric"/><storycode:paste select="#f-540657819071-Annotation"/><storycode:paste select="#f-540657819071-Bundle"/><storycode:paste select="#f-540657819071-CaptureRequest"/><storycode:paste select="#f-540657819071-DeriveRequest"/><storycode:paste select="#f-540657819071-ChangesRequest"/><storycode:paste select="#f-540657819071-CompareRequest"/><storycode:paste select="#f-540657819071-Comparison"/><storycode:paste select="#f-540657819071-ResolveRequest"/><storycode:paste select="#f-540657819071-ProtocolError"/><storycode:paste select="#f-540657819071-ProductionReceipt"/><storycode:paste select="#f-540657819071-deserializes"/></storycode:file>

<a id="output-be4ea13c15f0"></a>
## `crates/hick-context-protocol/tests/conformance.rs`

Validate the host/worker wire contract. Selection and refresh policy remain in the Python worker rather than this validation layer.

Related contracts: [Context protocol validates declared byte lineage before integration](contract.md).

<a id="f-be4ea13c15f0-run"></a>
### `run`

<storycode:copy id="f-be4ea13c15f0-run">
use hick_context_protocol::*;
use serde_json::{Value, json};

fn run(case: &Value) -> Result<Value> {
    let p = &case["payload"];
    Ok(match case["operation"].as_str().unwrap() {
        "bundle" => {
            let bundle = validate_bundle(p)?;
            let serialized = serde_json::to_value(bundle).unwrap();
            assert_eq!(serialized, *p, "generated wire types must round-trip");
            serialized
        }
        "negotiate" => serde_json::to_value(negotiate(&p[0], &p[1])?).unwrap(),
        "offset" => json!(utf16_to_utf8(
            p["text"].as_str().unwrap(),
            p["offset"].as_u64().unwrap()
        )?),
        "page" => serde_json::to_value(validate_page(
            &p["page"],
            p["epoch"].as_str().unwrap(),
            p["branch"].as_str().unwrap(),
        )?)
        .unwrap(),
        "frame" => validate_frame(
            p["raw"].as_str().unwrap().as_bytes(),
            p["limit"].as_u64().unwrap() as usize,
            if p["expected_id"].is_null() {
                None
            } else {
                Some(&p["expected_id"])
            },
        )?,
        "resolve" => serde_json::to_value(resolve_handle(&p["handle"], &p["request"])?).unwrap(),
        "replay" => {
            let mut keys = Idempotency::default();
            let mut results = Vec::new();
            for pair in p.as_array().unwrap() {
                results.push(keys.register(pair[0].as_str().unwrap(), pair[1].as_str().unwrap())?);
            }
            json!(results)
        }
        op => panic!("Unknown fixture operation {op}"),
    })
}
</storycode:copy>
<a id="f-be4ea13c15f0-shared_conformance"></a>
### `shared_conformance`

<storycode:copy id="f-be4ea13c15f0-shared_conformance">
// Guarantee: docs/guarantees/agent/context-protocol-validates-lineage.md
#[test]
fn shared_conformance() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../../../protocols/context-v1/fixtures.json")).unwrap();
    for case in cases {
        let result = run(&case);
        if case["expected"] == "ok" {
            let result = result.unwrap_or_else(|e| panic!("{}: {e}", case["name"]));
            if let Some(expected) = case.get("result") {
                assert_eq!(&result, expected, "{}", case["name"]);
            }
        } else {
            let error = result.expect_err(&format!("{} should fail", case["name"]));
            assert_eq!(
                serde_json::to_value(error.code).unwrap(),
                case["expected"],
                "{}: {error}",
                case["name"]
            );
        }
    }
}
</storycode:copy>

### Assemble `crates/hick-context-protocol/tests/conformance.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-context-protocol/tests/conformance.rs"><storycode:paste select="#f-be4ea13c15f0-run"/><storycode:paste select="#f-be4ea13c15f0-shared_conformance"/></storycode:file>

## Implementation review

### Implementation

- [apps/agent/src/hickory_agent_worker/context_protocol.py](contract.md#output-4922a914e9e6)
- [apps/agent/src/hickory_agent_worker/context_types.py](contract.md#output-62037abb921c)
- [crates/hick-context-protocol/src/lib.rs](contract.md#output-1c697d40a8e6)
- [crates/hick-context-protocol/src/types.rs](contract.md#output-540657819071)
- [scripts/check-codegen.sh](../../../mechanisms/scripts/check-codegen.sh.md#output-44e824684c22)

### Tests and fixtures

- [apps/agent/tests/test_context_protocol.py](contract.md#output-48176c4779fa)
- [crates/hick-context-protocol/tests/conformance.rs](contract.md#output-be4ea13c15f0)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
