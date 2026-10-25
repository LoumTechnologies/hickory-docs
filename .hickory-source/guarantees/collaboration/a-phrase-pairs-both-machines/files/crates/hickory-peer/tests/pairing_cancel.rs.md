<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-peer/tests/pairing_cancel.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [One Spoken Phrase Enrols Both Machines, With No Server And Nobody Trusted](../../../../contract.md)

Captured SHA-256: `5d7c83abf18b123a7ffd6463b0e42871aec10904616f7355bd75fea0ec597a98`.

<book:entry path="crates/hickory-peer/tests/pairing_cancel.rs">
use hickory_fleet::{Fleet, Identity, Kind};
use hickory_peer::pairing::{PairCancelled, PairPhase, host_cancellable, join_cancellable};
use hickory_peer::{Phrase, Reach};
use std::time::Duration;

#[tokio::test]
async fn cancelling_a_bound_pairing_host_closes_it_before_local_enrollment() {
    let root = tempfile::tempdir().unwrap();
    let identity = Identity::load_or_create(root.path(), "private-cancel-fixture").unwrap();
    let roster = Fleet::at(identity.dir());
    let roster_path = identity.dir().to_path_buf();
    let phrase = Phrase::generate();
    let (cancel, receiver) = tokio::sync::watch::channel(false);
    let (progress, mut observed) = tokio::sync::watch::channel(PairPhase::Preparing);
    let task = tokio::spawn(async move {
        host_cancellable(
            &identity,
            &roster,
            &phrase,
            Kind::Desktop,
            &Reach::Direct,
            "2026-10-20",
            receiver,
            Some(progress),
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if *observed.borrow() == PairPhase::WaitingForPeer {
                break;
            }
            observed
                .changed()
                .await
                .expect("Host exited before binding its local fixture endpoint");
        }
    })
    .await
    .expect("Host did not bind in time");
    cancel.send(true).unwrap();
    let error = tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert!(error.downcast_ref::<PairCancelled>().is_some());
    assert_eq!(*observed.borrow(), PairPhase::Closed);
    assert!(Fleet::at(&roster_path).machines().is_empty());
}

#[tokio::test]
async fn an_already_cancelled_dial_never_enrolls_or_binds() {
    let root = tempfile::tempdir().unwrap();
    let identity = Identity::load_or_create(root.path(), "private-dial-fixture").unwrap();
    let roster = Fleet::at(identity.dir());
    let (_cancel, receiver) = tokio::sync::watch::channel(true);
    let (progress, observed) = tokio::sync::watch::channel(PairPhase::Preparing);
    let error = join_cancellable(
        &identity,
        &roster,
        &Phrase::generate(),
        Kind::Desktop,
        &Reach::Direct,
        "2026-10-20",
        receiver,
        Some(progress),
    )
    .await
    .unwrap_err();
    assert!(error.downcast_ref::<PairCancelled>().is_some());
    assert_eq!(*observed.borrow(), PairPhase::Preparing);
    assert!(roster.machines().is_empty());
}
</book:entry>

</book:doc>
