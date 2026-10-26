<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain examples/receipts/hick-agent/fix/pool.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c1c69267331d"></a>
## `examples/receipts/hick-agent/fix/pool.py`

<a id="f-c1c69267331d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c1c69267331d-expression_statement">
import os

DEFAULT_POOL_SIZE = 10
</storycode:copy>
<a id="f-c1c69267331d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c1c69267331d-expression_statement-2">
FIXED_POOL_SIZE = 40
</storycode:copy>
<a id="f-c1c69267331d-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c1c69267331d-expression_statement-3">
ACQUIRE_TIMEOUT_SECONDS = 2.0

</storycode:copy>
<a id="f-c1c69267331d-pool_settings"></a>
### `pool_settings`

<storycode:copy id="f-c1c69267331d-pool_settings">
def pool_settings(env=os.environ):
    if env.get('CHECKOUT_POOL_FIX') == '1':
        return {'size': FIXED_POOL_SIZE, 'acquire_timeout': ACQUIRE_TIMEOUT_SECONDS}
    else:
        return {'size': DEFAULT_POOL_SIZE, 'acquire_timeout': None}
</storycode:copy>

### Assemble `examples/receipts/hick-agent/fix/pool.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="examples/receipts/hick-agent/fix/pool.py"><storycode:paste select="#f-c1c69267331d-expression_statement"/><storycode:paste select="#f-c1c69267331d-expression_statement-2"/><storycode:paste select="#f-c1c69267331d-expression_statement-3"/><storycode:paste select="#f-c1c69267331d-pool_settings"/></storycode:file>

</storycode:doc>
