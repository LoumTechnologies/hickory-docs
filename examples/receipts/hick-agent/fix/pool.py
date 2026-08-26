import os

DEFAULT_POOL_SIZE = 10
FIXED_POOL_SIZE = 40
ACQUIRE_TIMEOUT_SECONDS = 2.0

def pool_settings(env=os.environ):
    if env.get('CHECKOUT_POOL_FIX') == '1':
        return {'size': FIXED_POOL_SIZE, 'acquire_timeout': ACQUIRE_TIMEOUT_SECONDS}
    else:
        return {'size': DEFAULT_POOL_SIZE, 'acquire_timeout': None}
