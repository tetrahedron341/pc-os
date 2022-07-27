use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page as X86Page, PageSize, PageTable, PageTableFlags,
    PhysFrame as X86PhysFrame, Size4KiB,
};

use self::mmap::MemoryRegion;

pub type VirtAddr = x86_64::VirtAddr;
pub type PhysAddr = x86_64::PhysAddr;
pub type PhysFrame = X86PhysFrame<Size4KiB>;
pub type Page = X86Page<Size4KiB>;

mod frame_allocator;
pub(super) mod mmap;
pub mod space;

pub static FRAME_ALLOCATOR: OnceCell<Mutex<frame_allocator::BootInfoFrameAllocator>> =
    OnceCell::uninit();
pub static MAPPER: OnceCell<Mutex<OffsetPageTable>> = OnceCell::uninit();

/// Initialize the kernel frame allocator and page mapper
///
/// # Safety
/// `phys_mem_offset` must point to the beginning of physical memory.
/// `memory_map` must be a valid memory map that does not include any in-use memory pages.
pub(super) unsafe fn init(phys_mem_offset: VirtAddr, memory_map: &'static [MemoryRegion]) {
    PHYS_MEM_OFFSET = phys_mem_offset;
    let lvl_4_page_table = get_page_table();
    MAPPER.init_once(|| Mutex::new(OffsetPageTable::new(lvl_4_page_table, phys_mem_offset)));

    let falloc = frame_allocator::BootInfoFrameAllocator::init(memory_map);
    crate::serial_println!(
        "[arch::x86_64::memory::init] Free memory: {} KB",
        falloc.free_pages() * 4
    );
    FRAME_ALLOCATOR.init_once(|| Mutex::new(falloc));
}

unsafe fn get_page_table() -> &'static mut PageTable {
    let cr3 = x86_64::registers::control::Cr3::read().0.start_address();
    let cr3_virt = phys_to_virt(cr3);

    cr3_virt.as_mut_ptr::<PageTable>().as_mut().unwrap()
}

pub fn allocate_frame<S>() -> Option<X86PhysFrame<S>>
where
    S: PageSize,
    frame_allocator::BootInfoFrameAllocator: x86_64::structures::paging::FrameAllocator<S>,
{
    FRAME_ALLOCATOR.get().unwrap().lock().allocate_frame()
}

pub fn deallocate_frame(_f: PhysFrame) {}

/// # Safety
/// See the [`x86_64::structures::paging::Mapper::map_to`] docs.
pub unsafe fn map_page<S>(page: X86Page<S>, frame: X86PhysFrame<S>)
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

static mut PHYS_MEM_OFFSET: VirtAddr = VirtAddr::zero();

pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    unsafe { PHYS_MEM_OFFSET + phys.as_u64() }
}
