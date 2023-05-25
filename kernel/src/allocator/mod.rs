use x86_64::structures::paging::{mapper::MapToError, Page, Size4KiB};
use x86_64::VirtAddr;

mod bump;
mod fixed_size_block_allocator;
mod linked_list;

use fixed_size_block_allocator::FixedSizeBlockAllocator as Heap;

pub const HEAP_START: usize = 0xFFFF_E000_0000_0000;
pub const HEAP_SIZE: usize = 16 * 1024 * 1024; // 16 MB

/// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

#[global_allocator]
static ALLOC: Locked<Heap> = Locked::new(Heap::new());

#[alloc_error_handler]
fn alloc_err(err: alloc::alloc::Layout) -> ! {
    panic!("ALLOC ERROR: {:?}", err)
}

pub fn init_heap(// mapper: &mut impl Mapper<Size4KiB>,
    // frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    log::info!("Mapping heap pages... ");
    for page in page_range {
        let frame = crate::arch::memory::allocate_frame::<Size4KiB>()
            .ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            crate::arch::memory::map_page(page, frame);
        }
    }
    log::info!("OK");

    log::info!("Initializing heap...");

    unsafe {
        ALLOC.lock().init(HEAP_START, HEAP_SIZE);
    }

    log::info!("OK");

    Ok(())
}

fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // addr already aligned
    } else {
        addr - remainder + align
    }
}
