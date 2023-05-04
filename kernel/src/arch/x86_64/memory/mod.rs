use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageSize, PageTable, PageTableFlags, PageTableIndex, PhysFrame,
    RecursivePageTable,
};

use self::mmap::MemoryRegion;

pub type VirtAddr = x86_64::VirtAddr;
pub type PhysAddr = x86_64::PhysAddr;

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

    let falloc = frame_allocator::BootInfoFrameAllocator::init(memory_map);
    crate::serial_println!(
        "[arch::x86_64::memory::init] Free memory: {} KB",
        falloc.free_pages() * 4
    );
    FRAME_ALLOCATOR.init_once(|| Mutex::new(falloc));
}

/// Given a 9-bit page table index `xxx` such that `0oxxx_xxx_xxx_xxx_0000` points to the level 4 page table, create a pointer to that page table.
unsafe fn get_page_table(recursive_index: u16) -> &'static mut PageTable {
    let index = PageTableIndex::new(recursive_index);
    let page = Page::from_page_table_indices(index, index, index, index);
    let addr = page.start_address();

    &mut *addr.as_mut_ptr()
}

pub fn allocate_frame<S>() -> Option<PhysFrame<S>>
where
    S: PageSize,
    frame_allocator::BootInfoFrameAllocator: x86_64::structures::paging::FrameAllocator<S>,
{
    FRAME_ALLOCATOR.get().unwrap().lock().allocate_frame()
}

/// # Safety
/// See the [`x86_64::structures::paging::Mapper::map_to`] docs.
pub unsafe fn map_page<S>(page: Page<S>, frame: PhysFrame<S>)
where
    S: PageSize + core::fmt::Debug,
    x86_64::structures::paging::RecursivePageTable<'static>: x86_64::structures::paging::Mapper<S>,
{
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
