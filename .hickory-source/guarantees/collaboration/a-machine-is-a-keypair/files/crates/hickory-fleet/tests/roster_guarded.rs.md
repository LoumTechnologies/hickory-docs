<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-fleet/tests/roster_guarded.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Pair identity without silently granting execution](../../../../contract.md)

Captured SHA-256: `dce4b79ee8e0fa0d7f4522e0a8cba43a74c2910165686a6ddc57ec8b1bb8c62c`.

<book:entry path="crates/hickory-fleet/tests/roster_guarded.rs">
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
</book:entry>

</book:doc>
