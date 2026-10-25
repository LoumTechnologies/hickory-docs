<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/receipts/hick-agent/fix/pool.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ab342d98e18c6e991ae2f2208d309912caed907c67d3f7a21f901b3006351412`.

<book:entry path="examples/receipts/hick-agent/fix/pool.py">
import os

DEFAULT_POOL_SIZE = 10
FIXED_POOL_SIZE = 40
ACQUIRE_TIMEOUT_SECONDS = 2.0

def pool_settings(env=os.environ):
    if env.get('CHECKOUT_POOL_FIX') == '1':
        return {'size': FIXED_POOL_SIZE, 'acquire_timeout': ACQUIRE_TIMEOUT_SECONDS}
    else:
        return {'size': DEFAULT_POOL_SIZE, 'acquire_timeout': None}
</book:entry>

</book:doc>
