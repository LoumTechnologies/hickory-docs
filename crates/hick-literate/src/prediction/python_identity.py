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
