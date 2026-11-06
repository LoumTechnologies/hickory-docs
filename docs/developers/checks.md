# Visualize local checks

Open **Checks** in the bottom status bar. It shows up to the provider’s latest
200 runs, filtered to the opened folder, including workflow children when the
provider records them. Status refreshes every three seconds while the tab is open.
An unavailable provider leaves earlier results visibly stale. Opening the tab
never starts a service or a workload.

From this repository root:

```sh
# Build local-ci separately, or use an already installed binary.
just -f ../local-ci/justfile build
just checks-configure ../local-ci/target/debug/local-ci just check
just checks-submit
just checks-provider-status
```

`checks-configure` accepts an executable path followed by command argv. Omit argv
for `just check`. Arguments containing spaces must be quoted individually. The
configuration file printed by that command stores the canonical executable, command argv,
and a `workflow` boolean (default false). It lives under Hickory’s per-user app data
directory (`HICKORY_STATE_DIR` when set), in `checks/`, keyed by the canonical
opened folder. Repository files cannot enable this integration or choose an executable. Set `workflow` true for a controller that
registers child checks with local-ci. Ordinary subprocesses appear as one aggregate
run. Configure each opened folder separately; no Git repository is required for
workspace runs. A workspace run uses live files and has no immutable revision.
The repository recipes build the Hickory adapter as needed; Rust/Cargo is required.

Select post-commit behavior independently:

```sh
just checks-hook local-ci  # submit the exact commit to the configured provider
just checks-hook local    # existing local runner (default)
just checks-hook off      # no automatic checks
```

This sets clone-local `hickory.postCommitChecks`; it does not replace hooks or
change the staged source verification gate. Install this checkout’s hooks with
`just literate-bootstrap` on a fresh clone. The hook captures the commit ID before
detaching; subsequent commits cannot change which revision that submission uses.
Git returns without waiting for checks. Explicit submissions can spend time exporting
large commits and have no client deadline; observation requests are limited to 15
seconds. Submission errors go to the Git common
directory’s `checks/hook.log`. A failed submission never claims the commit passed.

For a manual immutable run, pass a full commit ID to `just checks-submit COMMIT`.
local-ci’s optional Git adapter exports committed bytes without Git metadata;
choose a workload that can run in that exported tree. `just check` is the default
compile check. `just ci` requests the broader repository suite and may require
additional tools and installed dependencies. `just checks` needs Git metadata and
is intended for the existing local runner rather than an exported tree.

To stop using the provider, select `just checks-hook local` or `off`, then run
`just checks-configure --off`. This removes only that folder’s integration config;
it does not cancel local-ci runs or stop its service. Configure a rebuilt binary
again if its path changes.

Monitor results yourself with the Checks pane, `just checks-provider-status`, or
`local-ci runs` / `local-ci status RUN_ID`. For the existing runner use
`just checks-status`. Read `checks/hook.log` if a commit has no corresponding run.
Broad regression execution and monitoring remain user-owned.

Focused verification: `just checks-integration-test`. To exercise the real
provider and detached Git hook in disposable workspaces, run
`just checks-integration-test /absolute/path/to/local-ci`. This does not run the
repository’s broad regression workload.
