use core::ptr::NonNull;

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

/// Pointer to the configuration space of a PCI(e) device
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenericPciHeaderPtr {
    Type0(NonNull<HeaderType0>),
    Type1(NonNull<HeaderType1>),
    Other(NonNull<BaseHeader>),
}

/// Creates a generic label for a given class of PCI device
pub fn device_kind(class: u8, subclass: u8) -> Option<&'static str> {
    let kind = match (class, subclass) {
        (0, _) => "Unspecified",
        (1, _) => "Mass Storage Controller",
        (2, 1) => "Ethernet Controller",
        (2, _) => "Network Controller",
        (3, _) => "Display Controller",
        (4, _) => "Multimedia Controller",
        (5, _) => "Memory Controller",
        (6, 4) => "PCI-to-PCI Bridge",
        (6, _) => "Bridge",
        (7, _) => "Simple Communication Controller",
        (8, _) => "Base System Peripheral",
        (9, _) => "Input Device Controller",
        (10, _) => "Docking Station",
        (11, _) => "Processor",
        (12, _) => "Serial Bus Controller",
        (13, _) => "Wireless Controller",
        _ => return None,
    };
    Some(kind)
}

/// Interface for a system capable of finding the configuration space of a given PCI(e) device.
pub trait PciHandler {
    type Error: core::fmt::Display;

    /// Finds a memory-mapped pointer to the configuration space of the given device.
    fn get_config_space(segment_group: u16, bus: u8, device: u8, function: u8) -> Result<GenericPciHeaderPtr, Self::Error>;
}