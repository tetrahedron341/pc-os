#[cfg_attr(feature = "phil_opp_bootloader", path = "boot/bootloader.rs")]
mod boot;
pub mod context;
mod gdt;
mod init;
#[path = "interrupts/mod.rs"]
mod interrupts;
pub mod memory;
mod syscall;

pub fn loop_forever() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}
