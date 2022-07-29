use core::task::{Poll, Waker};

use alloc::vec::Vec;

use crate::arch::cpu::this_cpu;

mod exec;
pub mod space;

pub use exec::create_process_from_elf;

pub enum ProcessState {
    Running(Waker),
    Runnable,
    Killed,
    Waiting,
}

pub struct Process {
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
