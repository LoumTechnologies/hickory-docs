<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/src/prediction/python_identity.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `c78466ed1976c51baf85815843741ee52b505afa20fb0c88d70c08fb4ae2cdc3`.

<book:entry path="crates/hick-literate/src/prediction/python_identity.py">
"""Fingerprint the explicitly supported private Python runtime.

This does not freeze the host or certify all environment/OS behavior. It emits
hashes only; paths, process environment values and package contents stay local.
"""
import hashlib
import json
import os
from pathlib import Path
import sys
import sysconfig


def files_digest(root):
    hasher = hashlib.sha256()
    root = Path(root)
    for path in sorted(root.rglob('*')):
        if not path.is_file() or any(part in ('__pycache__', 'site-packages', 'dist-packages') for part in path.relative_to(root).parts):
            continue
        if path.suffix not in ('.py', '.so', '.dll', '.pyd', '.zip'):
            continue
        hasher.update(str(path.relative_to(root)).encode())
        hasher.update(b'\0')
        hasher.update(hashlib.sha256(path.read_bytes()).digest())
    return hasher.hexdigest()


result = {
    'interpreter': hashlib.sha256(Path(sys.executable).resolve().read_bytes()).hexdigest(),
    'version': hashlib.sha256(sys.version.encode()).hexdigest(),
    'stdlib': files_digest(sysconfig.get_path('stdlib')),
    'import_environment': hashlib.sha256(json.dumps({name: os.environ.get(name) for name in ('PATH', 'PYTHONPATH', 'PYTHONHOME', 'LD_LIBRARY_PATH', 'LD_PRELOAD')}, sort_keys=True).encode()).hexdigest(),
}
library = Path(sysconfig.get_config_var('LIBDIR') or '') / (sysconfig.get_config_var('LDLIBRARY') or '')
if library.is_file():
    result['runtime_library'] = hashlib.sha256(library.read_bytes()).hexdigest()
if '--adapter' in sys.argv:
    import debugpy
    result['debugpy'] = files_digest(Path(debugpy.__file__).parent)
print(json.dumps(result, sort_keys=True))
</book:entry>

</book:doc>
