use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::task::AtomicWaker;

use crate::{clear_timer, set_timer, TimerId};

/// A future representing the notification that an elapsed duration has
/// occurred.
///
/// This is created through the `Delay::new` method indicating when the future should fire.
/// Note that these futures are not intended for high resolution timers.
pub struct Delay {
    timer_id: Option<TimerId>,
    waker: Arc<AtomicWaker>,
    at: Duration,
}

impl Delay {
    /// Creates a new future which will fire at `dur` time into the future.
    pub fn new(dur: Duration) -> Delay {
        let now = duration_since_epoch();

        Delay {
            timer_id: None,
            waker: Arc::new(AtomicWaker::new()),
            at: now + dur,
        }
    }

    /// Resets this timeout to an new timeout which will fire at the time
    /// specified by `at`.
    pub fn reset(&mut self, dur: Duration) {
        let now = duration_since_epoch();
        self.at = now + dur;

        if let Some(id) = self.timer_id.take() {
            clear_timer(id);
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = duration_since_epoch();

        if now >= self.at {
            Poll::Ready(())
        } else {
            // Register the latest waker
            self.waker.register(cx.waker());

            // Register to global timer
            if self.timer_id.is_none() {
                let waker = self.waker.clone();
                let id = set_timer(self.at - now, move || waker.wake());
                self.timer_id = Some(id);
            }

            Poll::Pending
        }
    }
}

impl Drop for Delay {
    fn drop(&mut self) {
        if let Some(id) = self.timer_id.take() {
            clear_timer(id);
        }
    }
}

impl fmt::Debug for Delay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Delay").finish()
    }
}

fn duration_since_epoch() -> Duration {
    Duration::from_nanos(ic_cdk::api::time())
}
