#![no_std]
#![no_main]

use core::panic::PanicInfo;
use pc_os::{serial_print, serial_println, exit_qemu, QemuExitCode};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    should_panic();
    serial_println!("[did not panic]");
    exit_qemu(QemuExitCode::Failure);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
}

fn should_panic() {
    serial_print!("{}::should_panic...\t", module_path!());
    assert_eq!(1,2)
}