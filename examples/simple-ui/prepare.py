"""Prepare the simple UI demo without overwriting an existing workspace."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess


def main():
    source = Path(__file__).resolve().parent
    root = source.parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dest", type=Path, default=root / ".dev/simple-ui/project")
    parser.add_argument("--record", action="store_true", help="run the supplied cell on the host and cache its real output")
    args = parser.parse_args()
    executable = root / "target/debug/hick"
    if not executable.is_file():
        parser.error("Build the CLI first: cargo build --locked -p hickory-cli --bin hick")
    destination = args.dest.resolve()
    if destination.exists():
        parser.error(f"Preserving existing workspace: {destination}. Choose a new --dest.")
    if source == destination or source in destination.parents:
        parser.error("Choose a destination outside the maintained fixture folder.")
    shutil.copytree(source, destination, ignore=shutil.ignore_patterns("__pycache__", ".hick-cache", "generated"))
    environment = {**os.environ, "HICKORY_EXECUTOR": "local"}
    subprocess.run([str(executable), "project", "materialize", "workbook.md"], cwd=destination, env=environment, check=True)
    command = [str(executable), "run" if args.record else "weave", "README.md"]
    if args.record:
        command.append("--cache")
    subprocess.run(command, cwd=destination, env=environment, check=True)
    print(f"Demo ready: {destination}")
    print(f'HICKORY_PROJECT_DIR="{destination}" just simple-dev browser')


if __name__ == "__main__":
    main()
