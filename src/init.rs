use crate::gdt;
use crate::interrupts;
use crate::memory;
use crate::video;

pub fn init(boot_info: &'static mut bootloader::BootInfo) -> InitServices {
    let idt_service = interrupts::init_idt();
    let gdt_service = gdt::init();
    let mut paging_service = unsafe {
        memory::init(
            boot_info.recursive_index.into_option().unwrap(),
            &boot_info.memory_regions,
        )
    };
    crate::allocator::init_heap(
        &mut paging_service.mapper,
        &mut paging_service.frame_allocator,
    )
    .unwrap();

    unsafe {
        video::vesa::init_screen(boot_info.framebuffer.as_mut().unwrap());
    }
    let console = video::vesa::console::Console::new(video::vesa::SCREEN.get().unwrap());
    video::console::CONSOLE.lock().replace(console);
    x86_64::instructions::interrupts::enable();
    InitServices {
        idt_service,
        gdt_service,
        paging_service,
    }
}

pub struct InitServices {
    pub idt_service: interrupts::IdtService,
    pub gdt_service: gdt::GdtService,
    pub paging_service: memory::PagingService,
}
