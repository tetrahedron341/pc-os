pub mod cpu;
pub mod gdt;
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

mod boot {
    #[cfg(feature = "phil_opp_bootloader")]
    mod phil_opp_bootloader;

    #[cfg(feature = "limine_bootloader")]
    mod limine;
}
