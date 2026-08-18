#!/usr/bin/env python3
"""Does anything in a crate's dependency tree spawn a process?

The check a compiler cannot do. A crate that shells out cross-compiles for
`aarch64-apple-ios` perfectly and dies the first time it runs, because iOS does
not permit a sandboxed app to fork/exec. `hick-store` is exactly that shape:
it compiles for Android and calls `git init --bare`.

`build.rs` is excluded deliberately — a build script runs on a build machine,
never on a phone, so spawning there is not a portability problem. Tests,
benches, and examples are excluded for the same reason.

Usage: python3 scripts/audit-spawns.py <path-to-crate>

See docs/specs/freeform/shipping-mobile-and-desktop.md and
experiments/git-libraries/README.md.
"""

import os, re, subprocess, sys, glob

candidate = sys.argv[1]
out = subprocess.run(["cargo","tree","--prefix","none","--no-dedupe","-e","normal"],
                     cwd=candidate, capture_output=True, text=True)
pkgs = set()
for line in out.stdout.splitlines():
    m = re.match(r"^([A-Za-z0-9_.-]+) v([0-9][^ ]*)", line.strip())
    if m: pkgs.add((m.group(1), m.group(2)))

roots = glob.glob(os.path.expanduser("~/.cargo/registry/src/*"))
spawners, scanned, missing = {}, 0, 0
for name, ver in sorted(pkgs):
    d = None
    for r in roots:
        p = os.path.join(r, f"{name}-{ver}")
        if os.path.isdir(p): d = p; break
    if d is None:
        missing += 1
        continue
    scanned += 1
    src = os.path.join(d, "src")
    hits = 0
    for dirpath, _, files in os.walk(src if os.path.isdir(src) else d):
        # build.rs spawns at BUILD time on a real machine; it never runs on a
        # device, so it is not a portability problem.
        if "/tests/" in dirpath or "/benches/" in dirpath or "/examples/" in dirpath:
            continue
        for f in files:
            if not f.endswith(".rs") or f == "build.rs": continue
            try: text = open(os.path.join(dirpath,f), encoding="utf-8", errors="ignore").read()
            except OSError: continue
            for ln in text.splitlines():
                t = ln.lstrip()
                if t.startswith("//") or t.startswith("*"): continue
                if "Command::new" in ln or "process::Command" in ln:
                    hits += 1
    if hits: spawners[f"{name} {ver}"] = hits

print(f"packages in tree: {len(pkgs)}  scanned: {scanned}  sources not found: {missing}")
print(f"packages that spawn a process: {len(spawners)}")
for k in sorted(spawners, key=lambda k: -spawners[k]):
    print(f"  {k}: {spawners[k]} site(s)")
