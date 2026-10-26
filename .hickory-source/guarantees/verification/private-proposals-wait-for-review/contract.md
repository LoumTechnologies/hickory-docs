<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Private edit proposals open review before writing

For: Reviewers deciding what an output, recording or prediction actually establishes.

An advertised private edit_doc call previews the captured source and waits for
independent host review before applying a source-bound edit. A prose proposal or
completion summary does not open that queue. The agent cannot supply its own
approval. The private catalogue advertises the supported whole="true" option,
latest whole-surface snapshot and complete replacement input, and identifies
run/after anchors as SHA-256 line hashes rather than source text. Invalid/stale
preview recovery exposes only fixed source/schema guidance, never raw host errors
or held-out target diagnostics. Existing exact-proposal freshness checks remain.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/private-proposals-wait-for-review.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-afffc8ba20eb"></a>
## `apps/agent/src/hickory_agent_worker/core.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [A running agent can be stopped](../../agent/a-running-agent-can-be-stopped/contract.md), [Model snippets cannot hang or flood an agent session indefinitely](../../agent/a-script-cannot-hang-or-flood-a-session/contract.md), [An exhausted Python iteration budget retains incomplete work](../../agent/an-exhausted-turn-budget-hands-off/contract.md), [Distinguish an accepted change from a verified runtime result](../../agent/completion-does-not-certify-verification/contract.md), [Separate model snippets from trusted host operations](../../agent/python-execution-has-two-boundaries/contract.md), [Python model requests carry validated context manifests](../../agent/python-requests-have-validated-context/contract.md), [Private edit proposals open review before writing](contract.md).

<a id="f-afffc8ba20eb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-afffc8ba20eb-expression_statement">
"""Agent policy. No Hickory process, UI, credentials, or workspace dependencies.

Inject ModelClient, ToolClient, an executor and a recorder to exercise the same
loop in a test, a terminal, or a confined Hickory worker.
"""
</storycode:copy>
<a id="f-afffc8ba20eb-json"></a>
### `json`

<storycode:copy id="f-afffc8ba20eb-json">
import json
</storycode:copy>
<a id="f-afffc8ba20eb-deepcopy"></a>
### `deepcopy`

<storycode:copy id="f-afffc8ba20eb-deepcopy">
from copy import deepcopy
</storycode:copy>
<a id="f-afffc8ba20eb-dataclass"></a>
### `dataclass`

<storycode:copy id="f-afffc8ba20eb-dataclass">
from dataclasses import dataclass, field
</storycode:copy>
<a id="f-afffc8ba20eb-Protocol"></a>
### `Protocol`

<storycode:copy id="f-afffc8ba20eb-Protocol">
from typing import Protocol, TypedDict

</storycode:copy>
<a id="f-afffc8ba20eb-END"></a>
### `END`

<storycode:copy id="f-afffc8ba20eb-END">
from langgraph.graph import END, START, StateGraph
</storycode:copy>
<a id="f-afffc8ba20eb-parse_action"></a>
### `parse_action`

<storycode:copy id="f-afffc8ba20eb-parse_action">
from .actions import parse_action
</storycode:copy>
<a id="f-afffc8ba20eb-INSTRUCTIONS"></a>
### `INSTRUCTIONS`

<storycode:copy id="f-afffc8ba20eb-INSTRUCTIONS">
from .instructions import INSTRUCTIONS
</storycode:copy>
<a id="f-afffc8ba20eb-ContextClient"></a>
### `ContextClient`

<storycode:copy id="f-afffc8ba20eb-ContextClient">
from .context import ContextClient, MemoryContext


</storycode:copy>
<a id="f-afffc8ba20eb-ModelClient"></a>
### `ModelClient`

<storycode:copy id="f-afffc8ba20eb-ModelClient">
class ModelClient(Protocol):
    def complete(self, messages: list[dict]) -> str: ...


</storycode:copy>
<a id="f-afffc8ba20eb-ToolClient"></a>
### `ToolClient`

<storycode:copy id="f-afffc8ba20eb-ToolClient">
class ToolClient(Protocol):
    def call(self, name: str, arguments: dict): ...
    def complete(self, summary: str) -> dict: ...


</storycode:copy>
<a id="f-afffc8ba20eb-Config"></a>
### `Config`

<storycode:copy id="f-afffc8ba20eb-Config">
@dataclass
class Config:
    prompt: str
    max_turns: int = 20
    model_retries: int = 2
    context_limit: int = 700_000
    context: str = ''
    document: str | None = None
    prior_turns: list[dict] = field(default_factory=list)
    tools: dict = field(default_factory=dict)
    context_references: list[str] = field(default_factory=list)


</storycode:copy>
<a id="f-afffc8ba20eb-State"></a>
### `State`

<storycode:copy id="f-afffc8ba20eb-State">
class State(TypedDict):
    action: dict
    turns: int
    stopped: bool
    summary: str


</storycode:copy>
<a id="f-afffc8ba20eb-Agent"></a>
### `Agent`

<storycode:copy id="f-afffc8ba20eb-Agent">
class Agent:
    def __init__(self, model: ModelClient, tools: ToolClient, execute,
                 record=lambda kind, data: None, cancelled=lambda: False, context: ContextClient | None = None):
        self.model, self.tools, self.execute = model, tools, execute
        self.record, self.cancelled = record, cancelled
        self.messages = []
        self.context = context if context is not None else MemoryContext()

    def check_cancelled(self):
        if self.cancelled():
            raise RuntimeError('Agent cancelled')

    def run(self, config: Config) -> str:
        if not 1 <= config.max_turns <= 100:
            raise ValueError('max_turns must be between 1 and 100')
        if not 0 <= config.model_retries <= 5:
            raise ValueError('model_retries must be between 0 and 5')
        # Build a fresh branch here. The host supplies data, never prompt policy.
        self.messages = [{'role': 'system', 'content': INSTRUCTIONS +
                          '\nTool schemas:\n' + json.dumps(config.tools)}]
        for turn in config.prior_turns:
            self.messages.extend([{'role': 'user', 'content': turn['prompt']},
                                  {'role': 'assistant', 'content': turn['answer']}])
        if config.context:
            self.messages.append({'role': 'user', 'content':
                'Editor context (user content):\n' + reading_context(config.context)})
        if config.document:
            self.messages.append({'role': 'user', 'content':
                'Default document: ' + config.document + '. Read before editing.'})
        if config.context and config.context_references:
            self.context.register('Editor context (user content):\n' + reading_context(config.context), config.context_references)
        self.messages.append({'role': 'user', 'content': config.prompt})

        def observe(data):
            self.record('observation', data)
            text = 'Worker observation (not independent verification): ' + json.dumps(data)
            self.context.register(text, getattr(self.context, 'references', ()))
            self.messages.append({'role': 'user', 'content': text})

        def model(state):
            self.check_cancelled()
            self.context.boundary(self.messages)
            if len(json.dumps(self.messages).encode()) > config.context_limit:
                raise RuntimeError('Conversation context budget exhausted; start a new turn')
            bundle = self.context.prepare(self.messages)
            self.record('context-prepared', deepcopy(bundle))
            for attempt in range(config.model_retries + 1):
                try:
                    if any('artifact' in part for part in bundle['request']['parts']) and not hasattr(self.model, 'complete_request'):
                        raise RuntimeError('Model adapter cannot deliver selected image artifacts; use an artifact-capable adapter')
                    raw = (self.model.complete_request(deepcopy(bundle)) if hasattr(self.model, 'complete_request')
                           else self.model.complete([{'role': p['role'], 'content': p['text']}
                               for p in bundle['request']['parts']]))
                    break
                except Exception as exc:
                    self.check_cancelled()
                    self.record('model-error', {'error': str(exc), 'attempt': attempt})
                    if attempt == config.model_retries:
                        raise
            self.context.response(raw, bundle)
            self.messages.append({'role': 'assistant', 'content': raw})
            try:
                action = parse_action(raw)
            except (ValueError, TypeError) as exc:
                observe({'ok': False, 'kind': 'model_protocol_error',
                         'tool_execution_started': False, 'error': str(exc),
                         'repair': 'Your response was not an action object. No snippet or tool ran. '
                                   'Reply with ONLY one raw JSON object: {"python":"snippet"} '
                                   'or {"done":"summary with limitations"}. '
                                   'Do not include prose, Markdown fences, or reasoning outside the object. '
                                   'Every action in the rejected response is unexecuted, including valid-looking '
                                   'objects. Retry the first unfinished requested step; do not skip ahead to edits.'})
                action = {}
            return {'action': action, 'turns': state['turns'] + 1}

        def act(state):
            self.check_cancelled()
            action = state['action']
            if 'done' in action:
                try:
                    receipt = self.tools.complete(action['done'])
                except Exception as exc:
                    receipt = {'accepted': False, 'error': str(exc)}
                self.check_cancelled()
                if receipt.get('accepted') is True:
                    return {'stopped': True, 'summary': action['done']}
                observe(receipt)
            elif 'python' in action:
                self.record('action', {'code': action['python'], 'interpreter': 'monty'})
                observe(self.execute(action['python']))
            return {'stopped': False}

        graph = StateGraph(State)
        graph.add_node('model', model)
        graph.add_node('act', act)
        graph.add_edge(START, 'model')
        graph.add_edge('model', 'act')
        graph.add_conditional_edges('act', lambda state: END if state['stopped'] or
            state['turns'] >= config.max_turns else 'model')
        result = graph.compile().invoke(
            {'action': {}, 'turns': 0, 'stopped': False, 'summary': ''},
            {'recursion_limit': config.max_turns * 3 + 5})
        if not result['stopped']:
            raise RuntimeError('Agent iteration limit exhausted without verified completion')
        return result['summary']


</storycode:copy>
<a id="f-afffc8ba20eb-reading_context"></a>
### `reading_context`

<storycode:copy id="f-afffc8ba20eb-reading_context">
def reading_context(raw):
    """Python owns selection of model-visible material; edit snapshots stay intact."""
    try:
        value = json.loads(raw)
    except (ValueError, TypeError):
        return raw
    if not isinstance(value, dict):
        return raw
    editors = value.get('editors', value)
    if isinstance(editors, dict) and isinstance(editors.get('projection'), dict):
        for buffer in editors.get('buffers', []):
            buffer.pop('content', None)
            buffer.pop('observations', None)
    return json.dumps(value, ensure_ascii=False)
</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/core.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/core.py"><storycode:paste select="#f-afffc8ba20eb-expression_statement"/><storycode:paste select="#f-afffc8ba20eb-json"/><storycode:paste select="#f-afffc8ba20eb-deepcopy"/><storycode:paste select="#f-afffc8ba20eb-dataclass"/><storycode:paste select="#f-afffc8ba20eb-Protocol"/><storycode:paste select="#f-afffc8ba20eb-END"/><storycode:paste select="#f-afffc8ba20eb-parse_action"/><storycode:paste select="#f-afffc8ba20eb-INSTRUCTIONS"/><storycode:paste select="#f-afffc8ba20eb-ContextClient"/><storycode:paste select="#f-afffc8ba20eb-ModelClient"/><storycode:paste select="#f-afffc8ba20eb-ToolClient"/><storycode:paste select="#f-afffc8ba20eb-Config"/><storycode:paste select="#f-afffc8ba20eb-State"/><storycode:paste select="#f-afffc8ba20eb-Agent"/><storycode:paste select="#f-afffc8ba20eb-reading_context"/></storycode:file>

## Implementation review

### Implementation

- [apps/agent/src/hickory_agent_worker/core.py](contract.md#output-afffc8ba20eb)
- [apps/agent/src/hickory_agent_worker/instructions.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/instructions.py.md#output-4915d386d720)
- [apps/web/src/components/PrivatePredictionCoach.tsx](../../../mechanisms/apps/web/src/components/PrivatePredictionCoach.tsx.md#output-f32ded7fd60b)
- [crates/hickory-agent/src/llm_openai.rs](../../../mechanisms/crates/hickory-agent/src/llm_openai.rs.md#output-0150d11c8f60)
- [crates/hickory-agent/src/provider.rs](../../../mechanisms/crates/hickory-agent/src/provider.rs.md#output-2be2d28e151b)
- [crates/hickory-cli/src/engine/mcp.rs](../../../mechanisms/crates/hickory-cli/src/engine/mcp.rs.md#output-d68d35ce1f7a)
- [crates/hickory-cli/src/serve/private_predictions.rs](../../../mechanisms/crates/hickory-cli/src/serve/private_predictions.rs.md#output-5eb3e4172fd6)
- [scripts/test-private-prediction-ui.sh](../../../mechanisms/scripts/test-private-prediction-ui.sh.md#output-d93b734fd0b3)

### Tests and fixtures

- [apps/agent/tests/test_worker.py](../../../mechanisms/apps/agent/tests/test_worker.py.md#output-9f33e52791d8)
- [apps/web/e2e/privatePredictionCoach.spec.ts](../../../mechanisms/apps/web/e2e/privatePredictionCoach.spec.ts.md#output-c4a491bb0b6d)
- [apps/web/src/components/PrivatePredictionCoach.test.tsx](../../../mechanisms/apps/web/src/components/PrivatePredictionCoach.tsx.md#output-4b3612758c98)
- [crates/hickory-cli/src/serve/private_prediction_tests.rs](../../../mechanisms/crates/hickory-cli/src/serve/private_prediction_tests.rs.md#output-7cb981639695)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
