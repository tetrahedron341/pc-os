use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
};

use self::mmap::MemoryRegion;

pub type VirtAddr = x86_64::VirtAddr;
pub type PhysAddr = x86_64::PhysAddr;

mod frame_allocator;
pub(super) mod mmap;

static FRAME_ALLOCATOR: OnceCell<Mutex<frame_allocator::BootInfoFrameAllocator>> =
    OnceCell::uninit();
static MAPPER: OnceCell<Mutex<OffsetPageTable>> = OnceCell::uninit();

/// Initialize the kernel frame allocator and page mapper
///
/// # Safety
/// `phys_mem_offset` must point to the beginning of physical memory.
/// `memory_map` must be a valid memory map that does not include any in-use memory pages.
pub(super) unsafe fn init(phys_mem_offset: VirtAddr, memory_map: &'static [MemoryRegion]) {
    let lvl_4_page_table = get_page_table(phys_mem_offset);
    MAPPER.init_once(|| Mutex::new(OffsetPageTable::new(lvl_4_page_table, phys_mem_offset)));

    let falloc = frame_allocator::BootInfoFrameAllocator::init(memory_map);
    crate::serial_println!(
        "[arch::x86_64::memory::init] Free memory: {} KB",
        falloc.free_pages() * 4
    );
    FRAME_ALLOCATOR.init_once(|| Mutex::new(falloc));
}

unsafe fn get_page_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    let cr3 = x86_64::registers::control::Cr3::read()
        .0
        .start_address()
        .as_u64();
    let cr3_virt = phys_mem_offset + cr3;

    cr3_virt.as_mut_ptr::<PageTable>().as_mut().unwrap()
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
    x86_64::structures::paging::OffsetPageTable<'static>: x86_64::structures::paging::Mapper<S>,
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
