---
date: 2026-08-21
---

# The pool fix



What Sam proposed in the meeting, as code: the pool size moves from ten to
forty with a two-second acquire timeout, and it ships behind a config flag
so Friday's deploy can be turned back without a rollback.

Proposal: raise the pool from ten to forty connections and put a two second acquire timeout on it so a stuck connection fails fast instead of backing everything up.


### `fix/pool.py`

```python
"""Checkout DB pool settings. Tangled from fix.hick — edit the document."""
import os

DEFAULT_POOL_SIZE = 10
FIXED_POOL_SIZE = 40
ACQUIRE_TIMEOUT_SECONDS = 2.0


def pool_settings(env=os.environ):
    """The flag CHECKOUT_POOL_FIX=1 selects the larger pool and the timeout."""
    if env.get("CHECKOUT_POOL_FIX") == "1":
        return {"size": FIXED_POOL_SIZE, "acquire_timeout": ACQUIRE_TIMEOUT_SECONDS}
    return {"size": DEFAULT_POOL_SIZE, "acquire_timeout": None}
```





{'size': 10, 'acquire_timeout': None}
{'size': 40, 'acquire_timeout': 2.0}



## Findings




