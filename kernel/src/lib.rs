#![no_std]
#![no_main]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(asm_sym)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]
#![feature(const_maybe_uninit_uninit_array)]
#![feature(mixed_integer_ops)]
#![feature(ptr_metadata)]
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
