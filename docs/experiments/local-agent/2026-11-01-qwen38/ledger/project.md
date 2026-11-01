<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

# In-Order Balance Processing

The task requires processing a list of signed integer entries against a starting nonnegative balance while preserving the original entry order. The key constraint is that an entry must be rejected if applying it would make the balance negative; rejected entries do not change the balance, and processing continues with subsequent entries. This ordering rule is essential: sorting deposits first would alter which entries are rejected and is therefore incorrect.

The chosen construction is a single linear pass over `entries`. For each entry, the prospective balance is computed as the current balance plus the entry value. If that prospective balance is negative, the zero-based index is recorded in the rejected list and the balance is left unchanged. Otherwise, the entry is applied to the balance. The final result is a dictionary containing the final balance and the list of rejected indices.

<hick:project/>

<hick:file path="solution.py">
def solve(data):
    balance = data["balance"]
    entries = data["entries"]
    rejected = []
    for i, e in enumerate(entries):
        if balance + e < 0:
            rejected.append(i)
        else:
            balance += e
    return {"balance": balance, "rejected": rejected}
</hick:file>

</hick:doc>