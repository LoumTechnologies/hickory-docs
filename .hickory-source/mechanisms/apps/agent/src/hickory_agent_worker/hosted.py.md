<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/agent/src/hickory_agent_worker/hosted.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0e03aa3e36b2"></a>
## `apps/agent/src/hickory_agent_worker/hosted.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [A running agent can be stopped](../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md), [Model snippets cannot hang or flood an agent session indefinitely](../../../../../guarantees/agent/a-script-cannot-hang-or-flood-a-session/contract.md), [An exhausted Python iteration budget retains incomplete work](../../../../../guarantees/agent/an-exhausted-turn-budget-hands-off/contract.md), [Distinguish an accepted change from a verified runtime result](../../../../../guarantees/agent/completion-does-not-certify-verification/contract.md), [Separate model snippets from trusted host operations](../../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md), [Python model requests carry validated context manifests](../../../../../guarantees/agent/python-requests-have-validated-context/contract.md).

<a id="f-0e03aa3e36b2-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0e03aa3e36b2-expression_statement">
"""Hickory stdio adapter. All agent decisions live in core.Agent."""
</storycode:copy>
<a id="f-0e03aa3e36b2-Path"></a>
### `Path`

<storycode:copy id="f-0e03aa3e36b2-Path">
from pathlib import Path
</storycode:copy>
<a id="f-0e03aa3e36b2-sys"></a>
### `sys`

<storycode:copy id="f-0e03aa3e36b2-sys">
import sys
</storycode:copy>
<a id="f-0e03aa3e36b2-Monty"></a>
### `Monty`

<storycode:copy id="f-0e03aa3e36b2-Monty">
from pydantic_monty import Monty
</storycode:copy>
<a id="f-0e03aa3e36b2-Broker"></a>
### `Broker`

<storycode:copy id="f-0e03aa3e36b2-Broker">
from .runtime import Broker, execute
</storycode:copy>
<a id="f-0e03aa3e36b2-Agent"></a>
### `Agent`

<storycode:copy id="f-0e03aa3e36b2-Agent">
from .core import Agent, Config
</storycode:copy>
<a id="f-0e03aa3e36b2-HostedContext"></a>
### `HostedContext`

<storycode:copy id="f-0e03aa3e36b2-HostedContext">
from .context import HostedContext


</storycode:copy>
<a id="f-0e03aa3e36b2-Model"></a>
### `Model`

<storycode:copy id="f-0e03aa3e36b2-Model">
class Model:
    def __init__(self, broker):
        self.broker = broker

    def complete_request(self, bundle):
        result = self.broker.call('inference', {'manifest': bundle})
        self.broker.context.import_references([result['reference']])
        self.broker.context.preferred[result['text']] = self.broker.context.external[result['reference']]
        return result['text']


</storycode:copy>
<a id="f-0e03aa3e36b2-Tools"></a>
### `Tools`

<storycode:copy id="f-0e03aa3e36b2-Tools">
class Tools:
    def __init__(self, broker):
        self.broker = broker

    def call(self, name, arguments):
        return self.broker.call('tool', {'name': name, 'arguments': arguments})

    def complete(self, summary):
        return self.broker.call('complete', {'summary': summary})


</storycode:copy>
<a id="f-0e03aa3e36b2-monty"></a>
### `monty`

<storycode:copy id="f-0e03aa3e36b2-monty">
def monty():
    return Monty(binary_path=str(Path(sys.executable).parent / 'monty'),
                 min_processes=1, max_processes=1, request_timeout=15)


</storycode:copy>
<a id="f-0e03aa3e36b2-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-0e03aa3e36b2-expression_statement-2">
LIMITS = {'max_memory': 32_000_000, 'max_feed_duration_secs': 5.0,
          'max_turn_duration_secs': 5.0, 'max_suspensions': 64,
          'max_recursion_depth': 100, 'max_total_sleep_secs': 0}


</storycode:copy>
<a id="f-0e03aa3e36b2-main"></a>
### `main`

<storycode:copy id="f-0e03aa3e36b2-main">
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
</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/hosted.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/hosted.py"><storycode:paste select="#f-0e03aa3e36b2-expression_statement"/><storycode:paste select="#f-0e03aa3e36b2-Path"/><storycode:paste select="#f-0e03aa3e36b2-sys"/><storycode:paste select="#f-0e03aa3e36b2-Monty"/><storycode:paste select="#f-0e03aa3e36b2-Broker"/><storycode:paste select="#f-0e03aa3e36b2-Agent"/><storycode:paste select="#f-0e03aa3e36b2-HostedContext"/><storycode:paste select="#f-0e03aa3e36b2-Model"/><storycode:paste select="#f-0e03aa3e36b2-Tools"/><storycode:paste select="#f-0e03aa3e36b2-monty"/><storycode:paste select="#f-0e03aa3e36b2-expression_statement-2"/><storycode:paste select="#f-0e03aa3e36b2-main"/></storycode:file>

</storycode:doc>
