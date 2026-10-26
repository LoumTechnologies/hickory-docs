<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Publish the committed Unstable build chosen by the maintainer

For: Maintainers publishing installers and people checking what a download contains.

Publish Unstable from the configured Linux-to-Mac workflow. The payloads must come from the committed revision; the tool must not invent a green-CI gate that this workflow explicitly removed.

## What must happen

Manual Unstable publication builds the exact committed HEAD present on GitHub through the configured Apple Silicon SSH builder. It uploads the DMG and signed app archive, then switches the updater manifest only after payload uploads succeed. Existing assets stay downloadable. Publishers using that builder share its lock. The command neither dispatches GitHub Actions nor gates or monitors regression results; the maintainer chooses publication timing. Stable release behavior is separate.

## Limits of the evidence

No publication is authorized or performed by this rewrite. Builder/signing configuration and real installability remain unverified. Manifest replacement is not atomic across GitHub failures, and separate builders can race.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/only-a-green-commit-becomes-downloadable.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: The explicit October 25 manual-publication requirement supersedes the older automatic green-CI-only Unstable contract. Keep old stable-channel rules historical until separately reviewed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-c2750bfe4a13"></a>
## `scripts/release-unstable.py`

Related contracts: [Publish the committed Unstable build chosen by the maintainer](contract.md).

<a id="f-c2750bfe4a13-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c2750bfe4a13-expression_statement">
#!/usr/bin/env python3
"""Publish the committed checkout using an Apple Silicon SSH builder, without Actions."""
</storycode:copy>
<a id="f-c2750bfe4a13-datetime-as-dt"></a>
### `datetime as dt`

<storycode:copy id="f-c2750bfe4a13-datetime-as-dt">
import datetime as dt
</storycode:copy>
<a id="f-c2750bfe4a13-json"></a>
### `json`

<storycode:copy id="f-c2750bfe4a13-json">
import json
</storycode:copy>
<a id="f-c2750bfe4a13-os"></a>
### `os`

<storycode:copy id="f-c2750bfe4a13-os">
import os
</storycode:copy>
<a id="f-c2750bfe4a13-re"></a>
### `re`

<storycode:copy id="f-c2750bfe4a13-re">
import re
</storycode:copy>
<a id="f-c2750bfe4a13-Path"></a>
### `Path`

<storycode:copy id="f-c2750bfe4a13-Path">
from pathlib import Path
</storycode:copy>
<a id="f-c2750bfe4a13-shlex"></a>
### `shlex`

<storycode:copy id="f-c2750bfe4a13-shlex">
import shlex
</storycode:copy>
<a id="f-c2750bfe4a13-subprocess-as-sp"></a>
### `subprocess as sp`

<storycode:copy id="f-c2750bfe4a13-subprocess-as-sp">
import subprocess as sp
</storycode:copy>
<a id="f-c2750bfe4a13-tempfile"></a>
### `tempfile`

<storycode:copy id="f-c2750bfe4a13-tempfile">
import tempfile

</storycode:copy>
<a id="f-c2750bfe4a13-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c2750bfe4a13-expression_statement-2">
REPO = "LoumTechnologies/hickory-docs"
</storycode:copy>
<a id="f-c2750bfe4a13-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c2750bfe4a13-expression_statement-3">
ROOT = Path(__file__).resolve().parents[1]


</storycode:copy>
<a id="f-c2750bfe4a13-run"></a>
### `run`

<storycode:copy id="f-c2750bfe4a13-run">
def run(*args, **kwargs):
    return sp.run(args, check=True, **kwargs)


</storycode:copy>
<a id="f-c2750bfe4a13-output"></a>
### `output`

<storycode:copy id="f-c2750bfe4a13-output">
def output(*args):
    return sp.check_output(args, text=True).strip()


</storycode:copy>
<a id="f-c2750bfe4a13-remote"></a>
### `remote`

<storycode:copy id="f-c2750bfe4a13-remote">
def remote(host, command, **kwargs):
    # SSH joins arguments into shell text; quote both shell layers explicitly.
    return run("ssh", "-oBatchMode=yes", "--", host,
               "bash -lc " + shlex.quote(command), **kwargs)


</storycode:copy>
<a id="f-c2750bfe4a13-manifest"></a>
### `manifest`

<storycode:copy id="f-c2750bfe4a13-manifest">
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


</storycode:copy>
<a id="f-c2750bfe4a13-publish"></a>
### `publish`

<storycode:copy id="f-c2750bfe4a13-publish">
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


</storycode:copy>
<a id="f-c2750bfe4a13-release_exists"></a>
### `release_exists`

<storycode:copy id="f-c2750bfe4a13-release_exists">
def release_exists():
    # A network/auth error must not be treated as a missing release.
    response = run("gh", "api", "--paginate", "--slurp", f"repos/{REPO}/releases", stdout=sp.PIPE, text=True)
    return any(release["tag_name"] == "unstable" for page in json.loads(response.stdout) for release in page)


</storycode:copy>
<a id="f-c2750bfe4a13-main"></a>
### `main`

<storycode:copy id="f-c2750bfe4a13-main">
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


</storycode:copy>
<a id="f-c2750bfe4a13-if_statement"></a>
### `if_statement`

<storycode:copy id="f-c2750bfe4a13-if_statement">
if __name__ == "__main__":
    try:
        main()
    except (ValueError, sp.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
</storycode:copy>

### Assemble `scripts/release-unstable.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/release-unstable.py" executable="true"><storycode:paste select="#f-c2750bfe4a13-expression_statement"/><storycode:paste select="#f-c2750bfe4a13-datetime-as-dt"/><storycode:paste select="#f-c2750bfe4a13-json"/><storycode:paste select="#f-c2750bfe4a13-os"/><storycode:paste select="#f-c2750bfe4a13-re"/><storycode:paste select="#f-c2750bfe4a13-Path"/><storycode:paste select="#f-c2750bfe4a13-shlex"/><storycode:paste select="#f-c2750bfe4a13-subprocess-as-sp"/><storycode:paste select="#f-c2750bfe4a13-tempfile"/><storycode:paste select="#f-c2750bfe4a13-expression_statement-2"/><storycode:paste select="#f-c2750bfe4a13-expression_statement-3"/><storycode:paste select="#f-c2750bfe4a13-run"/><storycode:paste select="#f-c2750bfe4a13-output"/><storycode:paste select="#f-c2750bfe4a13-remote"/><storycode:paste select="#f-c2750bfe4a13-manifest"/><storycode:paste select="#f-c2750bfe4a13-publish"/><storycode:paste select="#f-c2750bfe4a13-release_exists"/><storycode:paste select="#f-c2750bfe4a13-main"/><storycode:paste select="#f-c2750bfe4a13-if_statement"/></storycode:file>

## Implementation review

### Implementation

- [scripts/build-unstable-macos.sh](../../../mechanisms/scripts/build-unstable-macos.sh.md#output-f2dcad3b78b2)
- [scripts/dist-desktop.sh](../../../mechanisms/scripts/dist-desktop.sh.md#output-928dfd6fa282)
- [scripts/release-unstable.py](contract.md#output-c2750bfe4a13)

### Tests and fixtures

- [scripts/tests/test_release_unstable.py](../../../mechanisms/scripts/tests/test_release_unstable.py.md#output-66686df94df4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
