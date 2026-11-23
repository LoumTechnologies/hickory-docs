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
    parser.add_argument("--dest", type=Path, default=root / ".dev/simple-ui/code-review")
    parser.add_argument("--record", action="store_true", help="run the supplied cells on the host and cache their real output")
    args = parser.parse_args()
    executable = root / "target/debug" / ("hick.exe" if os.name == "nt" else "hick")
    if not executable.is_file():
        parser.error("Build the CLI first: cargo build --locked -p hickory-cli --bin hick")
    destination = args.dest.resolve()
    if destination.exists():
        parser.error(f"Preserving existing workspace: {destination}. Choose a new --dest.")
    if source == destination or source in destination.parents:
        parser.error("Choose a destination outside the maintained fixture folder.")
    shutil.copytree(source, destination, ignore=shutil.ignore_patterns(
        "__pycache__", ".git", ".hick-cache", ".hick-journal", ".hick-lsp.json", "generated", "sessions",
    ))
    environment = {**os.environ, "HICKORY_EXECUTOR": "local"}
    subprocess.run([str(executable), "project", "materialize", "project.md"], cwd=destination, env=environment, check=True)
    command = [str(executable), "run" if args.record else "weave", "README.md"]
    if args.record:
        command.append("--cache")
    subprocess.run(command, cwd=destination, env=environment, check=True)
    command[2] = "tables.md"
    subprocess.run(command, cwd=destination, env=environment, check=True)
    # The sample uses its own Git identity and configuration. Parent index
    # settings and the developer's commit templates must not enter the demo.
    git_environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    git_environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
    for arguments in [
        ["init", "--quiet", "--template="],
        ["add", "--", "."],
        ["-c", "user.name=Hickory Simple", "-c", "user.email=demo@example.invalid",
         "commit", "--quiet", "--message", "Prepare the corner shop workbook"],
    ]:
        subprocess.run(["git", *arguments], cwd=destination, env=git_environment, check=True)
    print(f"Demo ready: {destination}")
    print(f'HICKORY_PROJECT_DIR="{destination}" HICKORY_OPEN_FILE=workbook.md just simple-dev browser')


if __name__ == "__main__":
    main()
