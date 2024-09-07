use core::ops::Range;

use bitvec::slice::BitSlice;
use x86_64::{
    structures::paging::{
        FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB,
    },
    PhysAddr,
};

use crate::arch::memory::mmap::{MemoryKind, MemoryRegion};

const PAGE_SIZE: usize = 4096;

/// A physical memory allocator implemented using a buddy allocator in each available memory region.
pub struct BuddyAllocatorManager<const ENTRIES: usize> {
    /// Total available memory.
    remaining: usize,
    entries: [Option<BuddyAllocator>; ENTRIES],
}

impl<const N: usize> BuddyAllocatorManager<N> {
    pub unsafe fn from_mmap(
        memory_map: &'static [MemoryRegion],
        phys_to_virt_offset: *mut u8,
    ) -> Self {
        let mut entries = core::array::from_fn(|_| None);
        let mut i = 0;
        let mut remaining = 0;
        for region in memory_map {
            match region.kind {
                MemoryKind::Available => {}
                _ => continue,
            }
            let phys_start = PhysAddr::new(region.start as u64);
            let virt_start = phys_to_virt_offset.add(region.start);
            crate::println!("Creating allocator in {region:?}");

            let e = BuddyAllocator::new(phys_start, virt_start, region.len);

            remaining += e.remaining;
            entries[i] = Some(e);
            i += 1;
        }

        BuddyAllocatorManager { remaining, entries }
    }

    pub fn remaining(&self) -> usize {
        self.remaining
    }
}

unsafe impl<S: PageSize, const N: usize> FrameAllocator<S> for BuddyAllocatorManager<N>
where
    BuddyAllocator: FrameAllocator<S>,
{
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        for e in self.entries.iter_mut().filter_map(Option::as_mut) {
            if let Some(frame) = e.allocate_frame() {
                self.remaining -= frame.size() as usize;
                return Some(frame);
            }
        }
        None
    }
}

impl<S: PageSize, const N: usize> FrameDeallocator<S> for BuddyAllocatorManager<N>
where
    BuddyAllocator: FrameDeallocator<S>,
{
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<S>) {
        for e in self.entries.iter_mut().filter_map(Option::as_mut) {
            if e.contains_frame(frame) {
                e.deallocate_frame(frame);
                self.remaining += frame.size() as usize;
            }
        }
        panic!("attempt to deallocate page that is not managed by any allocator")
    }
}

/// Placed at the beginning of the memory region it is meant to allocate from. Assumes the entire
/// region is mapped continuously into virtual memory.
struct BuddyAllocator {
    /// Physical address of the start of this region.
    phys_start: PhysAddr,
    // /// Pointer to the start of the map of this region.
    // virt_start: *mut u8,
    /// Size in bytes of this region.
    region_size: usize,
    /// Bytes of not in-use memory within this region.
    remaining: usize,
    /// Bitmap of in-use memory in the region
    bitmap: BuddyBitmap<'static>,
}

impl BuddyAllocator {
    /// Creates a new [`BuddyAllocator`] at the specified location, and places a bitmap at its start.
    ///
    /// # Safety
    /// `virt_start` must point to the beginning of an entire map of the region `(phys_start..phys_start+size_bytes)`
    /// into virtual memory. This mapping must remain valid as long as this allocator lives.
    ///
    /// ### This physical memory must not be used by *anything* else.
    pub unsafe fn new(
        phys_start: PhysAddr,
        virt_start: *mut u8,
        size_bytes: usize,
    ) -> BuddyAllocator {
        let pages = size_bytes.next_multiple_of(PAGE_SIZE) / PAGE_SIZE;
        // 5 pages => 8 blocks => 4 layers (8,4,2,1)
        let layers = pages.next_power_of_two().ilog2() as usize + 1;
        let bitmap_len = BuddyBitmap::bytes_required_for_n_layers(layers);
        let bitmap_mem =
            BitSlice::from_slice_mut(core::slice::from_raw_parts_mut(virt_start, bitmap_len));
        let mut bitmap = BuddyBitmap::new(
            layers,
            &mut bitmap_mem[..BuddyBitmap::bits_required_for_n_layers(layers)],
        );
        // mark the bitmap as used
        let bitmap_pages = bitmap_len.next_multiple_of(PAGE_SIZE) / PAGE_SIZE;
        bitmap.dealloc_range(bitmap_pages..bitmap_len, 0).unwrap();

        BuddyAllocator {
            phys_start,
            region_size: size_bytes,
            remaining: size_bytes - bitmap_len,
            bitmap,
        }
    }

    /// Returns true iff the provided physical frame lies entirely within the
    /// region of this allocator.
    fn contains_frame<S: PageSize>(&self, frame: PhysFrame<S>) -> bool {
        let start = frame.start_address();
        let end = start + frame.size();

        let region = self.phys_start..(self.phys_start + self.region_size as u64);
        region.contains(&start) && region.contains(&end)
    }
}

unsafe impl FrameAllocator<Size4KiB> for BuddyAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<Size4KiB>> {
        let Ok(idx) = self.bitmap.alloc_range(1, 0) else {return None};
        let start = self.phys_start + idx.start * 4096;
        self.remaining -= 4096;
        return Some(PhysFrame::from_start_address(start).unwrap());
    }
}

unsafe impl FrameAllocator<Size2MiB> for BuddyAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<Size2MiB>> {
        let Ok(idx) = self.bitmap.alloc_range(1<<9, 0) else {return None};
        let start = self.phys_start + idx.start * 4096;
        self.remaining -= 4096 * 512;
        return Some(PhysFrame::from_start_address(start).unwrap());
    }
}

unsafe impl FrameAllocator<Size1GiB> for BuddyAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<Size1GiB>> {
        let Ok(idx) = self.bitmap.alloc_range(1<<18, 0) else {return None};
        let start = self.phys_start + idx.start * 4096;
        self.remaining -= 4096 * 512 * 512;
        return Some(PhysFrame::from_start_address(start).unwrap());
    }
}

impl FrameDeallocator<Size4KiB> for BuddyAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        assert!(frame.start_address() >= self.phys_start);
        assert!(frame.start_address() - self.phys_start < self.region_size as u64);

        let idx = (frame.start_address() - self.phys_start) as usize / 4096;
        self.bitmap.dealloc_bit(idx, 0).unwrap();
        self.remaining += 4096;
    }
}

impl FrameDeallocator<Size2MiB> for BuddyAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size2MiB>) {
        assert!(frame.start_address() >= self.phys_start);
        assert!(frame.start_address() - self.phys_start < self.region_size as u64);

        let idx = (frame.start_address() - self.phys_start) as usize / (4096 * 512);
        self.bitmap.dealloc_bit(idx, 9).unwrap();
        self.remaining += 4096 * 512;
    }
}

impl FrameDeallocator<Size1GiB> for BuddyAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size1GiB>) {
        assert!(frame.start_address() >= self.phys_start);
        assert!(frame.start_address() - self.phys_start < self.region_size as u64);

        let idx = (frame.start_address() - self.phys_start) as usize / (4096 * 512 * 512);
        self.bitmap.dealloc_bit(idx, 18).unwrap();
        self.remaining += 4096 * 512 * 512;
    }
}

#[derive(Debug)]
struct BuddyBitmap<'storage> {
    /// Number of layers. Each layer has half the bits of the layer below, and
    /// the topmost layer (`[num_layers-1]`) has one bit, so layer `[0]` has `2^(num_layers-1)` bits.
    num_layers: usize,
    /// The bitmap holding the layers of the allocator. Zero means free, one
    /// means used. The layers are placed consecutively, in order from low to high.
    ///
    /// This slice must have a length of exactly 2^(num_layers)-1.
    storage: &'storage mut BitSlice<u8>,
}

#[derive(Debug)]
#[allow(unused)]
enum AllocErr {
    IdxOutOfRange { idx: usize, layer: usize },
    DoubleFree { idx: usize, layer: usize },
    OutOfMemory { layer: usize },
    LayerDoesNotExist { layer: usize },
}

impl core::fmt::Display for AllocErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<'storage> BuddyBitmap<'storage> {
    /// Initialize a bitmap with `num_layers` layers that is completely used.
    /// To mark free space, use `dealloc_bit` or `dealloc_range`.
    /// # Panics
    /// Panics if given a bitslice that is not exactly (2^num_layers)-1 bits long.
    pub fn new(num_layers: usize, storage: &'storage mut BitSlice<u8>) -> Self {
        let ex_storage_len = (1 << num_layers) - 1;
        assert_eq!(
            storage.len(),
            ex_storage_len,
            "`storage` must be exactly {ex_storage_len} bits long."
        );
        storage.fill(true);
        BuddyBitmap {
            num_layers,
            storage,
        }
    }

    pub fn bits_required_for_n_layers(num_layers: usize) -> usize {
        (1 << num_layers) - 1
    }

    pub fn bytes_required_for_n_layers(num_layers: usize) -> usize {
        Self::bits_required_for_n_layers(num_layers).next_multiple_of(8) / 8
    }

    fn layer_range(&self, mut layer: usize) -> Range<usize> {
        let mut len = 1 << (self.num_layers - 1);
        let mut start = 0;
        while layer > 0 {
            start += len;
            layer -= 1;
            len >>= 1;
        }
        start..start + len
    }

    fn get_layer(&self, layer: usize) -> &BitSlice<u8> {
        let range = self.layer_range(layer);
        &self.storage[range]
    }

    fn get_layer_mut(&mut self, layer: usize) -> &mut BitSlice<u8> {
        let range = self.layer_range(layer);
        &mut self.storage[range]
    }

    fn dealloc_range(&mut self, idx_range: Range<usize>, layer: usize) -> Result<(), AllocErr> {
        if layer >= self.num_layers {
            return Err(AllocErr::LayerDoesNotExist { layer });
        }

        let blocks = &self.get_layer(layer)[idx_range.clone()];
        if let Some(idx) = blocks.first_zero() {
            return Err(AllocErr::DoubleFree { idx, layer });
        }
        let above_start: usize;
        let above_end: usize;
        // dealloc anything on the edges that isn't paired
        if idx_range.start % 2 == 1 {
            self.dealloc_bit(idx_range.start, layer)?;
            above_start = (idx_range.start + 1) / 2;
        } else {
            above_start = idx_range.start / 2;
        }
        if idx_range.end % 2 == 1 {
            self.dealloc_bit(idx_range.end - 1, layer)?;
            above_end = (idx_range.end - 1) / 2;
        } else {
            above_end = idx_range.end / 2;
        }

        let above_idx_range = above_start..above_end;
        if !above_idx_range.is_empty() {
            self.dealloc_range(above_idx_range, layer + 1)
        } else {
            Ok(())
        }
    }

    /// Deallocates the given bit in the given layer, and attempts to merge.
    fn dealloc_bit(&mut self, idx: usize, layer: usize) -> Result<(), AllocErr> {
        let pair_idx = idx & (!1);
        let num_layers = self.num_layers;
        // top layer
        if layer == num_layers - 1 {
            if self.get_layer_mut(layer).replace(0, false) == false {
                return Err(AllocErr::DoubleFree { idx, layer });
            } else {
                return Ok(());
            }
        }

        let (mut bit, mut buddy) = {
            let mut pair = {
                let pair_slice = self
                    .get_layer_mut(layer)
                    .get_mut(pair_idx..pair_idx + 2)
                    .ok_or(AllocErr::IdxOutOfRange { idx, layer })?;
                let (fst, snd) = pair_slice.split_at_mut(1);
                [fst.get_mut(0), snd.get_mut(0)]
            };
            let bit = pair[idx % 2].take().unwrap();
            let buddy = pair[(idx + 1) % 2].take().unwrap();
            (bit, buddy)
        };
        if bit.replace(false) == false {
            return Err(AllocErr::DoubleFree { idx, layer });
        }
        if (layer + 1) < num_layers && buddy == false {
            bit.set(true);
            buddy.set(true);
            drop(bit);
            drop(buddy);
            return self.dealloc_bit(idx / 2, layer + 1);
        } else {
            Ok(())
        }
    }

    /// Returns index of allocated bit in the given layer, splitting blocks on higher layers if necessary.
    fn alloc_bit(&mut self, layer: usize) -> Result<usize, AllocErr> {
        if layer >= self.num_layers {
            return Err(AllocErr::LayerDoesNotExist { layer });
        }

        if let Some(idx) = self.get_layer(layer).first_zero() {
            self.get_layer_mut(layer).set(idx, true);
            Ok(idx)
        } else {
            match self.alloc_bit(layer + 1) {
                Err(AllocErr::LayerDoesNotExist { .. } | AllocErr::OutOfMemory { .. }) => {
                    Err(AllocErr::OutOfMemory { layer })
                }
                Err(e) => Err(e),
                Ok(above_idx) => {
                    let idx = above_idx * 2;
                    // Set the right half of the block free, the left half should already be used
                    self.get_layer_mut(layer).set(idx + 1, false);
                    Ok(idx)
                }
            }
        }
    }

    /// Attempts to allocate a range of blocks by breaking up a larger block and trimming it down.
    fn alloc_range(&mut self, len: usize, layer: usize) -> Result<Range<usize>, AllocErr> {
        if layer >= self.num_layers {
            return Err(AllocErr::LayerDoesNotExist { layer });
        }
        if len == 0 {
            return Ok(0..0);
        } else if len == 1 {
            let idx = self.alloc_bit(layer)?;
            return Ok(idx..idx + 1);
        } else {
            let above_len = len.next_multiple_of(2) / 2;
            let above_range = self.alloc_range(above_len, layer + 1)?;
            let alloced_range = above_range.start * 2..above_range.end * 2;
            let to_dealloc = alloced_range.start + len..alloced_range.end;
            self.dealloc_range(to_dealloc, layer)?; // Anything that was allocated that we dont need gets dealloced.
            return Ok(alloced_range.start..alloced_range.start + len);
        }
    }
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::BitArray;

    use super::*;

    #[test_case]
    fn layer_range() {
        let bbm = BuddyBitmap {
            num_layers: 5,
            storage: &mut BitArray::<[u8; 0]>::ZERO[..],
        };
        assert_eq!(bbm.layer_range(0), 0..16);
        assert_eq!(bbm.layer_range(1), 16..24);
        assert_eq!(bbm.layer_range(2), 24..28);
        assert_eq!(bbm.layer_range(3), 28..30);
        assert_eq!(bbm.layer_range(4), 30..31);
    }

    #[test_case]
    fn bbm_alloc_dealloc() {
        use alloc::vec;
        let mut storage = vec![0u8; 2048 / 8];
        let mut bbm = BuddyBitmap {
            num_layers: 10,
            storage: &mut BitSlice::from_slice_mut(&mut storage)[..2047],
        };
        bbm.storage.fill(true);
        bbm.get_layer_mut(9).set(0, false);

        for layer in 0..10 {
            let num_blocks = 1 << (9 - layer);
            let mut alloced_blocks = vec![];
            for i in 0..num_blocks {
                match bbm.alloc_bit(layer) {
                    Ok(idx) => alloced_blocks.push(idx),
                    Err(e) => panic!("Allocation error on layer {layer}, i {i}.\n{e}"),
                }
            }
            crate::println!("layer {layer}, allocs:\n{alloced_blocks:?}");
            for idx in alloced_blocks.drain(..) {
                if let Err(e) = bbm.dealloc_bit(idx, layer) {
                    panic!("Deallocation error on layer {layer}, index {idx}.\n{e}");
                }
            }
        }
    }
}
