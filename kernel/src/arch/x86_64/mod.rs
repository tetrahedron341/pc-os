pub mod cpu;
pub mod gdt;
mod init;
pub mod interrupts;
pub mod memory;
mod syscall;

pub fn loop_forever() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}

mod boot {
    #[cfg(feature = "limine_bootloader")]
    mod limine;
}
