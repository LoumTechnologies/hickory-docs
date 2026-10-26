<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/prediction/python_identity.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-aafa574a1866"></a>
## `crates/hick-literate/src/prediction/python_identity.py`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../contract.md).

<a id="f-aafa574a1866-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-aafa574a1866-expression_statement">
"""Fingerprint the explicitly supported private Python runtime.

This does not freeze the host or certify all environment/OS behavior. It emits
hashes only; paths, process environment values and package contents stay local.
"""
</storycode:copy>
<a id="f-aafa574a1866-hashlib"></a>
### `hashlib`

<storycode:copy id="f-aafa574a1866-hashlib">
import hashlib
</storycode:copy>
<a id="f-aafa574a1866-json"></a>
### `json`

<storycode:copy id="f-aafa574a1866-json">
import json
</storycode:copy>
<a id="f-aafa574a1866-os"></a>
### `os`

<storycode:copy id="f-aafa574a1866-os">
import os
</storycode:copy>
<a id="f-aafa574a1866-Path"></a>
### `Path`

<storycode:copy id="f-aafa574a1866-Path">
from pathlib import Path
</storycode:copy>
<a id="f-aafa574a1866-sys"></a>
### `sys`

<storycode:copy id="f-aafa574a1866-sys">
import sys
</storycode:copy>
<a id="f-aafa574a1866-sysconfig"></a>
### `sysconfig`

<storycode:copy id="f-aafa574a1866-sysconfig">
import sysconfig


</storycode:copy>
<a id="f-aafa574a1866-files_digest"></a>
### `files_digest`

<storycode:copy id="f-aafa574a1866-files_digest">
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


</storycode:copy>
<a id="f-aafa574a1866-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-aafa574a1866-expression_statement-2">
result = {
    'interpreter': hashlib.sha256(Path(sys.executable).resolve().read_bytes()).hexdigest(),
    'version': hashlib.sha256(sys.version.encode()).hexdigest(),
    'stdlib': files_digest(sysconfig.get_path('stdlib')),
    'import_environment': hashlib.sha256(json.dumps({name: os.environ.get(name) for name in ('PATH', 'PYTHONPATH', 'PYTHONHOME', 'LD_LIBRARY_PATH', 'LD_PRELOAD')}, sort_keys=True).encode()).hexdigest(),
}
</storycode:copy>
<a id="f-aafa574a1866-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-aafa574a1866-expression_statement-3">
library = Path(sysconfig.get_config_var('LIBDIR') or '') / (sysconfig.get_config_var('LDLIBRARY') or '')
</storycode:copy>
<a id="f-aafa574a1866-if_statement"></a>
### `if_statement`

<storycode:copy id="f-aafa574a1866-if_statement">
if library.is_file():
    result['runtime_library'] = hashlib.sha256(library.read_bytes()).hexdigest()
</storycode:copy>
<a id="f-aafa574a1866-if_statement-2"></a>
### `if_statement`

<storycode:copy id="f-aafa574a1866-if_statement-2">
if '--adapter' in sys.argv:
    import debugpy
    result['debugpy'] = files_digest(Path(debugpy.__file__).parent)
</storycode:copy>
<a id="f-aafa574a1866-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-aafa574a1866-expression_statement-4">
print(json.dumps(result, sort_keys=True))
</storycode:copy>

### Assemble `crates/hick-literate/src/prediction/python_identity.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/prediction/python_identity.py"><storycode:paste select="#f-aafa574a1866-expression_statement"/><storycode:paste select="#f-aafa574a1866-hashlib"/><storycode:paste select="#f-aafa574a1866-json"/><storycode:paste select="#f-aafa574a1866-os"/><storycode:paste select="#f-aafa574a1866-Path"/><storycode:paste select="#f-aafa574a1866-sys"/><storycode:paste select="#f-aafa574a1866-sysconfig"/><storycode:paste select="#f-aafa574a1866-files_digest"/><storycode:paste select="#f-aafa574a1866-expression_statement-2"/><storycode:paste select="#f-aafa574a1866-expression_statement-3"/><storycode:paste select="#f-aafa574a1866-if_statement"/><storycode:paste select="#f-aafa574a1866-if_statement-2"/><storycode:paste select="#f-aafa574a1866-expression_statement-4"/></storycode:file>

</storycode:doc>
