<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/src/hickory_agent_worker/context_protocol.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Context protocol validates declared byte lineage before integration](../../../../../contract.md)

Captured SHA-256: `0d395fe4962de5ea0dcf292372f0d0900052df66ed29466072d49f6542f01653`.

<book:entry path="apps/agent/src/hickory_agent_worker/context_protocol.py">
"""Opt-in context v1 contract. No filesystem, refresh policy or broker side effects."""
from hashlib import sha256
from importlib.resources import files
import json
import unicodedata
from typing import cast

from jsonschema import Draft202012Validator, validators
from .context_types import Bundle, ErrorCode, Hello, Negotiated

SCHEMA = json.loads(files(__package__).joinpath('context_schema.json').read_text())
MAX_FRAME = 1_048_576
MAX_PAGE = 256
WireValidator = validators.extend(Draft202012Validator, type_checker=
    Draft202012Validator.TYPE_CHECKER.redefine(
        "integer", lambda _checker, value: isinstance(value, int) and not isinstance(value, bool)))


class ProtocolFailure(ValueError):
    def __init__(self, code: ErrorCode, message: str):
        self.code = code
        super().__init__(message)


def require(ok, code, message):
    if not ok:
        raise ProtocolFailure(code, message)


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


def digest(data):
    return sha256(data).hexdigest()


def assembly_digest(request):
    return digest(json.dumps(request, ensure_ascii=False, sort_keys=True,
                             separators=(',', ':'), allow_nan=False).encode())


def span_bytes(text, span):
    raw = text.encode()
    start, end = span['start'], span['end']
    require(start <= end <= len(raw), 'invalid_offset', 'Invalid UTF-8 byte range')
    try:
        raw[:start].decode(); raw[start:end].decode()
    except UnicodeDecodeError as exc:
        raise ProtocolFailure('invalid_offset', 'Split UTF-8 code point') from exc
    return raw[start:end]


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


def validate_handle(handle):
    validate(handle, 'Handle')
    require(handle['span']['start'] <= handle['span']['end'], 'invalid_offset', 'Invalid handle span')
    required = {'location': (), 'stop': ('run', 'stop'), 'frame': ('run', 'stop', 'frame'),
                'invocation': ('run', 'invocation')}[handle['kind']]
    require(all(k in handle for k in required), 'invalid_schema', 'Missing handle execution scope')
    return handle


def resolve_handle(handle, request):
    validate_handle(handle); validate(request, 'ResolveRequest')
    require(request['handle'] == handle['id'] and all(
        request['expected_source'][k] == handle['source'][k] for k in ('id', 'revision'))
        and all(request.get(k) == handle.get(k) for k in ('run', 'stop', 'frame', 'invocation')),
        'stale_handle', 'Rust must resolve against current revision and execution scope')
    return handle


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
</book:entry>

</book:doc>
