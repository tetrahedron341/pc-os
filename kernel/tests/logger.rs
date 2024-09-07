#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    use log::LevelFilter;

    kernel::init::init(boot_info);

    kernel::log::init(LevelFilter::Off, LevelFilter::Info, 64);

    // Clear the screen
    kernel::print!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");

    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info);
}

// #[test_case]
// fn logging() {
//     const TARGET: &[u8] = b"[INFO] logger - Test `logging`";

//     log::info!("Test `logging`");
//     kernel::log::flush();

//     let vga = kernel::vga::WRITER.lock();
//     let actual = {
//         let mut buf = [0u8; TARGET.len()];
//         let slice = &vga.buffer.chars[23][0..TARGET.len() * 2];
//         for i in 0..TARGET.len() {
//             buf[i] = slice[i].read().char;
//         }
//         buf
//     };

//     assert_eq!(
//         *TARGET,
//         actual,
//         "{:?} != {:?}",
//         core::str::from_utf8(TARGET),
//         core::str::from_utf8(&actual)
//     );
// }
