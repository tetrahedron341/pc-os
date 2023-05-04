use core::arch::asm;

use super::Process;
use super::Registers;
use alloc::collections::VecDeque;
use x86_64::structures::paging::{Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

pub struct Executor {
    procs: VecDeque<Process>,
    process_start: VirtAddr,
    paging_service: crate::memory::PagingService,
}

impl Executor {
    pub fn new(
        init_proc: Process,
        process_start: VirtAddr,
        paging_service: crate::memory::PagingService,
    ) -> Self {
        let mut s = Executor {
            procs: {
                let mut v = VecDeque::new();
                v.push_front(init_proc);
                v
            },
            process_start,
            paging_service,
        };
        s.map_current_process();
        s
    }

    fn current_process(&self) -> &Process {
        self.procs.front().unwrap()
    }

    fn current_process_mut(&mut self) -> &mut Process {
        self.procs.front_mut().unwrap()
    }

    /// Loads the next process
    fn next_process(&mut self) {
        self.unmap_current_process();
        self.procs.rotate_left(1);
        self.map_current_process();
    }

    fn unmap_current_process(&mut self) {
        for p in 0..self.current_process().code_len {
            let page: Page<Size4KiB> =
                Page::containing_address(self.process_start + p as u64 * 4096);
            self.paging_service.mapper.unmap(page).unwrap().1.flush();
        }
    }

    fn map_current_process(&mut self) {
        // Map the code frames
        for (p, frame) in self.procs.front().unwrap().frames.iter().enumerate() {
            let page: Page<Size4KiB> =
                Page::containing_address(self.process_start + p as u64 * 4096);
            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE;
            unsafe {
                self.paging_service
                    .mapper
                    .map_to(
                        page,
                        *frame,
                        flags,
                        &mut self.paging_service.frame_allocator,
                    )
                    .unwrap()
                    .flush();
            }
        }
        // Map the stack frames
        for (p, frame) in self.procs.front().unwrap().stack_frames.iter().enumerate() {
            let page: Page<Size4KiB> = Page::containing_address(super::STACK_TOP - p as u64 * 4096);
            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE;
            unsafe {
                self.paging_service
                    .mapper
                    .map_to(
                        page,
                        *frame,
                        flags,
                        &mut self.paging_service.frame_allocator,
                    )
                    .unwrap()
                    .flush();
            }
        }
    }

    /// Exits the current process, and prepares the next process in the scheduler.
    pub fn exit_current_process(&mut self) {
        self.unmap_current_process();
        self.procs.pop_front();
        if self.procs.is_empty() {
            panic!("Process queue emptied. TODO: Graceful shutdown.")
        }
        self.map_current_process();
    }

    /// Suspends the current process and all of its state,
    pub fn suspend_current_process(&mut self, registers: Registers) {
        self.current_process_mut().registers = registers;
        self.next_process();
    }

    // unsafe fn context_switch(&mut self, ip: usize, rsp: usize) -> ! {
    //     let mut ss = crate::arch::gdt::GDT.1.user_data_selector;
    //     ss.set_rpl(x86_64::PrivilegeLevel::Ring3);
    //     let mut cs = crate::arch::gdt::GDT.1.user_code_selector;
    //     cs.set_rpl(x86_64::PrivilegeLevel::Ring3);
    //     let ss = ss.0 as u64;
    //     let cs = cs.0 as u64;
    //     let rip = ip as u64;
    //     asm!(
    //         "push {ss}",
    //         "push {rsp}",
    //         "pushf",
    //         "push {cs}",
    //         "push {rip}",
    //         "iretq",
    //         ss = in(reg) ss,
    //         rsp = in(reg) rsp,
    //         cs = in(reg) cs,
    //         rip = in(reg) rip,
    //     );

    //     core::hint::unreachable_unchecked()
    // }

    // Enter user mode. Jumps into the currently running process in user mode. *This call will not return to the caller.*
    // pub fn run(&mut self) -> ! {
    //     let Process {
    //         registers: Registers { rip, rsp, .. },
    //         ..
    //     } = *self.current_process();
    //     unsafe {
    //         self.context_switch(rip, rsp);
    //     }
    // }
}
