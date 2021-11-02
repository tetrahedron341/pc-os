use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};
use spin::Mutex;

static TICKS: AtomicU64 = AtomicU64::new(0);
/// A workaround to create a static array of non-Copy `None`s
const _NONE_WAKER: Option<(Waker, u64)> = None;
static WAKERS: Mutex<[Option<(Waker, u64)>; 128]> = Mutex::new([_NONE_WAKER; 128]);

pub(crate) fn tick_timer() {
    let t = TICKS.fetch_add(1, Ordering::Relaxed);
    for entry in WAKERS.lock().iter_mut().filter(|w| w.is_some()) {
        let (waker, target) = entry.take().unwrap();
        if t >= target {
            waker.wake();
        } else {
            entry.replace((waker, target));
        }
    }
}

pub fn wait_n_ticks(n: u64) -> impl Future<Output = ()> {
    TimerWaiter(TICKS.load(Ordering::Relaxed) + n)
}

struct TimerWaiter(u64);

impl Future for TimerWaiter {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if TICKS.load(Ordering::Relaxed) >= self.0 {
            Poll::Ready(())
        } else {
            WAKERS
                .lock()
                .iter_mut()
                .find(|w| w.is_none())
                .expect("Out of timer slots")
                .replace((cx.waker().clone(), self.0));
            Poll::Pending
        }
    }
}
