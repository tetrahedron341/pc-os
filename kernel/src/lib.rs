#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(asm)]
#![feature(global_asm)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::new_without_default)]

extern crate alloc;

pub mod acpi;
pub mod allocator;
pub mod file;
pub mod gdt;
pub mod init;
pub mod interrupts;
pub mod log;
pub mod memory;
pub mod process;
pub mod serial;
pub mod syscall;
pub mod task;
pub mod video;

use core::panic::PanicInfo;

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    unreachable!("Should have already exited. Are you not using QEMU?")
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);
}

#[cfg(test)]
bootloader::entry_point!(test_entry_point);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_entry_point(boot_info: &'static mut bootloader::BootInfo) -> ! {
    init::init(boot_info);
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}