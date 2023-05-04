use x86_64::structures::paging::{PageTable, RecursivePageTable, FrameAllocator, Size4KiB, PhysFrame};
use x86_64::{VirtAddr, PhysAddr};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr() .. r.range.end_addr());
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

pub struct PagingService {
    pub mapper: RecursivePageTable<'static>,
    pub frame_allocator: BootInfoFrameAllocator,
    _private: (),
}

pub unsafe fn init(recursive_page_table_addr: VirtAddr, memory_map: &'static bootloader::bootinfo::MemoryMap) -> PagingService {
    let lvl_4_page_table = active_level_4_table(recursive_page_table_addr);
    let mapper = RecursivePageTable::new(lvl_4_page_table).unwrap();
    let frame_allocator = BootInfoFrameAllocator::init(memory_map);
    PagingService {
        mapper,
        frame_allocator,
        _private: ()
    }
}

unsafe fn active_level_4_table(recursive_page_table_addr: VirtAddr) -> &'static mut PageTable {
    &mut *recursive_page_table_addr.as_mut_ptr()
}