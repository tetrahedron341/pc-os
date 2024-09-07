use core::arch::asm;

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
    let (op, ptr) = unsafe {
        let r14: u64;
        let r15: u64;
        asm!(
            "mov {r14}, r14",
            "mov {r15}, r15",
            r14 = out(reg) r14,
            r15 = out(reg) r15,
        );
        (r14, r15 as usize as *mut u8)
    };

    let r: u64 = crate::syscall::syscall_dispatch(op, ptr).into();
    unsafe {
        asm!(
            "mov r14, {r}",
            r = in(reg) r
        )
    }
}
