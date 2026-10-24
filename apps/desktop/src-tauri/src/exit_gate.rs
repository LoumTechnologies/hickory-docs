//! An acknowledged detach, rather than a scheduled Drop, orders native exit.
use std::sync::atomic::{AtomicU8, Ordering};
#[derive(Default)]
pub(super) struct ExitGate(AtomicU8);
impl ExitGate {
    pub(super) fn ready(&self) -> bool {
        self.0.load(Ordering::SeqCst) == 2
    }
    pub(super) async fn prepare(
        &self,
        detach: impl std::future::Future<Output = anyhow::Result<()>>,
    ) -> anyhow::Result<bool> {
        if self
            .0
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Ok(false);
        }
        match detach.await {
            Ok(()) => {
                self.0.store(2, Ordering::SeqCst);
                Ok(true)
            }
            Err(error) => {
                self.0.store(0, Ordering::SeqCst);
                Err(error)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn exit_waits_for_ack_and_reentrant_requests_do_not_repeat_detach() {
        let gate = std::sync::Arc::new(ExitGate::default());
        let (ack, pending) = tokio::sync::oneshot::channel::<()>();
        let worker = gate.clone();
        let task = tokio::spawn(async move {
            worker
                .prepare(async {
                    pending.await.unwrap();
                    Ok(())
                })
                .await
        });
        tokio::task::yield_now().await;
        assert!(!gate.ready());
        assert!(
            !gate
                .prepare(async { panic!("duplicate detach must not run") })
                .await
                .unwrap()
        );
        ack.send(()).unwrap();
        assert!(task.await.unwrap().unwrap());
        assert!(gate.ready());
        assert!(
            !gate
                .prepare(async { panic!("already detached") })
                .await
                .unwrap()
        );
    }
    #[tokio::test]
    async fn failure_refuses_exit_and_allows_a_fresh_acknowledged_retry() {
        let gate = ExitGate::default();
        assert!(
            gate.prepare(async { anyhow::bail!("detach not acknowledged") })
                .await
                .is_err()
        );
        assert!(!gate.ready());
        assert!(gate.prepare(async { Ok(()) }).await.unwrap());
        assert!(gate.ready());
    }
}
