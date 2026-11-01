<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:project/>
<h1>Balance Processing with Entry Rejection</h1>

This solution processes a sequence of financial entries against an initial balance, ensuring that no entry causes the balance to drop below zero. The algorithm maintains the original order of entries and tracks which entries were rejected due to this constraint. It iterates through each entry once, updating the running balance only when an entry is accepted. The rejected indices are collected in their original positions within the input list.

The implementation must handle:
- Starting with a given balance
- Processing entries in their exact order
- Rejecting any entry that would make balance negative
- Returning final balance and list of rejected indices

<hick:file path="solution.py">
def solve(data):
    balance = data["balance"]
    entries = data["entries"]
    rejected = []
    
    for i, entry in enumerate(entries):
        if balance + entry < 0:
            rejected.append(i)
        else:
            balance += entry
    
    return {"balance": balance, "rejected": rejected}
</hick:file>
</hick:doc>