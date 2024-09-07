// use core::sync::atomic::{AtomicUsize, Ordering};

// #[derive(Clone)]
// pub struct Handler<'a> {
//     mapper: &'a spin::Mutex<x86_64::structures::paging::RecursivePageTable<'a>>,
//     frame_allocator: &'a spin::Mutex<crate::memory::BootInfoFrameAllocator>,
// }

// const ACPI_PAGE_START: usize = 0x60000000;
// static ACPI_PAGE_COUNT: AtomicUsize = AtomicUsize::new(0);

// impl<'a> acpi::AcpiHandler for Handler<'a> {
//     unsafe fn map_physical_region<T>(
//         &self,
//         physical_address: usize,
//         size: usize,
//     ) -> acpi::PhysicalMapping<Self, T> {
//         use x86_64::structures::paging::{Mapper, Page, PageTableFlags, PhysFrame, Size4KiB};
//         use x86_64::{PhysAddr, VirtAddr};
//         let mut mapper = self.mapper.lock();
//         let mut frame_allocator = self.frame_allocator.lock();

//         let start_frame: PhysFrame<Size4KiB> =
//             PhysFrame::containing_address(PhysAddr::new(physical_address as u64));
//         let end_frame =
//             PhysFrame::containing_address(PhysAddr::new((physical_address + size) as u64));

//         let mut start_page: Option<VirtAddr> = None;

//         for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
//             let page = Page::from_start_address(VirtAddr::new(
//                 (ACPI_PAGE_START + 4096 * ACPI_PAGE_COUNT.fetch_add(1, Ordering::Relaxed)) as u64,
//             ))
//             .unwrap();

//             if start_page == None {
//                 start_page = Some(page.start_address());
//             }

//             mapper
//                 .map_to(
//                     page,
//                     frame,
//                     PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
//                     &mut *frame_allocator,
//                 )
//                 .unwrap()
//                 .flush();
//         }

//         let ptr = (start_page.unwrap().as_u64() as usize + physical_address % 4096) as *mut T;

//         acpi::PhysicalMapping::new(
//             physical_address,
//             core::ptr::NonNull::new_unchecked(ptr),
//             size,
//             size,
//             self.clone(),
//         )
//     }

//     fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {
//         // This leaks. Fix this.
//     }
// }
