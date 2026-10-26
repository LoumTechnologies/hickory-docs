<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Pair identity without silently granting execution

For: An engineer pairing their own machines and reviewing a lost machine's access.

Pairing the same key twice must not restore an execute grant that was revoked. Registering identity alone must not be presented as an established connection.

## What must happen

Each machine retains its private identity locally and pairs by exchanging public identity. Fleet membership is mutual. Grants are per machine; execution is initially off, pairing is idempotent, conflicting identities under one name are refused, and damaged invitations fail. Registering a key does not by itself establish a network connection. Optional peer connectivity is governed by the peer-channel contract.


## Limits of the evidence

The original contract also describes platform and invitation cases. Those clauses remain in historical evidence and require a full fleet review before the replacement can be accepted as exhaustive. This page is partially reviewed, not a certification of every pairing path.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/a-machine-is-a-keypair.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Scope nothing-is-reachable to identity registration; remove the obsolete implication that the product has no optional connectivity.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-379c44af631b"></a>
## `apps/web/src/views/toolTabs.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Pair identity without silently granting execution](contract.md), [A Local Install Is a Launchable macOS App](../../release/a-local-install-is-a-launchable-macos-app/contract.md).

<a id="f-379c44af631b-targets"></a>
### `targets`

Every tool pane is reachable from the shell.
This exists because two of them were NOT: `FleetPane` and `MergedView` were
written, tested, and rendered by nothing — their own tests passed because a
test renders a component directly, which is exactly the thing a user cannot
do. A component with no route is dead code that looks finished.
docs/specs/freeform/shell-layouts.md

<storycode:copy id="f-379c44af631b-targets">
// Every tool pane is reachable from the shell.
//
// This exists because two of them were NOT: `FleetPane` and `MergedView` were
// written, tested, and rendered by nothing — their own tests passed because a
// test renders a component directly, which is exactly the thing a user cannot
// do. A component with no route is dead code that looks finished.
//
// docs/specs/freeform/shell-layouts.md
import { describe, expect, it } from "vitest";

import {
  FLEET_TAB,
  GIT_TAB,
  MERGED_TAB,
  WELCOME_TAB,
  openFleetTab,
  openGitTab,
  openMergedTab,
  openWelcomeTab,
  initialWorkspace,
} from "./workspaceState";
import { panes } from "../shell/layout";

const targets = (layout: ReturnType<typeof initialWorkspace>) =>
  [...panes(layout.root)].flatMap((p) => p.tabs.map((t) => t.target));

</storycode:copy>
<a id="f-379c44af631b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-379c44af631b-expression_statement">
describe("the tool tabs a person can open", () => {
  it("opens the fleet, and fronts the existing one rather than duplicating", () => {
    let layout = openFleetTab(initialWorkspace());
    expect(targets(layout)).toContain(FLEET_TAB);
    layout = openFleetTab(layout);
    expect(targets(layout).filter((t) => t === FLEET_TAB)).toHaveLength(1);
  });

  it("opens a merged view per path, because the view IS of a path", () => {
    // Two files compared at once are two tabs, not one confused one.
    let layout = openMergedTab(initialWorkspace(), "src/a.rs");
    layout = openMergedTab(layout, "src/b.rs");
    expect(targets(layout)).toContain(`${MERGED_TAB}src/a.rs`);
    expect(targets(layout)).toContain(`${MERGED_TAB}src/b.rs`);
    // And the same path twice is still one tab.
    layout = openMergedTab(layout, "src/a.rs");
    expect(targets(layout).filter((t) => t === `${MERGED_TAB}src/a.rs`)).toHaveLength(1);
  });

  it("still opens the ones that already worked", () => {
    expect(targets(openGitTab(initialWorkspace()))).toContain(GIT_TAB);
    expect(targets(openWelcomeTab(initialWorkspace()))).toContain(WELCOME_TAB);
  });

  it("gives every tool tab a name a person can read on the strip", () => {
    for (const layout of [
      openFleetTab(initialWorkspace()),
      openGitTab(initialWorkspace()),
      openMergedTab(initialWorkspace(), "src/a.rs"),
    ]) {
      for (const pane of panes(layout.root)) {
        for (const tab of pane.tabs) {
          // A tool tab must name itself; a document tab is titled from its
          // path by the strip, so only tool tabs are asserted here.
          if (tab.kind === "tool") {
            expect((tab.title ?? "").trim().length).toBeGreaterThan(0);
          }
        }
      }
    }
  });
});
</storycode:copy>

### Assemble `apps/web/src/views/toolTabs.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/toolTabs.test.ts"><storycode:paste select="#f-379c44af631b-targets"/><storycode:paste select="#f-379c44af631b-expression_statement"/></storycode:file>

<a id="output-cccf3953dc0f"></a>
## `crates/hickory-cli/tests/fleet.rs`

Related contracts: [Pair identity without silently granting execution](contract.md).

<a id="f-cccf3953dc0f-hick"></a>
### `hick`

`hick fleet` — identity for one engineer's several machines.
Protects `docs/guarantees/collaboration/a-machine-is-a-keypair.md`.
Nothing here reaches anything: this is the part that must be right first.
Two machines are two state directories, and the whole ceremony happens by
copying a line between them — which is the point, because the only
rendezvous available to two machines that cannot yet reach each other is
one we would have to run, and `local-only.md` deletes that.

<storycode:copy id="f-cccf3953dc0f-hick">
//! `hick fleet` — identity for one engineer's several machines.
//!
//! Protects `docs/guarantees/collaboration/a-machine-is-a-keypair.md`.
//!
//! Nothing here reaches anything: this is the part that must be right first.
//! Two machines are two state directories, and the whole ceremony happens by
//! copying a line between them — which is the point, because the only
//! rendezvous available to two machines that cannot yet reach each other is
//! one we would have to run, and `local-only.md` deletes that.

use std::path::Path;
use std::process::Command;

fn hick(state: &Path, machine: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    cmd.env("HICKORY_STATE_DIR", state);
    cmd.env("HICKORY_MACHINE_NAME", machine);
    cmd
}

</storycode:copy>
<a id="f-cccf3953dc0f-run"></a>
### `run`

<storycode:copy id="f-cccf3953dc0f-run">
fn run(state: &Path, machine: &str, args: &[&str]) -> (bool, String, String) {
    let out = hick(state, machine).args(args).output().expect("run hick");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

</storycode:copy>
<a id="f-cccf3953dc0f-two_machines_pair_by_copying_a_line_in_each_direction"></a>
### `two_machines_pair_by_copying_a_line_in_each_direction`

<storycode:copy id="f-cccf3953dc0f-two_machines_pair_by_copying_a_line_in_each_direction">
#[test]
fn two_machines_pair_by_copying_a_line_in_each_direction() {
    let laptop = tempfile::tempdir().unwrap();
    let desktop = tempfile::tempdir().unwrap();

    let (ok, invite_from_desktop, stderr) = run(desktop.path(), "desktop", &["fleet", "invite"]);
    assert!(ok, "{stderr}");
    assert!(
        invite_from_desktop.starts_with("hick-fleet:"),
        "{invite_from_desktop}"
    );
    // The ceremony is mutual, and the tool says so rather than leaving one
    // side half-paired.
    assert!(stderr.contains("MUTUAL"), "{stderr}");

    let (ok, added, stderr) = run(
        laptop.path(),
        "laptop",
        &["fleet", "accept", invite_from_desktop.trim()],
    );
    assert!(ok, "{stderr}");
    assert!(added.contains("desktop"), "{added}");
    assert!(added.contains("SHA256:"), "{added}");
    assert!(added.contains("view, edit"), "{added}");
    // Identity, not connectivity — said where somebody would otherwise assume
    // the machines can now see each other.
    assert!(added.contains("Nothing is reachable yet"), "{added}");

    // And back the other way.
    let (_, invite_from_laptop, _) = run(laptop.path(), "laptop", &["fleet", "invite"]);
    let (ok, _, stderr) = run(
        desktop.path(),
        "desktop",
        &["fleet", "accept", invite_from_laptop.trim()],
    );
    assert!(ok, "{stderr}");

    let (_, listed, _) = run(laptop.path(), "laptop", &["fleet", "list"]);
    assert!(listed.contains("this machine: laptop"), "{listed}");
    assert!(listed.contains("desktop"), "{listed}");
}

</storycode:copy>
<a id="f-cccf3953dc0f-execute_is_off_until_it_is_given_deliberately"></a>
### `execute_is_off_until_it_is_given_deliberately`

<storycode:copy id="f-cccf3953dc0f-execute_is_off_until_it_is_given_deliberately">
#[test]
fn execute_is_off_until_it_is_given_deliberately() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let (_, invite, _) = run(b.path(), "agent-box", &["fleet", "invite"]);
    run(a.path(), "laptop", &["fleet", "accept", invite.trim()]);

    let (_, listed, _) = run(a.path(), "laptop", &["fleet", "list"]);
    assert!(
        listed.contains("edit,view") || listed.contains("view,edit"),
        "{listed}"
    );
    assert!(!listed.contains("execute"), "{listed}");

    let (ok, granted, stderr) = run(
        a.path(),
        "laptop",
        &["fleet", "grant", "agent-box", "execute"],
    );
    assert!(ok, "{stderr}");
    assert!(granted.contains("execute"), "{granted}");
    // What it costs is a property of the machine, not of the grant.
    assert!(granted.contains("confined to"), "{granted}");

    let (ok, revoked, _) = run(
        a.path(),
        "laptop",
        &["fleet", "grant", "agent-box", "execute", "--revoke"],
    );
    assert!(ok);
    assert!(!revoked.contains("execute"), "{revoked}");
}

</storycode:copy>
<a id="f-cccf3953dc0f-a_phone_can_never_be_granted_execute"></a>
### `a_phone_can_never_be_granted_execute`

<storycode:copy id="f-cccf3953dc0f-a_phone_can_never_be_granted_execute">
#[test]
fn a_phone_can_never_be_granted_execute() {
    // It reads and captures; it cannot spawn a subprocess, so there is no
    // executor to grant.
    let a = tempfile::tempdir().unwrap();
    let phone = tempfile::tempdir().unwrap();
    let (_, invite, _) = run(phone.path(), "phone", &["fleet", "invite", "--phone"]);
    run(a.path(), "laptop", &["fleet", "accept", invite.trim()]);

    let (ok, _, stderr) = run(a.path(), "laptop", &["fleet", "grant", "phone", "execute"]);
    assert!(!ok, "{stderr}");
    assert!(stderr.contains("no executor to grant"), "{stderr}");
}

</storycode:copy>
<a id="f-cccf3953dc0f-revoking_a_machine_is_deleting_its_key"></a>
### `revoking_a_machine_is_deleting_its_key`

<storycode:copy id="f-cccf3953dc0f-revoking_a_machine_is_deleting_its_key">
#[test]
fn revoking_a_machine_is_deleting_its_key() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let (_, invite, _) = run(b.path(), "stolen", &["fleet", "invite"]);
    run(a.path(), "laptop", &["fleet", "accept", invite.trim()]);

    let (ok, removed, _) = run(a.path(), "laptop", &["fleet", "remove", "stolen"]);
    assert!(ok);
    assert!(
        removed.contains("no server holds a session you cannot reach"),
        "{removed}"
    );
    let (_, listed, _) = run(a.path(), "laptop", &["fleet", "list"]);
    assert!(!listed.contains("stolen"), "{listed}");
}

</storycode:copy>
<a id="f-cccf3953dc0f-the_key_lives_outside_every_repository"></a>
### `the_key_lives_outside_every_repository`

<storycode:copy id="f-cccf3953dc0f-the_key_lives_outside_every_repository">
#[test]
fn the_key_lives_outside_every_repository() {
    // Not in the project, for the reason hickory-workspace already gives: git
    // would eventually commit it.
    let state = tempfile::tempdir().unwrap();
    let (ok, out, stderr) = run(state.path(), "laptop", &["fleet", "whoami"]);
    assert!(ok, "{stderr}");
    assert!(out.contains("SHA256:"), "{out}");
    assert!(state.path().join("fleet/machine.key").exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(state.path().join("fleet/machine.key"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "the private half is owner-only");
    }
}

</storycode:copy>
<a id="f-cccf3953dc0f-a_damaged_invitation_is_refused_and_nothing_is_added"></a>
### `a_damaged_invitation_is_refused_and_nothing_is_added`

<storycode:copy id="f-cccf3953dc0f-a_damaged_invitation_is_refused_and_nothing_is_added">
#[test]
fn a_damaged_invitation_is_refused_and_nothing_is_added() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let (_, invite, _) = run(b.path(), "desktop", &["fleet", "invite"]);
    let cut = &invite.trim()[..invite.trim().len() - 4];

    let (ok, _, stderr) = run(a.path(), "laptop", &["fleet", "accept", cut]);
    assert!(!ok, "{stderr}");
    let (_, listed, _) = run(a.path(), "laptop", &["fleet", "list"]);
    assert!(listed.contains("no other machines paired"), "{listed}");
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/fleet.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/fleet.rs"><storycode:paste select="#f-cccf3953dc0f-hick"/><storycode:paste select="#f-cccf3953dc0f-run"/><storycode:paste select="#f-cccf3953dc0f-two_machines_pair_by_copying_a_line_in_each_direction"/><storycode:paste select="#f-cccf3953dc0f-execute_is_off_until_it_is_given_deliberately"/><storycode:paste select="#f-cccf3953dc0f-a_phone_can_never_be_granted_execute"/><storycode:paste select="#f-cccf3953dc0f-revoking_a_machine_is_deleting_its_key"/><storycode:paste select="#f-cccf3953dc0f-the_key_lives_outside_every_repository"/><storycode:paste select="#f-cccf3953dc0f-a_damaged_invitation_is_refused_and_nothing_is_added"/></storycode:file>

<a id="output-ca2359248ac2"></a>
## `crates/hickory-fleet/tests/roster_guarded.rs`

Related contracts: [Pair identity without silently granting execution](contract.md).

<a id="f-ca2359248ac2-replaced_machine_keys_refuse_stale_grants_and_removal_without_writes"></a>
### `replaced_machine_keys_refuse_stale_grants_and_removal_without_writes`

<storycode:copy id="f-ca2359248ac2-replaced_machine_keys_refuse_stale_grants_and_removal_without_writes">
use hickory_fleet::{Fleet, FleetTargetChanged, Grant, Identity, Kind};

#[test]
fn replaced_machine_keys_refuse_stale_grants_and_removal_without_writes() {
    let root = tempfile::tempdir().unwrap();
    let original = Identity::load_or_create(&root.path().join("original"), "desktop").unwrap();
    let replacement =
        Identity::load_or_create(&root.path().join("replacement"), "desktop").unwrap();
    let fleet = Fleet::at(&root.path().join("roster"));
    fleet
        .add(&original.invitation(Kind::Desktop), "2026-10-20")
        .unwrap();
    fleet.remove("desktop").unwrap();
    fleet
        .add(&replacement.invitation(Kind::Desktop), "2026-10-20")
        .unwrap();
    let before = std::fs::read(fleet.path()).unwrap();
    let error = fleet
        .set_grant_if_key("desktop", Grant::Execute, true, &original.public_key())
        .unwrap_err();
    assert!(error.downcast_ref::<FleetTargetChanged>().is_some());
    let error = fleet
        .remove_if_key("desktop", &original.public_key())
        .unwrap_err();
    assert!(error.downcast_ref::<FleetTargetChanged>().is_some());
    assert_eq!(std::fs::read(fleet.path()).unwrap(), before);
    let current = fleet
        .set_grant_if_key("desktop", Grant::Execute, true, &replacement.public_key())
        .unwrap();
    assert!(current.grants.contains(&Grant::Execute));
    assert!(
        fleet
            .remove_if_key("desktop", &replacement.public_key())
            .unwrap()
    );
}

</storycode:copy>
<a id="f-ca2359248ac2-corrupt_roster_remains_read_only_instead_of_becoming_an_empty_write"></a>
### `corrupt_roster_remains_read_only_instead_of_becoming_an_empty_write`

<storycode:copy id="f-ca2359248ac2-corrupt_roster_remains_read_only_instead_of_becoming_an_empty_write">
#[test]
fn corrupt_roster_remains_read_only_instead_of_becoming_an_empty_write() {
    let root = tempfile::tempdir().unwrap();
    let fleet = Fleet::at(root.path());
    let corrupt = b"{unreadable retained roster";
    std::fs::write(fleet.path(), corrupt).unwrap();
    let identity = Identity::load_or_create(&root.path().join("identity"), "desktop").unwrap();
    assert!(fleet.machines().is_empty());
    assert!(
        fleet
            .add(&identity.invitation(Kind::Desktop), "2026-10-20")
            .is_err()
    );
    assert!(fleet.set_grant("desktop", Grant::Execute, true).is_err());
    assert!(fleet.remove("desktop").is_err());
    assert_eq!(std::fs::read(fleet.path()).unwrap(), corrupt);
}

</storycode:copy>
<a id="f-ca2359248ac2-fleet_writer_process"></a>
### `fleet_writer_process`

Spawned only by the focused parent case; all identities/keys live in its tempdir.

<storycode:copy id="f-ca2359248ac2-fleet_writer_process">
// Spawned only by the focused parent case; all identities/keys live in its tempdir.
#[test]
fn fleet_writer_process() {
    let Some(root) = std::env::var_os("HICKORY_FLEET_TEST_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let index = std::env::var("HICKORY_FLEET_TEST_INDEX").unwrap();
    let identity = Identity::load_or_create(
        &root.join(format!("identity-{index}")),
        &format!("writer-{index}"),
    )
    .unwrap();
    std::fs::write(root.join(format!("ready-{index}")), b"ready").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !root.join("go").exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(root.join("go").exists(), "Parent did not start writers");
    let fleet = Fleet::at(&root.join("roster"));
    for entry in 0..10 {
        let mut invitation = identity.invitation(Kind::Desktop);
        invitation.name = format!("writer-{index}-{entry}");
        fleet.add(&invitation, "2026-10-20").unwrap();
    }
}

</storycode:copy>
<a id="f-ca2359248ac2-separate_process_writers_preserve_every_roster_entry"></a>
### `separate_process_writers_preserve_every_roster_entry`

<storycode:copy id="f-ca2359248ac2-separate_process_writers_preserve_every_roster_entry">
#[test]
fn separate_process_writers_preserve_every_roster_entry() {
    let root = tempfile::tempdir().unwrap();
    let executable = std::env::current_exe().unwrap();
    let children: Vec<_> = (0..4)
        .map(|index| {
            std::process::Command::new(&executable)
                .args(["--exact", "fleet_writer_process"])
                .env("HICKORY_FLEET_TEST_ROOT", root.path())
                .env("HICKORY_FLEET_TEST_INDEX", index.to_string())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !(0..4).all(|index| root.path().join(format!("ready-{index}")).exists())
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!((0..4).all(|index| root.path().join(format!("ready-{index}")).exists()));
    std::fs::write(root.path().join("go"), b"go").unwrap();
    for child in children {
        let result = child.wait_with_output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let fleet = Fleet::at(&root.path().join("roster"));
    let names: std::collections::HashSet<_> = fleet
        .machines()
        .into_iter()
        .map(|machine| machine.name)
        .collect();
    assert_eq!(names.len(), 40);
    for writer in 0..4 {
        for entry in 0..10 {
            assert!(names.contains(&format!("writer-{writer}-{entry}")));
        }
    }
}
</storycode:copy>

### Assemble `crates/hickory-fleet/tests/roster_guarded.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-fleet/tests/roster_guarded.rs"><storycode:paste select="#f-ca2359248ac2-replaced_machine_keys_refuse_stale_grants_and_removal_without_writes"/><storycode:paste select="#f-ca2359248ac2-corrupt_roster_remains_read_only_instead_of_becoming_an_empty_write"/><storycode:paste select="#f-ca2359248ac2-fleet_writer_process"/><storycode:paste select="#f-ca2359248ac2-separate_process_writers_preserve_every_roster_entry"/></storycode:file>

## Implementation review

Fleet state stores identity, enrolled public keys and verb grants. The peer channel uses that identity when admitting a connection and checks the grants for each request.

### Implementation

- [apps/web/src/views/FleetPane.tsx](../../../mechanisms/apps/web/src/views/FleetPane.tsx.md#output-6e68b623335b)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [apps/web/src/views/workspaceState.ts](../../../mechanisms/apps/web/src/views/workspaceState.ts.md#output-efd800f9ed5a)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/serve/history.rs](../../../mechanisms/crates/hickory-cli/src/serve/history.rs.md#output-be949b7ea526)
- [crates/hickory-fleet/src/lib.rs](../../../mechanisms/crates/hickory-fleet/src/lib.rs.md#output-b1e920114639)
- [crates/hickory-peer/src/grants.rs](../../../mechanisms/crates/hickory-peer/src/grants.rs.md#output-fb44d9f1f9e0)

### Tests and fixtures

- [apps/web/src/views/FleetPane.test.tsx](../../../mechanisms/apps/web/src/views/FleetPane.tsx.md#output-ef08e5ce40d1)
- [apps/web/src/views/toolTabs.test.ts](contract.md#output-379c44af631b)
- [crates/hickory-cli/tests/fleet.rs](contract.md#output-cccf3953dc0f)
- [crates/hickory-fleet/tests/roster_guarded.rs](contract.md#output-ca2359248ac2)

### Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
