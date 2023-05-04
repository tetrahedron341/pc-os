#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

kernel::kernel_main!(kernel::test::TestMainBuilder::new(test_main).build());

#[test_case]
fn test_println() {
    kernel::println!("test_println output");
}
