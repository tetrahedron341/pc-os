use x86_64::{structures::idt::InterruptStackFrameValue, VirtAddr};

/// Placed on the stack by the interrupt handler functions.
/// Used for storing/restoring CPU state during an interrupt.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Registers {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    pub error_code: u64,
    pub isf: InterruptStackFrameValue,
}

impl Registers {
    /// # Safety
    /// Must never be returned from an interrupt handler using `save_regs!`
    pub unsafe fn empty() -> Self {
        Registers {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rbp: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,

            error_code: 0,
            isf: InterruptStackFrameValue {
                instruction_pointer: VirtAddr::zero(),
                code_segment: 0,
                cpu_flags: 0,
                stack_pointer: VirtAddr::zero(),
                stack_segment: 0,
            },
        }
    }
}
