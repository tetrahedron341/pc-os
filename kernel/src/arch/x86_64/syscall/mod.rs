use core::arch::asm;

use crate::syscall::{SyscallOp, SyscallStatus};

static mut STACK: [u8; 32 * 1024] = [0; 32 * 1024];
static mut SYSCALL_RSP: *const u8 = unsafe { &STACK[STACK.len() - 8] as *const u8 };
static mut RETURN_RSP: *const u8 = core::ptr::null();

#[naked]
unsafe extern "C" fn _syscall_handler() {
    asm! {
        "mov [{ret_rsp} + rip], rsp",
        "mov rsp, [{sys_rsp} + rip]",
        "push rcx",
        "push r11",
        "call syscall_handler",
        "pop r11",
        "pop rcx",
        "mov [{sys_rsp} + rip], rsp",
        "mov rsp, [{ret_rsp} + rip]",
        "sysretq",
        sys_rsp = sym SYSCALL_RSP,
        ret_rsp = sym RETURN_RSP,
        options(noreturn)
    }
}

pub fn init() {
    use x86_64::{
        registers::model_specific::{Efer, EferFlags, LStar, Star},
        VirtAddr,
    };
    // Enable the SYSCALL/SYSRET instructions
    unsafe {
        Efer::update(|f| *f |= EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
    // Load the syscall function pointer into IA32_LSTAR
    LStar::write(VirtAddr::new(_syscall_handler as *const () as usize as u64));

    use super::gdt::SELECTORS;
    let kernel_cs = SELECTORS.code_selector;
    let kernel_ss = SELECTORS.data_selector;
    let user_cs = SELECTORS.user_code_selector;
    let user_ss = SELECTORS.user_data_selector;

    // Load the appropriate segment selectors to IA32_STAR
    Star::write(user_cs, user_ss, kernel_cs, kernel_ss).unwrap();
}

#[no_mangle]
extern "C" fn syscall_handler() {
    let (op, args) = unsafe {
        let op: u64;
        let [arg0, arg1, arg2, arg3]: [u64; 4];
        asm!(
            "mov {op}, rax",
            "mov {arg0}, rdi",
            "mov {arg1}, rsi",
            "mov {arg2}, rdx",
            "mov {arg3}, r8",
            op = out(reg) op,
            arg0 = out(reg) arg0,
            arg1 = out(reg) arg1,
            arg2 = out(reg) arg2,
            arg3 = out(reg) arg3,
        );
        (op, [arg0, arg1, arg2, arg3])
    };

    let r: u64 = {
        let op = u32::try_from(op).ok();
        if let Some(op) = op.and_then(|op| SyscallOp::new(op as u32, args)) {
            crate::syscall::syscall_dispatch(op)
        } else {
            SyscallStatus::InvalidOp
        }
    }
    .into();
    unsafe {
        asm!(
            "mov rax, {r}",
            r = in(reg) r
        )
    }
}
