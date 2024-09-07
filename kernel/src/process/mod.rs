use core::{task::{Poll, Waker}, num::NonZeroU64};

use alloc::vec::Vec;

use crate::arch::cpu::this_cpu;

mod exec;
pub mod space;

pub use exec::create_process_from_elf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(NonZeroU64);

impl ProcessId {
    /// Generates a new, guaranteed unique PID
    pub fn new_unique() -> Self {
        static COUNTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(1);
        let pid = COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        let Ok(pid) = NonZeroU64::try_from(pid) else { panic!("PID overflow") };
        ProcessId(pid)
    }

    pub fn as_u64(&self) -> u64 {
        self.0.get()
    }
}

impl From<ProcessId> for u64 {
    fn from(value: ProcessId) -> Self {
        value.as_u64()
    }
}

impl From<ProcessId> for NonZeroU64 {
    fn from(value: ProcessId) -> Self {
        value.0
    }
}

pub enum ProcessState {
    Running(Waker),
    Runnable,
    Killed,
    Waiting,
}

pub struct Process {
    pub pid: ProcessId,
    pub kernel_stack: Vec<u8>,
    pub state: ProcessState,
    pub space: crate::arch::memory::space::Space,
    pub context: *mut crate::arch::cpu::Context,
}

unsafe impl Send for Process {}

impl core::future::Future for Process {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        x86_64::instructions::interrupts::disable();
        let cpu = this_cpu();
        let p = self.get_mut();
        cpu.run_process(p, cx.waker().clone());

        match p.state {
            ProcessState::Running(_) => panic!("Yielded process in `Running` state!"),
            ProcessState::Runnable => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            ProcessState::Waiting => Poll::Pending,
            ProcessState::Killed => Poll::Ready(()),
        }
    }
}
