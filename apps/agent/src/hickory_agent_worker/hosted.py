"""Hickory stdio adapter. All agent decisions live in core.Agent."""
from pathlib import Path
import sys
from pydantic_monty import Monty
from .runtime import Broker, execute
from .core import Agent, Config, ModelResponse
from .context import HostedContext


class Model:
    def __init__(self, broker):
        self.broker = broker

    def complete_request(self, bundle):
        result = self.broker.call('inference', {'manifest': bundle})
        self.broker.context.import_references([result['reference']])
        self.broker.context.preferred[result['text']] = self.broker.context.external[result['reference']]
        return ModelResponse(result['text'], result.get('finish_reason'), result.get('metrics', {}))


class Tools:
    def __init__(self, broker):
        self.broker = broker
        self.handoff_pending = False

    def call(self, name, arguments):
        result = self.broker.call('tool', {'name': name, 'arguments': arguments})
        if name == 'submit_changeset' and not result.get('isError'):
            import json
            for part in result.get('content', []):
                if part.get('type') == 'text':
                    try:
                        self.handoff_pending |= json.loads(part['text']).get('accepted') is True
                    except (ValueError, TypeError, AttributeError):
                        pass
        return result

    def take_handoff(self):
        if not self.handoff_pending:
            return None
        self.handoff_pending = False
        return self.broker.call('handoff', {})

    def complete(self, summary):
        return self.broker.call('complete', {'summary': summary})


def monty():
    return Monty(binary_path=str(Path(sys.executable).parent / 'monty'),
                 min_processes=1, max_processes=1, request_timeout=15)


LIMITS = {'max_memory': 32_000_000, 'max_feed_duration_secs': 5.0,
          'max_turn_duration_secs': 5.0, 'max_suspensions': 64,
          'max_recursion_depth': 100, 'max_total_sleep_secs': 0}


def main():
    broker = Broker()
    bootstrap = broker.receive()
    if bootstrap.get('probe'):
        with monty() as pool:
            with pool.checkout(limits=LIMITS) as session:
                if execute(session, broker, '1 + 2')['result'] != 3:
                    raise RuntimeError('Monty startup failed')
        broker.send({'method': 'ready'})
        return
    try:
        with monty() as pool:
            with pool.checkout(limits=LIMITS) as session:
                agent = Agent(Model(broker), Tools(broker),
                    lambda code: execute(session, broker, code),
                    lambda kind, data: broker.call('record', {'kind': kind, 'data': data}),
                    context=HostedContext(broker, bootstrap.get('context_references', [])))
                agent.run(Config(**bootstrap))
    except Exception as exc:
        broker.send({'method': 'exit', 'params': {'error': str(exc)}})
        return
    broker.send({'method': 'exit'})
