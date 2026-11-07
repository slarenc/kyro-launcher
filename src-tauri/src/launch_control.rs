use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc, Mutex,
};
use tokio::sync::Notify;

pub const CANCELLED: &str = "Запуск отменён";
const PREPARING: u8 = 0;
const CANCELLING: u8 = 1;
const RUNNING: u8 = 2;
const FINISHED: u8 = 3;

#[derive(Default)]
pub struct LaunchControl {
    current: Mutex<Option<Arc<LaunchAttempt>>>,
}
pub struct LaunchAttempt {
    phase: AtomicU8,
    cancel: Notify,
}
impl LaunchControl {
    pub fn begin(&self) -> Arc<LaunchAttempt> {
        let attempt = Arc::new(LaunchAttempt {
            phase: AtomicU8::new(PREPARING),
            cancel: Notify::new(),
        });
        *self.current.lock().unwrap() = Some(attempt.clone());
        attempt
    }
    pub fn can_cancel(&self) -> bool {
        self.phase() == Some(PREPARING)
    }
    pub fn is_cancelling(&self) -> bool {
        self.phase() == Some(CANCELLING)
    }
    pub fn is_running(&self) -> bool {
        self.phase() == Some(RUNNING)
    }
    fn phase(&self) -> Option<u8> {
        self.current
            .lock()
            .ok()?
            .as_ref()
            .map(|a| a.phase.load(Ordering::SeqCst))
    }
    pub fn cancel(&self) -> bool {
        let Ok(current) = self.current.lock() else {
            return false;
        };
        let Some(attempt) = current.as_ref() else {
            return false;
        };
        if attempt
            .phase
            .compare_exchange(PREPARING, CANCELLING, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            attempt.cancel.notify_one();
            true
        } else {
            false
        }
    }
}
impl LaunchAttempt {
    pub async fn prepare<T>(
        &self,
        future: impl std::future::Future<Output = Result<T, String>>,
    ) -> Result<T, String> {
        let cancelled = self.cancel.notified();
        if self.phase.load(Ordering::SeqCst) == CANCELLING {
            return Err(CANCELLED.into());
        }
        tokio::select! { biased;
            _ = cancelled => Err(CANCELLED.into()),
            result = future => result,
        }
    }
    /// Cancellation and committing the Java spawn are mutually exclusive.
    pub fn start_game(&self) -> Result<(), String> {
        self.phase
            .compare_exchange(PREPARING, RUNNING, Ordering::SeqCst, Ordering::SeqCst)
            .map(|_| ())
            .map_err(|_| CANCELLED.into())
    }
    pub fn finish(&self) {
        self.phase.store(FINISHED, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn cancellation_drops_download_and_next_attempt_can_start() {
        let control = LaunchControl::default();
        let attempt = control.begin();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        struct DropFlag(Arc<std::sync::atomic::AtomicBool>);
        impl Drop for DropFlag {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let flag = dropped.clone();
        let task_attempt = attempt.clone();
        let task = tokio::spawn(async move {
            task_attempt
                .prepare(async move {
                    let _guard = DropFlag(flag);
                    let _ = started_tx.send(());
                    std::future::pending::<Result<(), String>>().await
                })
                .await
        });
        started_rx.await.unwrap();
        assert!(control.cancel());
        assert_eq!(task.await.unwrap().unwrap_err(), CANCELLED);
        assert!(dropped.load(Ordering::SeqCst));
        assert!(attempt.start_game().is_err());
        attempt.finish();
        assert!(!control.can_cancel());
        let next = control.begin();
        assert!(next.prepare(async { Ok(()) }).await.is_ok());
        next.start_game().unwrap();
        assert!(!control.cancel());
        assert!(control.is_running());
        next.finish();
    }
    #[tokio::test]
    async fn cancel_before_poll_or_after_prepare_never_spawns_game() {
        let control = LaunchControl::default();
        let attempt = control.begin();
        assert!(control.cancel());
        assert_eq!(
            attempt.prepare(async { Ok(()) }).await.unwrap_err(),
            CANCELLED
        );
        attempt.finish();
        let next = control.begin();
        next.prepare(async { Ok(()) }).await.unwrap();
        assert!(control.cancel());
        assert!(next.start_game().is_err());
        next.finish();
        assert!(!control.cancel());
    }
}
