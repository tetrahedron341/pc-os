use core::ptr::addr_of;

use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 1;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 16 * 1024; // jesus christ how much memory does panic! need?
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(addr_of!(STACK));
            stack_start + STACK_SIZE
        };
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 16 * 1024;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(addr_of!(STACK));
            stack_start + STACK_SIZE
        };
        tss
    };

    static ref _GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        use x86_64::structures::gdt::DescriptorFlags as Flags;
        let data_selector = gdt.add_entry(Descriptor::UserSegment(
            Flags::USER_SEGMENT.bits() | Flags::PRESENT.bits() | Flags::WRITABLE.bits(),
        ));
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
                user_code_selector,
                user_data_selector,
            },
        )
    };

    pub static ref GDT: &'static GlobalDescriptorTable = &_GDT.0;
    pub static ref SELECTORS: &'static Selectors = &_GDT.1;
}

pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::Segment;
    use x86_64::instructions::tables::load_tss;
    use x86_64::registers::segmentation::{CS, SS};

    GDT.load();

    unsafe {
        CS::set_reg(SELECTORS.code_selector);
        SS::set_reg(SELECTORS.data_selector);
        load_tss(SELECTORS.tss_selector);
    }
}
