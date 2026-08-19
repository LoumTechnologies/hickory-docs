# The self-hosted runner that runs the VM tests

`.github/workflows/vm-tests.yml` runs the download tests on Parallels guests
(`vmkit.conf`, `vmtest/scripts/`). Those need a real Mac with the VMs on it, so
they run on a **self-hosted runner** rather than a GitHub-hosted one. This is
how that runner is set up, and the two things about it that are not obvious.

## What is registered

| | |
|---|---|
| Scope | the **repository** `LoumTechnologies/hickory-docs`, not the organization |
| Name | `macbook-pro-hickory` |
| Labels | `self-hosted, macOS, X64` — exactly what the workflow's `runs-on` asks for |
| Directory | `~/actions-runner-hickory` |
| Service | `actions.runner.LoumTechnologies-hickory-docs.macbook-pro-hickory` (launchd, per-user) |

Repository scope is deliberate: it is the narrower grant, and this is the only
repository with VM tests. Making it organization-wide is a re-run of `config.sh`
against the org URL, not a migration.

## Where the runner directory can live: two rules that fight

This is the part that costs an afternoon, because the two constraints pull in
opposite directions and each failure looks like something else.

**Rule 1 — it must be OUTSIDE `~/Documents`, or the service cannot start.**
`~/Documents`, `~/Desktop` and `~/Downloads` are TCC-protected, and a launchd
agent has no access to them:

```
shell-init: error retrieving current directory: getcwd: cannot access parent
  directories: Operation not permitted
/bin/bash: /Users/…/Documents/src/actions-runner-hickory/runsvc.sh: Operation not permitted
```

The runner then reads `status=offline` on GitHub while `launchctl list` shows it
exiting **126**, and nothing says "permissions". Jobs queue forever rather than
failing — a workflow that never runs looks exactly like one nobody triggered.

**Rule 2 — the Linux and Windows guests read the checkout through the Parallels
share, which by default exposes only Desktop and Documents.** vmkit pushes
scripts into a *macOS* guest over `prlctl exec` (the share is unusable there
under headless TCC), but Linux and Windows guests read them from
`/media/psf/Home/...` and `\\Mac\Home\...`. With the runner outside Documents
those legs fail at the first step:

```
bash: /media/psf/Home/actions-runner-hickory/_work/…/vmtest/scripts/download.sh:
  No such file or directory
```

`prlctl list -i "Ubuntu Linux"` shows why:

```
Host Shared Folders: (-)
Host defined sharing: Off
Shared Profile: (+)
  Use desktop: on
  Use documents: on
```

So "in Documents" satisfies the guests and breaks the service; "outside
Documents" satisfies the service and breaks the guests.

### What does not work, and why it is worth knowing

**A named shared folder does not help.** `prlctl set <vm> --shf-host-add runner
--path ~/actions-runner-hickory` succeeds, and mounts at `/media/psf/runner` —
which vmkit never looks at. Its mapping is hardcoded to the *home* share
(`lib/transport.sh`, `guest_repo`): `$HOME/<rel>` becomes
`/media/psf/Home/<rel>` on Linux and `\\Mac\Home\<rel>` on Windows, and it
refuses outright for a repo outside `$HOME`.

**The setting that would work has no CLI flag.** What is needed is
`<ShareUserHomeDir>` inside `<HostSharing>` in the VM's `config.pvs` — the GUI's
"Share Mac → Home folder". `prlctl set <vm> --shf-host on` is *accepted and
silently does nothing*; every `--shf-*-home` spelling is rejected outright.
Setting it by editing `config.pvs` with the VM stopped does work, and
`prlctl list -i` then reports `Host defined sharing: User home directory`.

**But it does not survive `vmkit reset`, which is the end of it.** Parallels
stores the VM configuration *inside the snapshot*, so reverting to
`portzero-built` restores the old sharing config along with the disk. Measured:
after a revert, `config.pvs` was back to `Enabled=0 ShareUserHomeDir=0` and
`/media/psf/Home` showed only `Desktop Documents Downloads` again, while the
guest that had NOT been reverted kept the change. Since `vmkit test` resets
before every run, any host-side sharing change is undone before the test uses
it.

### What would actually fix it

One of:

1. **Bake it into the `built` snapshot** — revert, enable home sharing, boot,
   re-`vmkit checkpoint` as `built`. This is the real fix, and it rewrites a
   snapshot that another organization's tests also use, so it is a decision
   rather than a step.
2. **Grant the runner Full Disk Access** and keep it in `~/Documents`, which is
   where the other runner on this machine already lives. GUI-only, invisible to
   this repository, and does not survive a rebuild of the machine — but it needs
   no snapshot surgery.
3. **Teach vmkit to push into Linux and Windows guests** the way it already
   pushes into macOS ones (`push_script_macos` in `lib/transport.sh`), which
   would remove the share from the picture entirely. Filed upstream.

Until one of those lands, **only the macOS leg can run** — it is the one that
does not use the share.

**The macOS leg is unaffected either way**, because vmkit tar.gz-pushes into
macOS guests over stdin instead of using the share. That is why it can pass
while the other two legs fail at the first step.

## Installing another one

```sh
cd ~
tar -xzf <actions-runner-osx-x64-*.tar.gz> -C ~/actions-runner-hickory
cd ~/actions-runner-hickory
./config.sh --unattended \
  --url https://github.com/LoumTechnologies/hickory-docs \
  --token "$(gh api -X POST \
      repos/LoumTechnologies/hickory-docs/actions/runners/registration-token \
      --jq .token)" \
  --name macbook-pro-hickory --work _work
./svc.sh install && ./svc.sh start
```

As a **service**, never `./run.sh`. vmkit's `docs/HUMAN-SETUP.md` requires it,
and an interactive runner dies with the terminal that started it.

Check it took: `gh api repos/LoumTechnologies/hickory-docs/actions/runners`
should report `status=online`, and `launchctl list | grep actions.runner`
should show exit status `0` rather than `126`.

## Sharing the machine with another runner

This Mac also hosts a runner for a different organization. Two runners means two
jobs can start at the same moment, and **no `concurrency:` setting in either
repository can see the other one** — GitHub scopes concurrency per repository.

That matters because vmkit resolves contention by *stopping* every other running
VM, so a job landing during another job's test powers off its guest mid-run.
`vmkit-tests.yml` handles it by claiming the host with `vmkit hold` for the
duration of each leg, which makes every other vmkit invocation refuse (exit 1)
rather than start a second VM. See the comments in the workflow, and
PortZeroNetwork/vmkit#11 for making that the tool's default rather than each
caller's job.

## Disk

The VMs are ~200 GB and this repository's `target/` reaches ~19 GB on its own,
on a 466 GB disk. Both the runner's `_work` and Parallels snapshot operations
need headroom; `cargo clean` and clearing `apps/desktop/src-tauri/target` are
the two biggest levers when it gets tight.
