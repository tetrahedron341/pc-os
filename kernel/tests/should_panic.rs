#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

kernel::kernel_main!(kernel::test::TestMainBuilder::new(test_main)
    .should_panic()
    .build());

#[test_case]
fn should_panic() {
    assert_eq!(1, 2)
}
