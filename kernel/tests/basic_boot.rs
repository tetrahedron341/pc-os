#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

// #[no_mangle] // don't mangle the name of this function
// pub extern "C" fn _start() -> ! {
//     test_main();

//     loop {}
// }

#[test_case]
fn test_println() {
    kernel::println!("test_println output");
}
