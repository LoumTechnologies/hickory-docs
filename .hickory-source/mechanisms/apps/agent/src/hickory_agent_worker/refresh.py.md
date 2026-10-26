<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/agent/src/hickory_agent_worker/refresh.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1fafe6145c55"></a>
## `apps/agent/src/hickory_agent_worker/refresh.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Local refresh experiments preserve historical request evidence](../../../../../guarantees/agent/refresh-experiments-preserve-evidence/contract.md).

<a id="f-1fafe6145c55-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-1fafe6145c55-expression_statement">
"""Explicit refresh encodings; scheduling is the caller's responsibility.

Replacement operates only on a newly assembled request and exact old message
bytes. Never rewrite tool-call messages or promote source text to instructions.
"""
</storycode:copy>
<a id="f-1fafe6145c55-dataclass"></a>
### `dataclass`

<storycode:copy id="f-1fafe6145c55-dataclass">
from dataclasses import dataclass
</storycode:copy>
<a id="f-1fafe6145c55-deepcopy"></a>
### `deepcopy`

<storycode:copy id="f-1fafe6145c55-deepcopy">
from copy import deepcopy
</storycode:copy>
<a id="f-1fafe6145c55-json"></a>
### `json`

<storycode:copy id="f-1fafe6145c55-json">
import json


</storycode:copy>
<a id="f-1fafe6145c55-Update"></a>
### `Update`

<storycode:copy id="f-1fafe6145c55-Update">
@dataclass(frozen=True)
class Update:
    source: str
    before: str
    after: str | None
    revision: str
    state: str = 'changed'
    role: str = 'user'
    scope: str = 'conversation'
    parents: tuple = ()


</storycode:copy>
<a id="f-1fafe6145c55-RefreshPolicy"></a>
### `RefreshPolicy`

<storycode:copy id="f-1fafe6145c55-RefreshPolicy">
class RefreshPolicy:
    version = '1'

    def __init__(self, encoding='append'):
        if encoding not in ('append', 'projection', 'reread'):
            raise ValueError('unsupported refresh encoding')
        self.encoding = encoding
        self.name = 'refresh-' + encoding
        self.pending = []

    def queue_update(self, update):
        self.pending.append(update)

    def boundary(self, messages, register):
        pending, self.pending = self.pending, []
        for update in pending:
            # Compatibility with MemoryContext.queue's generic notices.
            if not isinstance(update, Update):
                text, parents = update
                messages.append(dict(role='user', content=text))
                register(text, parents)
                continue
            metadata = dict(source=update.source, revision=update.revision,
                            state=update.state, authority=update.role, scope=update.scope)
            replaced = False
            if self.encoding == 'projection' and update.after is not None:
                for message in messages:
                    if (set(message) == {'role', 'content'} and
                            message['role'] == update.role and
                            message['content'] == update.before):
                        message['content'] = update.after
                        register(update.after, update.parents)
                        replaced = True
            metadata['replaced_prior_read'] = replaced
            # Changed inputs invalidate dependent summaries/plans; do not quietly
            # retain their conclusions as current simply because bytes remain.
            notice = ('Context update; previous plans and summaries based on this source '
                      'are historical and require revalidation. ' + json.dumps(metadata))
            if self.encoding == 'reread':
                notice += '\nReread the source before relying on it; no new bytes supplied.'
            elif not replaced:
                notice += '\nReplacement passage: ' + json.dumps(update.after)
            messages.append(dict(role='user', content=notice))
            register(notice, update.parents)

    def assemble(self, context, messages):
        """Copy before projection so original history/receipts stay immutable."""
        candidate = deepcopy(messages)
        self.boundary(candidate, context.register)
        return context.prepare(candidate)
</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/refresh.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/refresh.py"><storycode:paste select="#f-1fafe6145c55-expression_statement"/><storycode:paste select="#f-1fafe6145c55-dataclass"/><storycode:paste select="#f-1fafe6145c55-deepcopy"/><storycode:paste select="#f-1fafe6145c55-json"/><storycode:paste select="#f-1fafe6145c55-Update"/><storycode:paste select="#f-1fafe6145c55-RefreshPolicy"/></storycode:file>

</storycode:doc>
