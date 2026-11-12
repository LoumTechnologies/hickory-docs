"""Copy locked local math assets into the native resource bundle; never download."""
from pathlib import Path
import os
import tempfile

ROOT = Path(__file__).resolve().parents[1]
source = ROOT / "apps/web/node_modules/katex/dist"
target = ROOT / "apps/apple-simple/Sources/Resources/katex"
if not (source / "katex.min.js").is_file():
    print("Native math will keep showing source. Run npm --prefix apps/web ci, then rerun scripts/prepare-simple-resources.py for typesetting.")
else:
    target.mkdir(parents=True, exist_ok=True)
    files = [source / "katex.min.js", source / "katex.min.css", *sorted((source / "fonts").glob("*"))]
    for original in files:
        destination = target / original.relative_to(source)
        data = original.read_bytes()
        if destination.is_file() and destination.read_bytes() == data:
            destination.chmod(0o644)
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as temporary:
            temporary.write(data)
            staged = temporary.name
        os.chmod(staged, 0o644)
        os.replace(staged, destination)
    print("Prepared local native math resources.")
# Mermaid's UMD bundle is self-contained; native previews never fetch a CDN.
mermaid = ROOT / "apps/web/node_modules/mermaid/dist/mermaid.min.js"
if mermaid.is_file():
    destination = ROOT / "apps/apple-simple/Sources/Resources/node_modules/mermaid.min.js"
    destination.parent.mkdir(parents=True, exist_ok=True)
    data = mermaid.read_bytes()
    if not destination.is_file() or destination.read_bytes() != data:
        with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as temporary:
            temporary.write(data)
            staged = temporary.name
        os.chmod(staged, 0o644)
        os.replace(staged, destination)
    print("Prepared local native diagram resources.")
# Reuse the same emulator for program output and authored expectations.
for package, original, name in [("@xterm/xterm", "lib/xterm.js", "xterm.js"), ("@xterm/xterm", "css/xterm.css", "xterm.css"), ("@xterm/addon-fit", "lib/addon-fit.js", "addon-fit.js")]:
    original = ROOT / "apps/web/node_modules" / package / original
    if not original.is_file():
        print("Terminal assets missing. Run npm --prefix apps/web ci, then prepare resources again."); continue
    destination = ROOT / "apps/apple-simple/Sources/Resources/node_modules" / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    data = original.read_bytes()
    if not destination.is_file() or destination.read_bytes() != data:
        with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as temporary:
            temporary.write(data); staged = temporary.name
        os.chmod(staged, 0o644); os.replace(staged, destination)
