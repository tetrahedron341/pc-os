use x86_64::structures::paging::{OffsetPageTable, PageTable, PhysFrame, Size4KiB};

use super::{allocate_frame, phys_to_virt, PhysAddr, MAPPER};

/// x86_64 address space.
///
/// Addresses at or above 0xffff800000000000 belongs to global kernel space.
/// Everything below belongs to this struct alone, and the corresponding page table directories will
/// be freed once thie struct is dropped.
pub struct Space {
    cr3: PhysAddr,
}

impl Space {
    pub fn new() -> Self {
        let page_table_frame = allocate_frame::<Size4KiB>().unwrap();

        let mut s = Space {
            cr3: page_table_frame.start_address(),
        };

        // Copy all higher-half (kernel) page mappings
        let mut pt = s.page_table();
        let mut kpt = MAPPER.get().unwrap().lock();
        for i in 0..256 {
            pt.level_4_table()[i].set_unused();
        }
        for i in 256..512 {
            pt.level_4_table()[i] = kpt.level_4_table()[i].clone();
        }

        s
    }

    pub fn page_table(&mut self) -> OffsetPageTable<'_> {
        let ptr = phys_to_virt(self.cr3).as_mut_ptr::<PageTable>();
        let page_table = unsafe { ptr.as_mut().unwrap() };

        unsafe { OffsetPageTable::new(page_table, super::PHYS_MEM_OFFSET) }
    }

    pub fn load(&mut self) {
        unsafe {
            x86_64::registers::control::Cr3::write(
                PhysFrame::from_start_address(self.cr3).unwrap(),
                x86_64::registers::control::Cr3Flags::empty(),
            )
        }
    }
}
