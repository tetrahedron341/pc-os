use alloc::boxed::Box;
use conquer_once::spin::OnceCell;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use spin::Mutex;

use executor::Executor;

use self::executor::TaskWaker;

pub mod executor;
pub mod keyboard;
pub mod simple_executor;
pub mod timer;

pub static EXECUTOR: OnceCell<Mutex<Executor>> = OnceCell::uninit();

pub fn init_executor() {
    EXECUTOR.init_once(|| Mutex::new(Executor::new()))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(cx)
    }
}

fn run_ready_tasks() {
    let exec = EXECUTOR.get().unwrap();

    let pop_task = || {
        let mut exec = exec.lock();

        let t = loop {
            if let Ok(task_id) = exec.task_queue.pop() {
                if exec.tasks.contains_key(&task_id) {
                    break Some(task_id);
                } else {
                    continue; // The task no longer exists for whatever reason
                };
            } else {
                break None;
            }
        };

        if let Some(task_id) = t {
            let task_queue = exec.task_queue.clone();
            let waker = exec
                .waker_cache
                .entry(task_id)
                .or_insert_with(move || TaskWaker::new_as_waker(task_id, task_queue))
                .clone();
            let task = exec.tasks.remove(&task_id).unwrap();

            Some((task_id, task, waker))
        } else {
            None
        }
    };

    while let Some((task_id, mut task, waker)) = pop_task() {
        log::trace!("Running task {task_id:?}");
        let mut cx = Context::from_waker(&waker);
        match task.poll(&mut cx) {
            Poll::Ready(()) => {
                let mut exec = exec.lock();
                exec.tasks.remove(&task_id);
                exec.waker_cache.remove(&task_id);
            }
            Poll::Pending => {
                // Put that thing back where it came from, or so help me!
                let mut exec = exec.lock();
                exec.tasks.insert(task_id, task);
            }
        }
    }
}

/// Let the executor take control of this CPU core.
pub fn run() -> ! {
    loop {
        run_ready_tasks();
        sleep_if_idle();
    }
}

fn sleep_if_idle() {
    x86_64::instructions::interrupts::disable();
    let queue_is_empty = {
        let exec = EXECUTOR.get().unwrap().lock();
        exec.task_queue.is_empty()
    };
    if queue_is_empty {
        x86_64::instructions::interrupts::enable_and_hlt();
    } else {
        x86_64::instructions::interrupts::enable();
    }
}
