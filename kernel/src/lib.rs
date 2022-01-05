#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(asm_sym)]
#![test_runner(crate::test::test_runner)]
#![allow(clippy::new_without_default)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod acpi;
pub mod allocator;
pub mod arch;
pub mod boot;
pub mod file;
pub mod init;
pub mod log;
pub mod memory;
#[cfg(not(feature = "custom_panic"))]
mod panic;
pub mod process;
pub mod serial;
pub mod syscall;
pub mod task;
pub mod test;
pub mod video;

#[allow(clippy::all)]
pub mod uapi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
