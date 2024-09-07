use core::arch::asm;

use super::Registers;

/// The full context of a process, stored at the bottom of the stack of its process.
#[derive(Debug)]
#[repr(C)]
pub struct Context {
    pub registers: Registers,
    pub rip: u64,
}

impl Context {
    /// Save the current CPU state and perform a context switch.
    /// # Safety
    /// The target context must be valid and not cause any UB.
    pub unsafe extern "C" fn switch(save: &mut *mut Context, load: *mut Context) {
        let save = save as *mut *mut Context;
        asm! {
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rsi",
            "push rdi",
            "push rbp",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            "mov [{save}], rsp",

            "mov rsp, {load}",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rbp",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            "ret",

            save = in(reg) save,
            load = in(reg) load,
        }
    }
}
