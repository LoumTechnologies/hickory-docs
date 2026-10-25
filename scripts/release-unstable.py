#!/usr/bin/env python3
"""Publish the committed checkout using an Apple Silicon SSH builder, without Actions."""
import datetime as dt
import json
import os
import re
from pathlib import Path
import shlex
import subprocess as sp
import tempfile

REPO = "LoumTechnologies/hickory-docs"
ROOT = Path(__file__).resolve().parents[1]


def run(*args, **kwargs):
    return sp.run(args, check=True, **kwargs)


def output(*args):
    return sp.check_output(args, text=True).strip()


def remote(host, command, **kwargs):
    # SSH joins arguments into shell text; quote both shell layers explicitly.
    return run("ssh", "-oBatchMode=yes", "--", host,
               "bash -lc " + shlex.quote(command), **kwargs)


def manifest(version, signature, archive, sha):
    if not signature.strip():
        raise ValueError("The updater signature is empty")
    return {
        "version": version,
        "notes": f"Unstable build from {sha}. Save your work and reopen the app after installation.",
        "pub_date": dt.datetime.now(dt.timezone.utc).isoformat(),
        "platforms": {"darwin-aarch64": {
            "signature": signature.strip(),
            "url": f"https://github.com/{REPO}/releases/download/unstable/{archive}",
        }},
    }


def publish(directory, version, sha):
    """Upload immutable payloads first, then switch the updater manifest last."""
    prefix = f"hickory-docs-{version}-aarch64-apple-darwin"
    archive = directory / (prefix + ".app.tar.gz")
    signature = directory / (archive.name + ".sig")
    dmg = directory / (prefix + ".dmg")
    for path in (archive, signature, dmg):
        if not path.is_file() or path.stat().st_size == 0:
            raise ValueError(f"Missing build artifact: {path.name}")
    latest = directory / "unstable.json"
    latest.write_text(json.dumps(manifest(version, signature.read_text(), archive.name, sha), indent=2) + "\n")
    if not release_exists():
        run("gh", "release", "create", "unstable", "--repo", REPO, "--title", "Unstable",
            "--prerelease", "--target", sha, "--notes", "Manually published Apple Silicon desktop builds.")
    # Existing archives stay available to clients that already fetched an older manifest.
    payloads = sorted(str(p) for p in directory.iterdir() if p.is_file() and p != latest)
    run("gh", "release", "upload", "unstable", "--repo", REPO, *payloads)
    run("gh", "release", "upload", "unstable", "--repo", REPO, str(latest), "--clobber")
    # Only this designated rolling tag moves; stable version tags are untouched.
    run("gh", "api", f"repos/{REPO}/git/refs/tags/unstable", "--method", "PATCH",
        "-f", f"sha={sha}", "-F", "force=true")
    notes = directory / "release-notes.md"
    notes.write_text(f"Apple Silicon desktop {version}, built from `{sha}`.\n\n"
                     "Published manually over SSH. Other-platform and CLI assets may be older.\n")
    run("gh", "release", "edit", "unstable", "--repo", REPO, "--title", "Unstable",
        "--prerelease", "--notes-file", str(notes))


def release_exists():
    # A network/auth error must not be treated as a missing release.
    response = run("gh", "api", "--paginate", "--slurp", f"repos/{REPO}/releases", stdout=sp.PIPE, text=True)
    return any(release["tag_name"] == "unstable" for page in json.loads(response.stdout) for release in page)


def main():
    os.chdir(ROOT)
    host = os.environ.get("HICKORY_MAC_BUILDER", "")
    if not host or host.startswith("-"):
        raise ValueError("Set HICKORY_MAC_BUILDER to an Apple Silicon Mac's SSH alias (see apps/desktop/RELEASING.md)")
    if output("git", "status", "--porcelain", "--untracked-files=no"):
        raise ValueError("Commit tracked changes before publishing; only HEAD is built")
    sha = output("git", "rev-parse", "HEAD")
    # Publishing only pushed commits makes every installed build reproducible.
    run("gh", "api", f"repos/{REPO}/commits/{sha}", stdout=sp.DEVNULL)
    base = json.loads((ROOT / "apps/desktop/src-tauri/tauri.conf.json").read_text())["version"].split("-")[0]
    # A lock on the shared builder covers build AND publication, including competing Linux agents.
    acquire = '''set -e
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || { echo 'Builder must be an Apple Silicon Mac' >&2; exit 1; }
cache="$HOME/.cache/hickory-docs-unstable"
mkdir -p "$cache"
mkdir "$cache/publish.lock" || { echo 'Another publisher holds ~/.cache/hickory-docs-unstable/publish.lock' >&2; exit 1; }
mktemp -d "$cache/build.XXXXXX"
'''
    result = remote(host, acquire, stdout=sp.PIPE, text=True)
    work = result.stdout.strip()
    if not re.fullmatch(r"/[^\n]+/\.cache/hickory-docs-unstable/build\.[A-Za-z0-9]+", work):
        raise ValueError("Builder returned an invalid build path")
    quoted = shlex.quote(work)
    try:
        stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%d%H%M%S%f")
        version = f"{base}-unstable.{stamp}.g{sha[:8]}"
        with tempfile.TemporaryDirectory(prefix="hickory-unstable-") as temp:
            temp = Path(temp)
            source = temp / "source.tgz"
            with source.open("wb") as stream:
                run("git", "archive", "--format=tar.gz", sha, stdout=stream)
            with source.open("rb") as stream:
                remote(host, f"tar -xzf - -C {quoted}", stdin=stream)
            remote(host, f"cd {quoted} && bash scripts/build-unstable-macos.sh {shlex.quote(version)}")
            artifacts = temp / "artifacts"
            artifacts.mkdir()
            # Fetch fixed names through tar; the builder needs no GitHub credentials.
            with (temp / "artifacts.tar").open("wb") as stream:
                remote(host, f"tar -cf - -C {quoted}/dist/desktop .", stdout=stream)
            import tarfile
            with tarfile.open(temp / "artifacts.tar") as tar:
                tar.extractall(artifacts, filter="data")
            publish(artifacts, version, sha)
    finally:
        remote(host, f"rm -rf -- {quoted}; rmdir \"$HOME/.cache/hickory-docs-unstable/publish.lock\"")
    print(f"Published {version}: https://github.com/{REPO}/releases/tag/unstable")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, sp.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
