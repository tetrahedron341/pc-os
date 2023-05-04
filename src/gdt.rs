use lazy_static::lazy_static;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 16 * 1024; // jesus christ how much memory does panic! need?
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe {&STACK});
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 16 * 1024;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe {&STACK});
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };

    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        use x86_64::structures::gdt::DescriptorFlags as Flags;
        let data_selector = gdt.add_entry(Descriptor::UserSegment((Flags::USER_SEGMENT | Flags::PRESENT | Flags::WRITABLE).bits()));
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        (gdt, Selectors {code_selector, data_selector, tss_selector, user_code_selector, user_data_selector})
    };
}

pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

pub struct GdtService { _private: () }

pub fn init() -> GdtService {
    use x86_64::instructions::segmentation::{set_cs, load_ss};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();

    unsafe {
        set_cs(GDT.1.code_selector);
        load_ss(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }

    GdtService { _private: () }
}