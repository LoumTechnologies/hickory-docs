<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/audit-spawns.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d27db94b5f64"></a>
## `scripts/audit-spawns.py`

<a id="f-d27db94b5f64-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d27db94b5f64-expression_statement">
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

</storycode:copy>
<a id="f-d27db94b5f64-os"></a>
### `os`

<storycode:copy id="f-d27db94b5f64-os">
import os, re, subprocess, sys, glob

</storycode:copy>
<a id="f-d27db94b5f64-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d27db94b5f64-expression_statement-2">
candidate = sys.argv[1]
</storycode:copy>
<a id="f-d27db94b5f64-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d27db94b5f64-expression_statement-3">
out = subprocess.run(["cargo","tree","--prefix","none","--no-dedupe","-e","normal"],
                     cwd=candidate, capture_output=True, text=True)
</storycode:copy>
<a id="f-d27db94b5f64-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-d27db94b5f64-expression_statement-4">
pkgs = set()
</storycode:copy>
<a id="f-d27db94b5f64-for_statement"></a>
### `for_statement`

<storycode:copy id="f-d27db94b5f64-for_statement">
for line in out.stdout.splitlines():
    m = re.match(r"^([A-Za-z0-9_.-]+) v([0-9][^ ]*)", line.strip())
    if m: pkgs.add((m.group(1), m.group(2)))

</storycode:copy>
<a id="f-d27db94b5f64-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-d27db94b5f64-expression_statement-5">
roots = glob.glob(os.path.expanduser("~/.cargo/registry/src/*"))
</storycode:copy>
<a id="f-d27db94b5f64-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-d27db94b5f64-expression_statement-6">
spawners, scanned, missing = {}, 0, 0
</storycode:copy>
<a id="f-d27db94b5f64-for_statement-2"></a>
### `for_statement`

<storycode:copy id="f-d27db94b5f64-for_statement-2">
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

</storycode:copy>
<a id="f-d27db94b5f64-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-d27db94b5f64-expression_statement-7">
print(f"packages in tree: {len(pkgs)}  scanned: {scanned}  sources not found: {missing}")
</storycode:copy>
<a id="f-d27db94b5f64-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-d27db94b5f64-expression_statement-8">
print(f"packages that spawn a process: {len(spawners)}")
</storycode:copy>
<a id="f-d27db94b5f64-for_statement-3"></a>
### `for_statement`

<storycode:copy id="f-d27db94b5f64-for_statement-3">
for k in sorted(spawners, key=lambda k: -spawners[k]):
    print(f"  {k}: {spawners[k]} site(s)")
</storycode:copy>

### Assemble `scripts/audit-spawns.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/audit-spawns.py" executable="true"><storycode:paste select="#f-d27db94b5f64-expression_statement"/><storycode:paste select="#f-d27db94b5f64-os"/><storycode:paste select="#f-d27db94b5f64-expression_statement-2"/><storycode:paste select="#f-d27db94b5f64-expression_statement-3"/><storycode:paste select="#f-d27db94b5f64-expression_statement-4"/><storycode:paste select="#f-d27db94b5f64-for_statement"/><storycode:paste select="#f-d27db94b5f64-expression_statement-5"/><storycode:paste select="#f-d27db94b5f64-expression_statement-6"/><storycode:paste select="#f-d27db94b5f64-for_statement-2"/><storycode:paste select="#f-d27db94b5f64-expression_statement-7"/><storycode:paste select="#f-d27db94b5f64-expression_statement-8"/><storycode:paste select="#f-d27db94b5f64-for_statement-3"/></storycode:file>

</storycode:doc>
