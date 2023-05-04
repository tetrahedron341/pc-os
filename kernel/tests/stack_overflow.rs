#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

kernel::kernel_main!(kernel::test::TestMainBuilder::new(test_main)
    .should_panic()
    .build());

#[test_case]
fn overflow() {
    // kernel::serial_println!("[stack_overflow] begin");
    #[allow(unconditional_recursion)]
    fn stack_overflow() {
        let x = 0u32;
        unsafe {
            core::hint::black_box((&x as *const u32).read_volatile());
        }

        let rsp: u64;
        unsafe { core::arch::asm!("mov {}, rsp", out(reg) rsp) }
        kernel::serial_println!("[stack_overflow] rsp = 0x{:016x}", rsp);
        stack_overflow();
    }
    stack_overflow()
}
