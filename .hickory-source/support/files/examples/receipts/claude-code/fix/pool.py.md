<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/receipts/claude-code/fix/pool.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cdefbf7c575657d7858bb1b99835a2a68c900bc5cd665b5fdefb4d79a608a864`.

<book:entry path="examples/receipts/claude-code/fix/pool.py">
"""Checkout DB pool settings. Tangled from fix.md — edit the document."""
import os

DEFAULT_POOL_SIZE = 10
FIXED_POOL_SIZE = 40
ACQUIRE_TIMEOUT_SECONDS = 2.0


def pool_settings(env=os.environ):
    """The flag CHECKOUT_POOL_FIX=1 selects the larger pool and the timeout."""
    if env.get("CHECKOUT_POOL_FIX") == "1":
        return {"size": FIXED_POOL_SIZE, "acquire_timeout": ACQUIRE_TIMEOUT_SECONDS}
    return {"size": DEFAULT_POOL_SIZE, "acquire_timeout": None}
</book:entry>

</book:doc>
