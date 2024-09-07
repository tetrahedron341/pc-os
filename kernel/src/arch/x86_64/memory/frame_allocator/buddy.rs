use core::ptr::NonNull;

use bitvec::slice::BitSlice;
use x86_64::PhysAddr;

/// A physical memory allocator implemented using a buddy allocator in each available memory region.
pub struct BuddyAllocatorManager<const ENTRIES: usize> {
    /// Total available memory.
    remaining: usize,
    entries: [Option<BuddyAllocatorManagerEntry>; ENTRIES],
}

struct BuddyAllocatorManagerEntry {
    /// Acts as both a pointer to the memory managed by this allocator, and to the allocator
    /// struct itself.
    allocator: NonNull<BuddyAllocator>,
    /// The physical address of the start of this region.
    physical_start: PhysAddr,
    /// The total size of the region
    region_size: usize,
    /// The amount of available bytes in this region
    remaining: usize,
}

/// Placed at the beginning of the memory region it is meant to allocate from. Assumes the entire
/// region is mapped continuously into virtual memory.
struct BuddyAllocator {
    /// Bytes used by the allocator itself, including bitmaps.
    allocator_len: usize,
    /// Total size of the region managed by this allocator
    region_len: usize,
    /// Bytes not currently in use or allocated by this allocator.
    remaining: usize,
    /// Size in bytes of every region in layer zero. The region size of each following layer is half
    /// the size of the layer before.
    ///
    /// Must be a power of two.
    layer_zero_region_size: usize,
    /// Number of layers. Must be less than log_2(layer_zero_region_size) - 10.
    num_layers: usize,
    /// The bitmaps representing each layer.
    layers: [&'static mut BitSlice],
}
