#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]
#![feature(const_maybe_uninit_uninit_array)]
#![feature(ptr_metadata)]
#![feature(never_type)]
#![feature(int_roundings)]
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
mod panic;
pub mod process;
pub mod serial;
pub mod syscall;
pub mod task;
pub mod test;
pub mod video;

#[cfg(test)]
crate::kernel_main!(test::TestMainBuilder::new(test_main).build());
