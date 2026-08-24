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

fn run(state: &Path, machine: &str, args: &[&str]) -> (bool, String, String) {
    let out = hick(state, machine).args(args).output().expect("run hick");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

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
