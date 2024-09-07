//! Entry point for Philipp Oppermann's bootloader crate

use bootloader::BootInfo;

use crate::boot::BootModule;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        bootloader::entry_point!(kernel_entry);

        fn kernel_entry(boot_info: &'static mut BootInfo) -> ! {
            crate::init::kernel_main(initialize(boot_info));
        }
    } else {
        bootloader::entry_point!(test_entry_point);

        /// Entry point for `cargo test`
        fn test_entry_point(boot_info: &'static mut bootloader::BootInfo) -> ! {
            initialize(boot_info);
            crate::test_main();
            crate::arch::loop_forever();
        }
    }
}

fn initialize(boot_info: &'static mut bootloader::BootInfo) -> crate::init::InitServices {
    super::interrupts::init_idt();
    super::gdt::init();
    unsafe {
        super::memory::init(
            boot_info.recursive_index.into_option().unwrap(),
            &boot_info.memory_regions,
        )
    };
    crate::allocator::init_heap().unwrap();
    super::syscall::init();

    let modules = boot_info
        .modules
        .iter()
        .map(|m| unsafe { load_module(*m) })
        .collect();

    unsafe {
        crate::video::vesa::init_screen(boot_info.framebuffer.as_mut().unwrap());
    }
    let console =
        crate::video::vesa::console::Console::new(crate::video::vesa::SCREEN.get().unwrap());
    crate::video::console::CONSOLE.lock().replace(console);

    x86_64::instructions::interrupts::enable();

    const SERIAL_LOG_MIN: log::LevelFilter = log::LevelFilter::Info;
    const CONSOLE_LOG_MIN: log::LevelFilter = log::LevelFilter::Warn;

    crate::log::init(SERIAL_LOG_MIN, CONSOLE_LOG_MIN, 128);

    crate::init::InitServices { modules }
}

unsafe fn load_module(module_desc: bootloader::boot_info::Module) -> BootModule {
    let ptr =
        super::memory::phys_to_virt(x86_64::PhysAddr::new(module_desc.phys_addr)).as_mut_ptr();
    BootModule {
        name: core::str::from_utf8(&module_desc.name)
            .unwrap()
            .trim_end_matches('\0')
            .into(),
        data: core::slice::from_raw_parts_mut(ptr, module_desc.len),
    }
}
