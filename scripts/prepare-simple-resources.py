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

# Bundle the existing spreadsheet locally for WKWebView and Android WebView.
# esbuild is pinned by the web lockfile; no network or second formula engine.
import json
import subprocess


def bundle_preview(entry, filename, *options):
    web = ROOT / "apps/web"
    resources = ROOT / "apps/apple-simple/Sources/Resources/node_modules"
    resources.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as scratch:
        bundle = Path(scratch) / filename
        subprocess.run([
            str(web / "node_modules/.bin/esbuild"), "--bundle", "--format=iife",
            "--platform=browser", "--jsx=automatic", "--define:import.meta.env={}",
            "--log-level=warning", "--outfile=" + str(bundle), *options,
        ], input=entry, text=True, cwd=web, check=True)
        for output in Path(scratch).iterdir():
            destination = resources / output.name
            data = output.read_bytes()
            if destination.is_file() and destination.read_bytes() == data:
                continue
            with tempfile.NamedTemporaryFile(dir=resources, delete=False) as temporary:
                temporary.write(data)
                staged = temporary.name
            os.chmod(staged, 0o644)
            os.replace(staged, destination)


entry = "import { mountNativeTable, mountNativeOutput } from " + json.dumps(str(ROOT / "apps/web/src/components/MarkdownTable.tsx")) + ";\n" + "import " + json.dumps(str(ROOT / "apps/web/src/styles.css")) + ";\n" + "if (document.body.classList.contains('notebook-mode')) { globalThis.hickoryOutput = mountNativeOutput(document.getElementById('formula')); globalThis.hickoryOutput.update(globalThis.hickoryOutputSource || ''); } else globalThis.hickoryTable = mountNativeTable(document.getElementById('formula'), message => { const json = JSON.stringify(message); if (globalThis.webkit?.messageHandlers?.table) globalThis.webkit.messageHandlers.table.postMessage(JSON.parse(json)); else globalThis.HickoryTable?.postMessage(json); });"
bundle_preview(entry, "table.js")
print("Prepared shared native spreadsheet resources.")

# Bundle the same optional coverage renderer for both native hosts.
entry = "import { mountNativeCoverage } from " + json.dumps(str(ROOT / "apps/web/src/coverage/NativeCoverage.tsx")) + ";\n" + "globalThis.hickoryCoverage = mountNativeCoverage(document.getElementById('surface'), message => { const json = JSON.stringify(message); if (globalThis.webkit?.messageHandlers?.coverage) globalThis.webkit.messageHandlers.coverage.postMessage(JSON.parse(json)); else globalThis.HickoryCoverage?.postMessage(json); });"
bundle_preview(entry, "coverage.js")
print("Prepared shared native coverage resources.")

# Bundle the authoring form with the same parser the browser uses. Embedded
# WASM bytes let the native page stay local with network access disabled.
entry = (
    "import { mountNativeElements } from " + json.dumps(str(ROOT / "apps/web/src/components/ElementAuthoring.tsx")) + ";\n"
    "globalThis.hickoryElements = mountNativeElements(document.getElementById('elements'), message => {"
    "if (globalThis.webkit?.messageHandlers?.elements) globalThis.webkit.messageHandlers.elements.postMessage(message); "
    "else globalThis.HickoryElements?.postMessage(JSON.stringify(message)); });"
)
bundle_preview(entry, "element-authoring.js", "--loader:.wasm=binary", "--minify",
               "--define:process.env.NODE_ENV=\"production\"", "--define:import.meta.url=undefined")
print("Prepared shared native element authoring resources.")
