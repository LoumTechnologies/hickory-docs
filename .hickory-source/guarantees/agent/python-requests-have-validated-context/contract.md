<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Python model requests carry validated context manifests

For: Authors using an agent to change documents while retaining review control.

Given the built-in confined Python agent,
when Python selects messages at a model boundary,
then it registers Python constructions, preserves captured read/editor references,
and submits an ordered manifest covering each message's exact UTF-8 bytes, role,
branch, refresh policy and observed cursor. Rust resolves references against its
immutable store and validates complete segment coverage before constructing the
messages passed to the provider adapter. Rust adds no prompt or refresh policy.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/python-requests-have-validated-context.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-778ee53bd3ed"></a>
## `apps/agent/src/hickory_agent_worker/context.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Python model requests carry validated context manifests](contract.md), [Local refresh experiments preserve historical request evidence](../refresh-experiments-preserve-evidence/contract.md).

<a id="f-778ee53bd3ed-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-778ee53bd3ed-expression_statement">
"""Injectable lineage and refresh policy; no workspace or credential access.

Append updates are a baseline policy. A boundary drains only the updates already
queued; it never waits for a source to stop changing or edits an in-flight request.
"""
</storycode:copy>
<a id="f-778ee53bd3ed-deepcopy"></a>
### `deepcopy`

<storycode:copy id="f-778ee53bd3ed-deepcopy">
from copy import deepcopy
</storycode:copy>
<a id="f-778ee53bd3ed-json"></a>
### `json`

<storycode:copy id="f-778ee53bd3ed-json">
import json
</storycode:copy>
<a id="f-778ee53bd3ed-uuid4"></a>
### `uuid4`

<storycode:copy id="f-778ee53bd3ed-uuid4">
from uuid import uuid4
</storycode:copy>
<a id="f-778ee53bd3ed-Protocol"></a>
### `Protocol`

<storycode:copy id="f-778ee53bd3ed-Protocol">
from typing import Protocol
</storycode:copy>
<a id="f-778ee53bd3ed-digest"></a>
### `digest`

<storycode:copy id="f-778ee53bd3ed-digest">
from .context_protocol import digest, validate_bundle


</storycode:copy>
<a id="f-778ee53bd3ed-identity"></a>
### `identity`

<storycode:copy id="f-778ee53bd3ed-identity">
def identity():
    return str(uuid4())


</storycode:copy>
<a id="f-778ee53bd3ed-span"></a>
### `span`

<storycode:copy id="f-778ee53bd3ed-span">
def span(text):
    return {'start': 0, 'end': len(text.encode()), 'unit': 'utf8_bytes'}


</storycode:copy>
<a id="f-778ee53bd3ed-ContextClient"></a>
### `ContextClient`

<storycode:copy id="f-778ee53bd3ed-ContextClient">
class ContextClient(Protocol):
    def boundary(self, messages): ...
    def prepare(self, messages): ...
    def register(self, text, parents=()): ...
    def response(self, text, bundle): ...


</storycode:copy>
<a id="f-778ee53bd3ed-AppendUpdates"></a>
### `AppendUpdates`

<storycode:copy id="f-778ee53bd3ed-AppendUpdates">
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


</storycode:copy>
<a id="f-778ee53bd3ed-MemoryContext"></a>
### `MemoryContext`

<storycode:copy id="f-778ee53bd3ed-MemoryContext">
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


</storycode:copy>
<a id="f-778ee53bd3ed-HostedContext"></a>
### `HostedContext`

<storycode:copy id="f-778ee53bd3ed-HostedContext">
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
</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/context.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/context.py"><storycode:paste select="#f-778ee53bd3ed-expression_statement"/><storycode:paste select="#f-778ee53bd3ed-deepcopy"/><storycode:paste select="#f-778ee53bd3ed-json"/><storycode:paste select="#f-778ee53bd3ed-uuid4"/><storycode:paste select="#f-778ee53bd3ed-Protocol"/><storycode:paste select="#f-778ee53bd3ed-digest"/><storycode:paste select="#f-778ee53bd3ed-identity"/><storycode:paste select="#f-778ee53bd3ed-span"/><storycode:paste select="#f-778ee53bd3ed-ContextClient"/><storycode:paste select="#f-778ee53bd3ed-AppendUpdates"/><storycode:paste select="#f-778ee53bd3ed-MemoryContext"/><storycode:paste select="#f-778ee53bd3ed-HostedContext"/></storycode:file>

<a id="output-c807e1692e39"></a>
## `apps/agent/tests/test_context_runtime.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Python model requests carry validated context manifests](contract.md), [Local refresh experiments preserve historical request evidence](../refresh-experiments-preserve-evidence/contract.md).

<a id="f-c807e1692e39-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c807e1692e39-expression_statement">
"""Guarantee: docs/guarantees/agent/python-requests-have-validated-context.md."""
</storycode:copy>
<a id="f-c807e1692e39-unittest"></a>
### `unittest`

<storycode:copy id="f-c807e1692e39-unittest">
import unittest
</storycode:copy>
<a id="f-c807e1692e39-deepcopy"></a>
### `deepcopy`

<storycode:copy id="f-c807e1692e39-deepcopy">
from copy import deepcopy
</storycode:copy>
<a id="f-c807e1692e39-MemoryContext"></a>
### `MemoryContext`

<storycode:copy id="f-c807e1692e39-MemoryContext">
from hickory_agent_worker.context import MemoryContext, HostedContext, AppendUpdates
</storycode:copy>
<a id="f-c807e1692e39-validate_bundle"></a>
### `validate_bundle`

<storycode:copy id="f-c807e1692e39-validate_bundle">
from hickory_agent_worker.context_protocol import validate_bundle, ProtocolFailure
</storycode:copy>
<a id="f-c807e1692e39-Agent"></a>
### `Agent`

<storycode:copy id="f-c807e1692e39-Agent">
from hickory_agent_worker.core import Agent, Config


</storycode:copy>
<a id="f-c807e1692e39-ContextRuntimeTests"></a>
### `ContextRuntimeTests`

<storycode:copy id="f-c807e1692e39-ContextRuntimeTests">
class ContextRuntimeTests(unittest.TestCase):
    def test_explicit_host_artifacts_are_selected_as_whole_non_text_parts(self):
        context = MemoryContext('images')
        node = dict(id='captured-image', digest='a' * 64, artifact=dict(digest='a' * 64,
            media_type='image/png', byte_length=70, locator='retained:' + 'a' * 64),
            parents=[], derivation='root', coverage='complete', mappings=[])
        context.external[node['id']] = node
        bundle = context.prepare([{'role': 'user', 'content': 'Inspect this captured image'}])
        part = bundle['request']['parts'][1]
        self.assertEqual(part['role'], 'user')
        self.assertEqual(part['scope'], 'captured-ui-image')
        self.assertNotIn('text', part)
        self.assertEqual(part['artifact'], node['artifact'])
        self.assertEqual(part['segments'][0]['node'], node['id'])
        changed = deepcopy(bundle)
        changed['request']['parts'][1]['artifact']['locator'] = '/etc/passwd'
        with self.assertRaises(ProtocolFailure):
            validate_bundle(changed)

    def test_custom_policy_identity_is_recorded(self):
        class CustomPolicy(AppendUpdates):
            name = 'experimental'
            version = '2'
        request = MemoryContext(policy=CustomPolicy()).prepare([])['request']
        self.assertEqual((request['policy'], request['policy_version']), ('experimental', '2'))

    def test_exact_unicode_roles_and_tamper_rejection(self):
        context = MemoryContext('branch-a')
        bundle = context.prepare([{'role': 'system', 'content': 'Rules\r\n😀'},
                                  {'role': 'user', 'content': 'é'}])
        self.assertEqual(bundle['request']['branch'], 'branch-a')
        modified = deepcopy(bundle)
        modified['request']['parts'][0]['text'] += '!'
        with self.assertRaises(ProtocolFailure):
            validate_bundle(modified)
        modified = deepcopy(bundle)
        modified['request']['parts'][0]['segments'] = []
        with self.assertRaises(ProtocolFailure):
            validate_bundle(modified)

    def test_update_during_inference_waits_for_next_boundary(self):
        context = MemoryContext()
        context.queue('Revision one')
        requests = []
        class Model:
            def complete_request(self, bundle):
                requests.append(deepcopy(bundle))
                if len(requests) == 1:
                    context.queue('Revision two')
                    return '{"python":"42"}'
                return '{"done":"ok"}'
        class Tools:
            def complete(self, summary):
                return {'accepted': True}
        Agent(Model(), Tools(), lambda code: {'result': 42}, context=context).run(Config('Task'))
        first = [p['text'] for p in requests[0]['request']['parts']]
        second = [p['text'] for p in requests[1]['request']['parts']]
        self.assertIn('Revision one', first)
        self.assertNotIn('Revision two', first)
        self.assertIn('Revision two', second)
        self.assertNotEqual(requests[0]['request']['id'], requests[1]['request']['id'])
        self.assertEqual(context.pending, [])

    def test_compaction_retains_declared_ancestry_without_exactness_claim(self):
        context = MemoryContext('child')
        parent = context.register('Original requirement')
        summary = context.register('Short summary', [parent['id']])
        bundle = context.prepare([{'role': 'user', 'content': 'Short summary'}])
        self.assertEqual(summary['coverage'], 'opaque')
        self.assertEqual(bundle['request']['parts'][0]['segments'][0]['node'], summary['id'])
        self.assertIn(parent, bundle['nodes'])

    def test_retry_pins_manifest_and_does_not_consume_late_updates(self):
        context = MemoryContext()
        seen = []
        class Model:
            def complete_request(self, bundle):
                seen.append(deepcopy(bundle))
                if len(seen) == 1:
                    context.queue('Changed during failed request')
                    raise RuntimeError('failed')
                return '{"done":"ok"}'
        class Tools:
            def complete(self, summary):
                return {'accepted': True}
        Agent(Model(), Tools(), None, context=context).run(Config('Task'))
        self.assertEqual(seen[0], seen[1])
        self.assertEqual(context.pending[0][0], 'Changed during failed request')

    def test_continuous_change_stream_gets_one_poll_per_boundary(self):
        class Broker:
            def __init__(self):
                self.polls = 0
            def call(self, method, params):
                cursor = {'epoch': 'epoch', 'branch': 'branch', 'position': self.polls}
                if method == 'context.hello':
                    return {'cursor': cursor}
                if method == 'context.updates':
                    return []
                if method == 'context.changes':
                    self.polls += 1
                    cursor['position'] = self.polls
                    return {'next_cursor': cursor, 'has_more': True,
                            'events': [{'sequence': self.polls, 'state': 'changed'}]}
                raise AssertionError(method)
        broker = Broker()
        context = HostedContext(broker)
        messages = []
        context.boundary(messages)
        self.assertEqual(broker.polls, 1)
        self.assertEqual(len(messages), 1)
        self.assertIn('reread', messages[0]['content'])


</storycode:copy>
<a id="f-c807e1692e39-if_statement"></a>
### `if_statement`

<storycode:copy id="f-c807e1692e39-if_statement">
if __name__ == '__main__':
    unittest.main()
</storycode:copy>

### Assemble `apps/agent/tests/test_context_runtime.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/tests/test_context_runtime.py"><storycode:paste select="#f-c807e1692e39-expression_statement"/><storycode:paste select="#f-c807e1692e39-unittest"/><storycode:paste select="#f-c807e1692e39-deepcopy"/><storycode:paste select="#f-c807e1692e39-MemoryContext"/><storycode:paste select="#f-c807e1692e39-validate_bundle"/><storycode:paste select="#f-c807e1692e39-Agent"/><storycode:paste select="#f-c807e1692e39-ContextRuntimeTests"/><storycode:paste select="#f-c807e1692e39-if_statement"/></storycode:file>

## Test the behavior: `model_response_retains_production_genealogy_after_restart`

This is the maintained test fragment from `crates/hick-context-store/tests/lineage.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-335caf8febec-model_response_retains_production_genealogy_after_restart"></a>
### `model_response_retains_production_genealogy_after_restart`

<storycode:copy id="f-335caf8febec-model_response_retains_production_genealogy_after_restart">
// Guarantee: docs/guarantees/agent/python-requests-have-validated-context.md
#[test]
fn model_response_retains_production_genealogy_after_restart() {
    let (root, mut service) = service();
    let input = saved(&mut service, root.path(), "Requirement 📝", "input");
    assert!(
        service
            .store
            .model_response("bad", vec!["missing".into()], "Invented")
            .is_err()
    );
    let output = service
        .store
        .model_response("request-1", vec![input.id.clone()], "Summary")
        .unwrap();
    assert_eq!(output.derivation, DerivationKind::ModelOutput);
    assert_eq!(output.coverage, Coverage::Partial);
    assert_eq!(output.parents, vec![input.id]);
    let receipt = service.store.productions()[0].clone();
    assert_eq!(receipt.output, output.id);
    assert_eq!(
        service
            .store
            .node(&receipt.response)
            .unwrap()
            .text
            .as_deref(),
        Some("Summary")
    );
    drop(service);
    let reopened = Service::open(root.path(), &root.path().join("store.json"), "child").unwrap();
    assert_eq!(reopened.store.node(&output.id).unwrap(), &output);
    assert_eq!(reopened.store.productions()[0].request, "request-1");
}

</storycode:copy>

## Test the behavior: `cancelling_active_inference_records_uncertain_delivery`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_python_agent.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-08f0cd3133cc-cancelling_active_inference_records_uncertain_delivery"></a>
### `cancelling_active_inference_records_uncertain_delivery`

<storycode:copy id="f-08f0cd3133cc-cancelling_active_inference_records_uncertain_delivery">
// Guarantee: docs/guarantees/agent/python-requests-have-validated-context.md
#[tokio::test(flavor = "multi_thread")]
async fn cancelling_active_inference_records_uncertain_delivery() {
    let Some(python) = python() else {
        return;
    };
    let app = App::start().await;
    app.configure(&python).await;
    let model = Model::new([json!({"pending_stream":true})]);
    app.state.agent.set_llm_override(model.clone());
    let id = app.send("Cancel during transport", None).await;
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            app.refresh_capture().await;
            if !model.requests.lock().unwrap().is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        app.request("POST", &app.route("/stop"), json!({})).await.0,
        200
    );
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "stopped");
    let source = std::fs::read_to_string(app.root.join(turn["session"].as_str().unwrap())).unwrap();
    assert!(source.contains("cancelled_uncertain"));
    assert!(!source.contains("provider_confirmed"));
    assert_eq!(model.requests.lock().unwrap().len(), 1);
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/agent/src/hickory_agent_worker/context.py](contract.md#output-778ee53bd3ed)
- [apps/agent/src/hickory_agent_worker/core.py](../../verification/private-proposals-wait-for-review/contract.md#output-afffc8ba20eb)
- [apps/agent/src/hickory_agent_worker/hosted.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/hosted.py.md#output-0e03aa3e36b2)
- [apps/agent/src/hickory_agent_worker/runtime.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/runtime.py.md#output-56e1a215e79c)
- [crates/hick-context-store/src/lib.rs](../../../mechanisms/crates/hick-context-store/src/lib.rs.md#output-86b54fda68eb)
- [crates/hickory-cli/src/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/python_agent.rs.md#output-609d98dd1fad)

### Tests and fixtures

- [apps/agent/tests/test_context_runtime.py](contract.md#output-c807e1692e39)
- [crates/hick-context-store/tests/lineage.rs](../../../mechanisms/crates/hick-context-store/tests/lineage.rs.md#output-335caf8febec)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../mechanisms/crates/hickory-cli/tests/serve_python_agent.rs.md#output-08f0cd3133cc)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
