use core::mem::MaybeUninit;

use x86_64::structures::paging::PageTableFlags;

use crate::arch::{
    self,
    memory::{self, mmap::MemoryRegion},
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

    // Set up a recursive page table index
    let (cr3, _cr3flags) = x86_64::registers::control::Cr3::read();
    let lvl4_page_table = unsafe {
        let ptr = (cr3.start_address().as_u64() + phys_mem_start)
            as *mut x86_64::structures::paging::PageTable;
        ptr.as_mut().unwrap()
    };
    // Find the highest unused level 4 entry. It will be used as the recursive index.
    let (recursive_index, recursive_entry) = lvl4_page_table
        .iter_mut()
        .enumerate()
        .filter_map(|(i, pte)| {
            // Require the recursive index to be higher half
            if pte.is_unused() && i >= 256 {
                Some((i as u16, pte))
            } else {
                None
            }
        })
        .last()
        .unwrap();
    // The recursive index points back to the level 4 page table
    // https://os.phil-opp.com/paging-implementation/#recursive-page-tables
    recursive_entry.set_addr(
        cr3.start_address(),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
    );
    x86_64::instructions::tlb::flush_all();

    unsafe { arch::x86_64::memory::init(recursive_index, mmap) };
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
