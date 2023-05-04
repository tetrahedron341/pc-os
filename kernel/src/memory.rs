use bootloader::boot_info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::{
    FrameAllocator, Page, PageTable, PageTableIndex, PhysFrame, RecursivePageTable, Size4KiB,
};
use x86_64::PhysAddr;

pub struct BootInfoFrameAllocator {
    memory_map: &'static [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    unsafe fn init(memory_map: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        frame_addresses.map(|a| PhysFrame::containing_address(PhysAddr::new(a)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

#[non_exhaustive]
pub struct PagingService {
    pub mapper: RecursivePageTable<'static>,
    pub frame_allocator: BootInfoFrameAllocator,
}

/// Initialize the kernel frame allocator and page mapper
///
/// # Safety
/// `recursive_index` must point to an actual level 4 page table.
/// `memory_map` must be a valid memory map that does not include any in-use memory pages.
pub unsafe fn init(
    recursive_index: u16,
    memory_map: &'static bootloader::boot_info::MemoryRegions,
) -> PagingService {
    let lvl_4_page_table = get_page_table(recursive_index);
    let mapper = RecursivePageTable::new(lvl_4_page_table).unwrap();
    let frame_allocator = BootInfoFrameAllocator::init(memory_map);
    PagingService {
        mapper,
        frame_allocator,
    }
}

/// Given a 9-bit page table index `xxx` such that `0oxxx_xxx_xxx_xxx_0000` points to the level 4 page table, create a pointer to that page table.
unsafe fn get_page_table(recursive_index: u16) -> &'static mut PageTable {
    let index = PageTableIndex::new(recursive_index);
    let page = Page::from_page_table_indices(index, index, index, index);
    let addr = page.start_address();

    &mut *addr.as_mut_ptr()
}

const PHYS_MEM_OFFSET: u64 = 0x0000_4000_0000_0000;

pub fn phys_to_virt(phys: x86_64::PhysAddr) -> x86_64::VirtAddr {
    x86_64::VirtAddr::new(phys.as_u64() + PHYS_MEM_OFFSET)
}
