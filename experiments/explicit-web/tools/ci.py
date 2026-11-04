import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parents[1]

def run(*args, env=None): subprocess.run(args, cwd=HERE, env=env, check=True)

def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--browser", action="store_true"); args = parser.parse_args()
    if args.browser:
        npm = shutil.which("npm")
        if npm is None: raise SystemExit("Install Node 22+ and npm for the browser witness.")
        run(npm, "ci", "--prefix", "tests", "--ignore-scripts", "--no-audit", "--no-fund")
        run("node", "tests/node_modules/playwright/cli.js", "install", "--with-deps", "chromium")
        env = dict(os.environ, EXPLICIT_PLAYWRIGHT_MODULE=str(HERE / "tests/node_modules/@playwright/test"))
        run(sys.executable, "tests/browser.py", env=env)
        return
    run(sys.executable, "tools/check.py")
    run("dotnet", "run", "--project", "Checks", "--no-build", "--no-launch-profile")
    run(sys.executable, "test_http.py")
    run(sys.executable, "test_http.py", env=dict(os.environ, EXPLICIT_TEST_HOST="Baseline"))
    run(sys.executable, "evaluation/compiler_matrix.py", "--output", "target/compiler.json")

if __name__ == "__main__": main()
