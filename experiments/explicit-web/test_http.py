import subprocess
import sys
from pathlib import Path

if __name__ == "__main__":
    raise SystemExit(subprocess.call([sys.executable, "-m", "unittest", "discover", "-s", "tests", "-p", "test_*.py", "-v"], cwd=Path(__file__).resolve().parent))
