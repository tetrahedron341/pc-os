use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, PageTable, PageTableFlags, PageTableIndex, RecursivePageTable,
};

use self::mmap::MemoryRegion;

pub type VirtAddr = x86_64::VirtAddr;
pub type PhysAddr = x86_64::PhysAddr;
pub type PhysFrame = x86_64::structures::paging::PhysFrame;
pub type Page = x86_64::structures::paging::Page;

mod frame_allocator;
pub(super) mod mmap;

static FRAME_ALLOCATOR: OnceCell<Mutex<frame_allocator::BootInfoFrameAllocator>> =
    OnceCell::uninit();
static MAPPER: OnceCell<Mutex<RecursivePageTable>> = OnceCell::uninit();

/// Initialize the kernel frame allocator and page mapper
///
/// # Safety
/// `recursive_index` must point to an actual level 4 page table.
/// `memory_map` must be a valid memory map that does not include any in-use memory pages.
pub(super) unsafe fn init(recursive_index: u16, memory_map: &'static [MemoryRegion]) {
    let lvl_4_page_table = get_page_table(recursive_index);
    MAPPER.init_once(|| Mutex::new(RecursivePageTable::new(lvl_4_page_table).unwrap()));
    FRAME_ALLOCATOR
        .init_once(|| Mutex::new(frame_allocator::BootInfoFrameAllocator::init(memory_map)));
}

/// Given a 9-bit page table index `xxx` such that `0oxxx_xxx_xxx_xxx_0000` points to the level 4 page table, create a pointer to that page table.
unsafe fn get_page_table(recursive_index: u16) -> &'static mut PageTable {
    let index = PageTableIndex::new(recursive_index);
    let page = Page::from_page_table_indices(index, index, index, index);
    let addr = page.start_address();

    &mut *addr.as_mut_ptr()
}

pub fn allocate_frame() -> Option<PhysFrame> {
    FRAME_ALLOCATOR.get().unwrap().lock().allocate_frame()
}

pub unsafe fn map_page(page: Page, frame: PhysFrame) {
    MAPPER
        .get()
        .unwrap()
        .lock()
        .map_to(
            page,
            frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            &mut *FRAME_ALLOCATOR.get().unwrap().lock(),
        )
        .unwrap()
        .flush();
}

const PHYS_MEM_OFFSET: u64 = 0x0000_4000_0000_0000;

pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    x86_64::VirtAddr::new(phys.as_u64() + PHYS_MEM_OFFSET)
}
