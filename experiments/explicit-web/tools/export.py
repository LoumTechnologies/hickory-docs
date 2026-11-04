import argparse
from pathlib import Path
import re

HERE = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser(); parser.add_argument("destination", type=Path); args = parser.parse_args()
    source_root = HERE.parents[1] / ".hickory-source/mechanisms/explicit-web"
    if not source_root.is_dir(): raise SystemExit("Export from the parent Hickory checkout, where the maintained .hickory-source chapters exist.")
    destination = args.destination.resolve()
    destination.mkdir(parents=True, exist_ok=False)
    source = destination / ".hickory-source"; source.mkdir()
    index = (source_root.parent / "explicit-web-experiment.md").read_text()
    pages = [source_root / name for name in re.findall(r'file="explicit-web/([^"]+\.md)"', index)]
    for page in pages:
        text = re.sub(r'(' + r'<' + r'web:file\s+path=")experiments/explicit-web/', r'\1', page.read_text())
        seen = set()
        def unique(match):
            path = match.group(1)
            if path in seen: return ""
            seen.add(path); return match.group(0)
        text = re.sub(r'<' + r'web:file path="([^"]+)"[^>]*>.*?<' + r'/web:file>', unique, text, flags=re.S)
        (source / page.name).write_text(text)
    includes = "".join('<book:include file=".hickory-source/' + page.name + '" />\n' for page in pages)
    (destination / "project.md").write_text('<book:doc xmlns:book="http://www.hickorydocs.com/1.0">\n# Build Explicit Web\n\n' + includes + '<book:project exclude=".dev/ target/ Core/bin/ Core/obj/ Data/bin/ Data/obj/ Security/bin/ Security/obj/ Hosting/bin/ Hosting/obj/ Server/bin/ Server/obj/ Baseline/bin/ Baseline/obj/ Checks/bin/ Checks/obj/ tests/node_modules/" />\n</book:doc>\n')
    print("Exported maintained source. Run: hick project materialize project.md --root " + str(destination))

if __name__ == "__main__": main()
