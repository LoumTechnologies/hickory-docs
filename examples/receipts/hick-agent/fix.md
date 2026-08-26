---
date: 2026-08-21
---

# The pool fix



## Proposal

Proposal: raise the pool from ten to forty connections and put a two second acquire timeout on it so a stuck connection fails fast instead of backing everything up.

## Implementation





### `fix/pool.py`

```python
import os

DEFAULT_POOL_SIZE = 10
FIXED_POOL_SIZE = 40
ACQUIRE_TIMEOUT_SECONDS = 2.0

def pool_settings(env=os.environ):
    if env.get('CHECKOUT_POOL_FIX') == '1':
        return {'size': FIXED_POOL_SIZE, 'acquire_timeout': ACQUIRE_TIMEOUT_SECONDS}
    else:
        return {'size': DEFAULT_POOL_SIZE, 'acquire_timeout': None}
```


{'size': 10, 'acquire_timeout': None}
{'size': 40, 'acquire_timeout': 2.0}



## Findings




