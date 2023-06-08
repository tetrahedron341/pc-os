/// Common fields of all configurations spaces
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct BaseHeader {
    pub vid: u16,
    pub did: u16,
    pub commmand: u16,
    pub status: u16,
    pub revision: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class: u8,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
}

/// General device configuration space
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct HeaderType0 {
    pub base: BaseHeader,
    pub base_addresses: [u32; 6],
    pub cis: u32,
    pub ss_vid: u16,
    pub ss_id: u16,
    pub exrom: u32,
    pub cap: u8,
    pub _res0: u8,
    pub _res1: u16,
    pub _res2: u32,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8,
}

/// Pci-to-pci configuration space
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct HeaderType1 {
    pub base: BaseHeader,
    pub base_addresses: [u32; 2],
    pub pri_bus: u8,
    pub sec_bus: u8,
    pub sub_bus: u8,
    pub sec_latency_timer: u8,
    pub io_base: u8,
    pub io_limit: u8,
    pub sec_status: u16,
    pub mem_base: u16,
    pub mem_limit: u16,
    pub prefetchable_mem_base: u16,
    pub prefetchable_mem_limit: u16,
    pub prefetchable_mem_base_upper: u32,
    pub prefetchable_mem_limit_upper: u32,
    pub io_base_upper: u16,
    pub io_limit_upper: u16,
    pub cap: u8,
    pub _res0: u8,
    pub _res1: u16,
    pub exrom: u32,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bridge_control: u16,
}
