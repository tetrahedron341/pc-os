use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::Waker;
use crossbeam_queue::ArrayQueue;

pub struct Executor {
    pub(super) tasks: BTreeMap<TaskId, Task>,
    pub(super) task_queue: Arc<ArrayQueue<TaskId>>,
    pub(super) waker_cache: BTreeMap<TaskId, Waker>,
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
}

pub(super) struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    pub fn new_as_waker(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    pub fn wake_task(&self) {
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
