<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/tools/reconciliation_audit.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-17709284026f"></a>
## `hickory-dogfood/tools/reconciliation_audit.py`

Construct and verify the maintained literate project, preserving migration evidence separately from current verification.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="hickory-dogfood/tools/reconciliation_audit.py">
"""Check exact original paragraph coverage without asserting semantic correctness."""
from collections import defaultdict
import hashlib

ACTIONS={
 'historical-contract',
 'replaced-by-reconciled-summary-with-original-retained',
 'retained-requirement-or-context',
 'retained-follow-up-for-review',
}


def validate(ledger, contracts, read_original):
    by_origin=defaultdict(list)
    expected={record['origin']:record for record in contracts.values()}
    for entry in ledger:
        if entry['origin'] not in expected:
            raise ValueError('Ledger contains an unknown original: '+entry['origin'])
        by_origin[entry['origin']].append(entry)
    if set(by_origin)!=set(expected):
        raise ValueError('A contract has no paragraph disposition.')
    for origin,record in expected.items():
        raw=read_original(origin)
        cursor=0
        for entry in by_origin[origin]:
            if entry['guarantee']!=record['id'] or entry['destination']!=record['page']:
                raise ValueError('Paragraph assigned to a different requirement: '+origin)
            if entry['action'] not in ACTIONS:
                raise ValueError('Unknown paragraph disposition: '+origin)
            if entry['start_byte']!=cursor or not cursor<entry['end_byte']<=len(raw):
                raise ValueError('Missing, overlapping or invalid paragraph range: '+origin)
            chunk=raw[cursor:entry['end_byte']]
            if hashlib.sha256(chunk).hexdigest()!=entry['sha256']:
                raise ValueError('Paragraph bytes differ from their recorded identity: '+origin)
            cursor=entry['end_byte']
        if cursor!=len(raw):
            raise ValueError('Original tail has no disposition: '+origin)
    return len(by_origin),len(ledger)
</storycode:entry>

</storycode:doc>
