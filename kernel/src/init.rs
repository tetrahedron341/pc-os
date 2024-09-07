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

    let modules = boot_info
        .modules
        .iter()
        .map(|m| unsafe { BootModule::load(*m) })
        .collect();

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
        modules,
    }
}

pub struct InitServices {
    pub idt_service: interrupts::IdtService,
    pub gdt_service: gdt::GdtService,
    pub paging_service: memory::PagingService,
    pub modules: alloc::vec::Vec<BootModule>,
}

pub struct BootModule {
    pub name: alloc::string::String,
    pub data: &'static mut [u8],
}

impl core::fmt::Debug for BootModule {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootModule")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl BootModule {
    unsafe fn load(module_desc: bootloader::boot_info::Module) -> Self {
        let ptr =
            crate::memory::phys_to_virt(x86_64::PhysAddr::new(module_desc.phys_addr)).as_mut_ptr();
        BootModule {
            name: core::str::from_utf8(&module_desc.name)
                .unwrap()
                .trim_end_matches('\0')
                .into(),
            data: core::slice::from_raw_parts_mut(ptr, module_desc.len),
        }
    }
}
