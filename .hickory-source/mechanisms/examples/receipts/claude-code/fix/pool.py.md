<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain examples/receipts/claude-code/fix/pool.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-179bc63093d9"></a>
## `examples/receipts/claude-code/fix/pool.py`

<a id="f-179bc63093d9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-179bc63093d9-expression_statement">
"""Checkout DB pool settings. Tangled from fix.md — edit the document."""
</storycode:copy>
<a id="f-179bc63093d9-os"></a>
### `os`

<storycode:copy id="f-179bc63093d9-os">
import os

</storycode:copy>
<a id="f-179bc63093d9-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-179bc63093d9-expression_statement-2">
DEFAULT_POOL_SIZE = 10
</storycode:copy>
<a id="f-179bc63093d9-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-179bc63093d9-expression_statement-3">
FIXED_POOL_SIZE = 40
</storycode:copy>
<a id="f-179bc63093d9-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-179bc63093d9-expression_statement-4">
ACQUIRE_TIMEOUT_SECONDS = 2.0


</storycode:copy>
<a id="f-179bc63093d9-pool_settings"></a>
### `pool_settings`

<storycode:copy id="f-179bc63093d9-pool_settings">
def pool_settings(env=os.environ):
    """The flag CHECKOUT_POOL_FIX=1 selects the larger pool and the timeout."""
    if env.get("CHECKOUT_POOL_FIX") == "1":
        return {"size": FIXED_POOL_SIZE, "acquire_timeout": ACQUIRE_TIMEOUT_SECONDS}
    return {"size": DEFAULT_POOL_SIZE, "acquire_timeout": None}
</storycode:copy>

### Assemble `examples/receipts/claude-code/fix/pool.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="examples/receipts/claude-code/fix/pool.py"><storycode:paste select="#f-179bc63093d9-expression_statement"/><storycode:paste select="#f-179bc63093d9-os"/><storycode:paste select="#f-179bc63093d9-expression_statement-2"/><storycode:paste select="#f-179bc63093d9-expression_statement-3"/><storycode:paste select="#f-179bc63093d9-expression_statement-4"/><storycode:paste select="#f-179bc63093d9-pool_settings"/></storycode:file>

</storycode:doc>
