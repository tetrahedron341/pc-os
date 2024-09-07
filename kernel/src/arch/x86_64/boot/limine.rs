use core::mem::MaybeUninit;

use crate::arch::{
    self,
    memory::{self, mmap::MemoryRegion, VirtAddr},
};

static MMAP_REQUEST: limine::LimineMmapRequest = limine::LimineMmapRequest::new(0);

static HHDM_REQUEST: limine::LimineHhdmRequest = limine::LimineHhdmRequest::new(0);

#[no_mangle]
fn _start() -> ! {
    arch::x86_64::interrupts::init_idt();
    arch::x86_64::gdt::init();

    let mmap_response = MMAP_REQUEST
        .get_response()
        .get()
        .expect("MMAP request failed");
    let limine_mmap = mmap_response.mmap().expect("MMAP request failed");
    let mmap: &'static [MemoryRegion] = {
        static mut MMAP_BUFFER: [MaybeUninit<MemoryRegion>; 256] = MaybeUninit::uninit_array();
        let mmap =
            unsafe { MaybeUninit::slice_assume_init_mut(&mut MMAP_BUFFER[..limine_mmap.len()]) };
        for (i, r) in limine_mmap.iter().enumerate() {
            mmap[i] = r.into();
        }
        mmap
    };

    let phys_mem_start = HHDM_REQUEST
        .get_response()
        .get()
        .expect("HHDM request failed")
        .offset;

    unsafe { arch::x86_64::memory::init(VirtAddr::new(phys_mem_start), mmap) };
    crate::allocator::init_heap().unwrap();
    arch::x86_64::syscall::init();

    // let modules = boot_info
    //     .modules
    //     .iter()
    //     .map(|m| unsafe { load_module(*m) })
    //     .collect();

    // unsafe {
    //     crate::video::vesa::init_screen(boot_info.framebuffer.as_mut().unwrap());
    // }
    // let console =
    //     crate::video::vesa::console::Console::new(crate::video::vesa::SCREEN.get().unwrap());
    // crate::video::console::CONSOLE.lock().replace(console);

    x86_64::instructions::interrupts::enable();

    const SERIAL_LOG_MIN: log::LevelFilter = log::LevelFilter::Info;
    const CONSOLE_LOG_MIN: log::LevelFilter = log::LevelFilter::Warn;

    crate::log::init(SERIAL_LOG_MIN, CONSOLE_LOG_MIN, 128);

    crate::init::kernel_main(crate::init::InitServices {
        modules: alloc::vec![],
        framebuffer: None,
    });
}

impl From<limine::LimineMemoryMapEntryType> for memory::mmap::MemoryKind {
    fn from(k: limine::LimineMemoryMapEntryType) -> Self {
        use limine::LimineMemoryMapEntryType::*;
        match k {
            Usable => memory::mmap::MemoryKind::Available,
            BootloaderReclaimable => memory::mmap::MemoryKind::Reserved,
            Reserved => memory::mmap::MemoryKind::Reserved,
            KernelAndModules => memory::mmap::MemoryKind::Reserved,
            _ => memory::mmap::MemoryKind::Other,
        }
    }
}

impl From<&limine::LimineMemmapEntry> for memory::mmap::MemoryRegion {
    fn from(e: &limine::LimineMemmapEntry) -> Self {
        memory::mmap::MemoryRegion {
            start: e.base as usize,
            len: e.len as usize,
            kind: e.typ.into(),
        }
    }
}
