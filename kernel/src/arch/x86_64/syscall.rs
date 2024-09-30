use core::{arch::asm, ptr::addr_of};

use crate::util::Align16;
use kernel_uapi::syscall::{Syscall, SyscallResult};

const STACK_LEN: usize = 32 * 1024;
static mut STACK: Align16<[u8; STACK_LEN]> = Align16([0; STACK_LEN]);
static mut SYSCALL_RSP: *const u8 = unsafe { addr_of!(STACK.0[0]).add(STACK_LEN) };
static mut RETURN_RSP: *const u8 = core::ptr::null();

#[naked]
unsafe extern "C" fn _syscall_handler() {
    extern "C" fn syscall_handler(syscall: &mut Syscall, out: *mut SyscallResult) {
        unsafe { *out = crate::syscall::syscall_handler(syscall) }
    }

    asm! {
        "mov [{ret_rsp} + rip], rsp",
        "mov rsp, [{sys_rsp} + rip]",
        "push rcx",
        "push r11",
        "call {syscall_handler}",
        "pop r11",
        "pop rcx",
        "mov [{sys_rsp} + rip], rsp",
        "mov rsp, [{ret_rsp} + rip]",
        "sysretq",
        sys_rsp = sym SYSCALL_RSP,
        ret_rsp = sym RETURN_RSP,
        syscall_handler = sym syscall_handler,
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
