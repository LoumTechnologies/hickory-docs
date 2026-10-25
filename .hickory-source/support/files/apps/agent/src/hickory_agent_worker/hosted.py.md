<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/src/hickory_agent_worker/hosted.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A running agent can be stopped](../../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md)
- [Model snippets cannot hang or flood an agent session indefinitely](../../../../../../guarantees/agent/a-script-cannot-hang-or-flood-a-session/contract.md)
- [An exhausted Python iteration budget retains incomplete work](../../../../../../guarantees/agent/an-exhausted-turn-budget-hands-off/contract.md)
- [Completion does not certify runtime verification](../../../../../../guarantees/agent/completion-does-not-certify-verification/contract.md)
- [Python agent code crosses two explicit boundaries](../../../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md)
- [Python model requests carry validated context manifests](../../../../../../guarantees/agent/python-requests-have-validated-context/contract.md)

Captured SHA-256: `9a3389edee39c55734dc66f678f46ead9edf1b47c4e5c87b8b8faede00796e33`.

<book:entry path="apps/agent/src/hickory_agent_worker/hosted.py">
"""Hickory stdio adapter. All agent decisions live in core.Agent."""
from pathlib import Path
import sys
from pydantic_monty import Monty
from .runtime import Broker, execute
from .core import Agent, Config
from .context import HostedContext


class Model:
    def __init__(self, broker):
        self.broker = broker

    def complete_request(self, bundle):
        result = self.broker.call('inference', {'manifest': bundle})
        self.broker.context.import_references([result['reference']])
        self.broker.context.preferred[result['text']] = self.broker.context.external[result['reference']]
        return result['text']


class Tools:
    def __init__(self, broker):
        self.broker = broker

    def call(self, name, arguments):
        return self.broker.call('tool', {'name': name, 'arguments': arguments})

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
</book:entry>

</book:doc>
