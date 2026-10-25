<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/agent/src/hickory_agent_worker/refresh.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Local refresh experiments preserve historical request evidence](../../../../../../guarantees/agent/refresh-experiments-preserve-evidence/contract.md)

Captured SHA-256: `59f13b6d474c0acbf0cc6c6aee8fa519f73ff6a54d1dda98fa1abbfc2b7618d9`.

<book:entry path="apps/agent/src/hickory_agent_worker/refresh.py">
"""Explicit refresh encodings; scheduling is the caller's responsibility.

Replacement operates only on a newly assembled request and exact old message
bytes. Never rewrite tool-call messages or promote source text to instructions.
"""
from dataclasses import dataclass
from copy import deepcopy
import json


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
</book:entry>

</book:doc>
