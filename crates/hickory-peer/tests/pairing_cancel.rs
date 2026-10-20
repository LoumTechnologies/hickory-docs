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
