<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/src/hickory_agent_worker/context.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Python model requests carry validated context manifests](../../../../../contract.md)
- [Local refresh experiments preserve historical request evidence](../../../../../../refresh-experiments-preserve-evidence/contract.md)

Captured SHA-256: `ade95c245825dd2b65a9210063f4c982e30ffb3867ad851f92fa92a39429abd2`.

<book:entry path="apps/agent/src/hickory_agent_worker/context.py">
"""Injectable lineage and refresh policy; no workspace or credential access.

Append updates are a baseline policy. A boundary drains only the updates already
queued; it never waits for a source to stop changing or edits an in-flight request.
"""
from copy import deepcopy
import json
from uuid import uuid4
from typing import Protocol
from .context_protocol import digest, validate_bundle


def identity():
    return str(uuid4())


def span(text):
    return {'start': 0, 'end': len(text.encode()), 'unit': 'utf8_bytes'}


class ContextClient(Protocol):
    def boundary(self, messages): ...
    def prepare(self, messages): ...
    def register(self, text, parents=()): ...
    def response(self, text, bundle): ...


class AppendUpdates:
    """Replaceable refresh encoding, independent of the immutable ledger."""
    name = 'append-update'
    version = '1'

    def __init__(self):
        self.pending = []

    def boundary(self, messages, register):
        pending, self.pending = self.pending, []
        for text, parents in pending:
            messages.append({'role': 'user', 'content': text})
            register(text, parents)


class MemoryContext:
    def __init__(self, branch='standalone', policy=None):
        self.branch = branch
        self.nodes = {}
        self.policy = policy if policy is not None else AppendUpdates()
        self.cursor = 'memory:0'
        self.external = {}
        self.productions = {}
        self.preferred = {}

    def queue(self, text, parents=()):
        self.policy.pending.append((text, list(parents)))

    @property
    def pending(self):
        return self.policy.pending

    def boundary(self, messages):
        self.policy.boundary(messages, self.register)

    def response(self, text, bundle):
        if text not in self.preferred:
            self.register(text, [s['node'] for p in bundle['request']['parts'] for s in p['segments']])

    def register(self, text, parents=()):
        # Equal bytes need not imply equal source identity. Reuse only this
        # Python construction, never substitute it for a source-backed capture.
        if not parents and text in self.preferred:
            return self.preferred[text]
        key = (text, tuple(parents))
        if key not in self.nodes:
            self.nodes[key] = dict(id=identity(), text=text, digest=digest(text.encode()),
                parents=list(parents), derivation='opaque' if parents else 'root',
                coverage='opaque' if parents else 'complete', mappings=[])
        self.preferred[text] = self.nodes[key]
        return self.nodes[key]

    def prepare(self, messages):
        parts = []
        for index, message in enumerate(deepcopy(messages)):
            node = self.register(message['content'])
            parts.append(dict(slot=str(index), role=message['role'], scope='conversation',
                text=message['content'], segments=[dict(span=span(message['content']),
                    node=node['id'], node_span=span(node['text']))]))
        # The Python append policy selects artifacts returned by explicit host
        # reads. Rust validates/resolves only these selected immutable descriptors.
        for node in self.external.values():
            if 'artifact' not in node:
                continue
            artifact = node['artifact']
            whole = {'start': 0, 'end': artifact['byte_length'], 'unit': 'utf8_bytes'}
            parts.append(dict(slot=str(len(parts)), role='user', scope='captured-ui-image',
                artifact=deepcopy(artifact), segments=[dict(span=whole, node=node['id'], node_span=whole)]))
        nodes = list(self.external.values()) + [n for n in self.nodes.values()
                                                 if n['id'] not in self.external]
        bundle = dict(version=1, nodes=nodes, captures=[], deliveries=[],
            handles=[], annotations=[], productions=list(self.productions.values()), request=dict(id=identity(),
                branch=self.branch, policy=self.policy.name, policy_version=self.policy.version,
                observed_cursor=self.cursor, parts=parts, provider_coverage='host_controlled_only'))
        return validate_bundle(deepcopy(bundle))


class HostedContext(MemoryContext):
    def __init__(self, broker, bootstrap_references=()):
        super().__init__()
        self.broker = broker
        hello = broker.call('context.hello', dict(versions=[1], capabilities=[
            'context.capture', 'context.derive', 'context.changes', 'context.compare'],
            transport='cursor_polling', max_frame_bytes=1048576, max_page_items=256))
        self.position = hello['cursor']
        self.branch = self.position['branch']
        self.registered = set()
        self.references = list(bootstrap_references)
        broker.context = self
        self.import_references(self.references)

    def boundary(self, messages):
        # One bounded poll per model boundary: continuous editing cannot starve
        # inference. Newer changes remain after the returned cursor for next time.
        # Explicit selected updates enter only this subsequent safe boundary.
        for item in self.broker.call('context.updates', {}):
            update = item['update']
            self.import_references(item['references'])
            self.queue('Context update ' + update['id'] + ': ' + json.dumps(update) +
                       '\nPrevious plans based on this source require revalidation.', item['references'])
        references = list(dict.fromkeys(self.references))[:256]
        try:
            page = self.broker.call('context.changes', {'cursor': self.position,
                'references': references, 'limit': 256})
            self.position = page['next_cursor']
            if page['events']:
                self.queue('Context source changes (historical observations remain historical; '
                    'reread before relying on changed inputs): ' + json.dumps(page['events']))
        except RuntimeError:
            recovery = self.broker.call('context.reconcile', {'references': references})
            self.position = recovery['cursor']
            self.queue('Context reconciliation: ' + json.dumps(recovery['currentness']))
        self.cursor = json.dumps(self.position, sort_keys=True)
        super().boundary(messages)

    def import_references(self, references):
        if references:
            result = self.broker.call('context.references', {'references': references})
            self.external.update((n['id'], n) for n in result['nodes'])
            self.references.extend(references)
            self.productions.update((p['id'], p) for p in result.get('productions', []))

    def prepare(self, messages):
        bundle = super().prepare(messages)
        for node in bundle['nodes']:
            if node['id'] not in self.registered and node['id'] not in self.external:
                self.broker.call('context.derive', {'idempotency_key': node['id'], 'node': node})
                self.registered.add(node['id'])
        return bundle
</book:entry>

</book:doc>
