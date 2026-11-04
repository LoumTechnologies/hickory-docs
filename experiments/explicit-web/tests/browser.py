import os
from pathlib import Path
import subprocess
import tempfile
from support import HERE, build, server

if __name__ == "__main__":
    build()
    module = Path(os.environ.get("EXPLICIT_PLAYWRIGHT_MODULE", str(HERE / "tests/node_modules/@playwright/test")))
    if not module.exists(): module = HERE.parents[1] / "apps/web/node_modules/@playwright/test"
    if not module.exists(): raise SystemExit("Install the parent repo's apps/web dependencies before running the browser witness.")
    with tempfile.TemporaryDirectory() as directory, server(directory) as address:
        env = dict(os.environ, EXPLICIT_BROWSER_URL=address, EXPLICIT_PLAYWRIGHT_MODULE=str(module))
        raise SystemExit(subprocess.call(["node", str(HERE / "tests/browser.cjs")], env=env))
