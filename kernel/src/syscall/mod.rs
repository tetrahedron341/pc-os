mod syscalls;

pub use syscalls::*;

static mut STACK: [u8; 32 * 1024] = [0; 32 * 1024];
#[no_mangle]
static mut SYSCALL_RSP: *const u8 = unsafe { &STACK[STACK.len() - 8] as *const u8 };
#[no_mangle]
static mut RETURN_RSP: *const u8 = core::ptr::null();

global_asm!(include_str!("syscall.s"));

extern "C" {
    fn _syscall_handler();
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

    use crate::gdt::GDT;
    let kernel_cs = GDT.1.code_selector;
    let kernel_ss = GDT.1.data_selector;
    let user_cs = GDT.1.user_code_selector;
    let user_ss = GDT.1.user_data_selector;

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

    let r = syscalls::syscall_dispatch(op, ptr);
    unsafe {
        asm!(
            "mov r14, {r}",
            r = in(reg) r
        )
    }
}
