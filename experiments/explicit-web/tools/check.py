from pathlib import Path
import py_compile
import subprocess
import sys

HERE = Path(__file__).resolve().parents[1]

def main():
    for path in HERE.rglob("*"):
        if any(part in {"bin", "obj", "node_modules", ".dev", "target", "__pycache__"} for part in path.parts): continue
        if path.suffix in {".cs", ".py", ".cjs"} and path.is_file():
            count = len(path.read_text().splitlines())
            if count > 220: raise SystemExit(f"{path.relative_to(HERE)} has {count} lines; split at a responsibility boundary (limit 220).")
            if path.suffix == ".py": py_compile.compile(str(path), doraise=True)
    return subprocess.call(["dotnet", "build", "Checks/Checks.csproj", "--nologo", "-m:1", "-nr:false"], cwd=HERE)

if __name__ == "__main__": raise SystemExit(main())
