use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, f: impl core::future::Future<Output = ()> + Send + 'static) {
        let task = Task::new(f);
        let id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("Tried to spawn two tasks with same ID")
        }
        self.task_queue.push(id).expect("queue full");
    }

    pub fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new_as_waker(task_id, task_queue.clone()));
            let mut cx = Context::from_waker(waker);
            match task.poll(&mut cx) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    /// Let the executor take control of this CPU core.
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        x86_64::instructions::interrupts::disable();
        if self.task_queue.is_empty() {
            x86_64::instructions::interrupts::enable_and_hlt();
        } else {
            x86_64::instructions::interrupts::enable();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new_as_waker(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task queue full");
    }
}

use alloc::task::Wake;

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
