"""Agent policy. No Hickory process, UI, credentials, or workspace dependencies.

Inject ModelClient, ToolClient, an executor and a recorder to exercise the same
loop in a test, a terminal, or a confined Hickory worker.
"""
import json
from copy import deepcopy
from dataclasses import dataclass, field
from typing import Protocol, TypedDict

from langgraph.graph import END, START, StateGraph
from .actions import parse_action
from .instructions import INSTRUCTIONS
from .context import ContextClient, MemoryContext


class ModelClient(Protocol):
    def complete(self, messages: list[dict]) -> str: ...


class ToolClient(Protocol):
    def call(self, name: str, arguments: dict): ...
    def complete(self, summary: str) -> dict: ...


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


class State(TypedDict):
    action: dict
    turns: int
    stopped: bool
    summary: str


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
                observe({'ok': False, 'error': str(exc)})
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
