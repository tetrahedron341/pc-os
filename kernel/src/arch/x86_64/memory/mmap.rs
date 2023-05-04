/// Describes a region of physical memory at boot-time.
/// Should be derived from bootloader structures.
#[derive(Debug)]
pub struct MemoryRegion {
    /// Physical address of the first byte of the region
    pub start: usize,
    /// Length in bytes of the region
    pub len: usize,

    /// Whether the region is available or in use
    pub kind: MemoryKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MemoryKind {
    /// Memory is not in use and can be freely used by the kernel.
    Available,
    /// Unusable memory.
    Reserved,
    /// Anything else. Do not allocate.
    Other,
}
