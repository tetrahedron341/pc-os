//! Entry point for Philipp Oppermann's bootloader crate

use bootloader::BootInfo;

bootloader::entry_point!(kernel_entry);

fn kernel_entry(boot_info: &'static mut BootInfo) -> ! {
    let _ = super::interrupts::init_idt();
    let _ = super::gdt::init();
    let mut paging_service = unsafe {
        crate::memory::init(
            boot_info.recursive_index.into_option().unwrap(),
            &boot_info.memory_regions,
        )
    };
    crate::allocator::init_heap(
        &mut paging_service.mapper,
        &mut paging_service.frame_allocator,
    )
    .unwrap();

    x86_64::instructions::interrupts::enable();
    x86_64::instructions::interrupts::int3();
    super::loop_forever();
}
