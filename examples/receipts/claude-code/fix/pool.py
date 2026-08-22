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
