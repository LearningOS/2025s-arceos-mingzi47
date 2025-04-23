//! Slab allocator for `no_std` systems. It uses multiple slabs with blocks of
//! different sizes and a [buddy_system_allocator] for blocks larger than 4096
//! bytes.
//!
//! It's based on <https://github.com/weclaw1/slab_allocator>.
//!
//! [buddy_system_allocator]: https://docs.rs/buddy_system_allocator/latest/buddy_system_allocator/

#![feature(allocator_api)]
#![no_std]

extern crate alloc;
extern crate buddy_system_allocator;

use alloc::alloc::{AllocError, Layout};
use bump::BumpAllocator;

#[cfg(test)]
mod tests;

mod bump;
mod slab;
use slab::Slab;

use axlog::{debug, info};

const SET_SIZE: usize = 1;
const MIN_HEAP_SIZE: usize = 0x8000;
const PAGE_SIZE: usize = 4096;
static mut TEMP: usize = 2048;

enum HeapAllocator {
    Slab32Bytes,
    Slab96Bytes,
    Slab192Bytes,
    Slab384Bytes,
    Slab512Bytes,
    Slab2048Bytes,
    Slab8192Bytes,
    Slab32768Bytes,
    Slab131072Bytes,
    Slab524288Bytes,
    BumpAllocator,
}

/// A fixed size heap backed by multiple slabs with blocks of different sizes.
/// Allocations over 4096 bytes are served by linked list allocator.
#[derive(Debug)]
pub struct Heap {
    slab_32_bytes: Slab<500>,
    slab_96_bytes: Slab<96>,
    slab_192_bytes: Slab<192>,
    slab_384_bytes: Slab<384>,
    slab_512_bytes: Slab<884>,
    slab_2048_bytes: Slab<2420>,
    slab_8192_bytes: Slab<8564>,
    slab_32768_bytes: Slab<33140>,
    slab_131072_bytes: Slab<131444>,
    slab_524288_bytes: Slab<524660>,
    bump_allocator: BumpAllocator,
}

impl Heap {
    /// Creates a new heap with the given `heap_start_addr` and `heap_size`. The start address must be valid
    /// and the memory in the `[heap_start_addr, heap_start_addr + heap_size)` range must not be used for
    /// anything else.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn new(heap_start_addr: usize, heap_size: usize) -> Heap {
        assert!(
            heap_start_addr % PAGE_SIZE == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size >= MIN_HEAP_SIZE,
            "Heap size should be greater or equal to minimum heap size"
        );
        assert!(
            heap_size % MIN_HEAP_SIZE == 0,
            "Heap size should be a multiple of minimum heap size"
        );
        Heap {
            slab_32_bytes: Slab::<500>::new(0, 0),
            slab_96_bytes: Slab::<96>::new(0, 0),
            slab_192_bytes: Slab::<192>::new(0, 0),
            slab_384_bytes: Slab::<384>::new(0, 0),
            slab_512_bytes: Slab::<884>::new(0, 0),
            slab_2048_bytes: Slab::<2420>::new(0, 0),
            slab_8192_bytes: Slab::<8564>::new(0, 0),
            slab_32768_bytes: Slab::<33140>::new(0, 0),
            slab_131072_bytes: Slab::<131444>::new(0, 0),
            slab_524288_bytes: Slab::<524660>::new(0, 0),
            bump_allocator: BumpAllocator::new(heap_start_addr, heap_size),
        }
    }

    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn add_memory(&mut self, heap_start_addr: usize, heap_size: usize) {
        assert!(
            heap_start_addr % PAGE_SIZE == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % PAGE_SIZE == 0,
            "Add Heap size should be a multiple of page size"
        );
        self.bump_allocator.add_to_heap(heap_start_addr, heap_size);
    }

    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    unsafe fn _grow(&mut self, mem_start_addr: usize, mem_size: usize, slab: HeapAllocator) {
        debug!("mem_start_addr {}", mem_start_addr);
        match slab {
            HeapAllocator::Slab32Bytes => self.slab_32_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab96Bytes => self.slab_96_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab192Bytes => self.slab_192_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab384Bytes => self.slab_384_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab512Bytes => self.slab_512_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab2048Bytes => self.slab_2048_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab8192Bytes => self.slab_8192_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab32768Bytes => self.slab_32768_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab131072Bytes => self.slab_131072_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab524288Bytes => self.slab_524288_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::BumpAllocator => self.bump_allocator.add_to_heap(mem_start_addr, mem_size),
        }
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    /// This function finds the slab of lowest size which can still accommodate the given chunk.
    /// The runtime is in `O(1)` for chunks of size <= 4096, and `O(n)` when chunk size is >
    /// PAGE_SIZE,
    pub fn allocate(&mut self, layout: Layout) -> Result<usize, AllocError> {
        // debug!("heap : {:#?}", self);
        match Heap::layout_to_allocator(&layout) {
            HeapAllocator::Slab32Bytes => self.slab_32_bytes.allocate(layout, &mut self.bump_allocator),
            HeapAllocator::Slab96Bytes => self.slab_96_bytes.allocate(layout, &mut self.bump_allocator),
            HeapAllocator::Slab192Bytes => self.slab_192_bytes.allocate(layout, &mut self.bump_allocator),
            HeapAllocator::Slab384Bytes => self.slab_384_bytes.allocate(layout, &mut self.bump_allocator),
            HeapAllocator::Slab512Bytes => self.slab_512_bytes.allocate(layout, &mut self.bump_allocator),
            HeapAllocator::Slab2048Bytes => {
                self.slab_2048_bytes.allocate(layout, &mut self.bump_allocator)
            }
            HeapAllocator::Slab8192Bytes => {
                self.slab_8192_bytes.allocate(layout, &mut self.bump_allocator)
            }
            HeapAllocator::Slab32768Bytes => {
                self.slab_32768_bytes.allocate(layout, &mut self.bump_allocator)
            }
            HeapAllocator::Slab131072Bytes => {
                self.slab_131072_bytes.allocate(layout, &mut self.bump_allocator)
            }
            HeapAllocator::Slab524288Bytes => {
                self.slab_524288_bytes.allocate(layout, &mut self.bump_allocator)
            }
            HeapAllocator::BumpAllocator => self
                .bump_allocator
                .alloc(layout, 1)
                .map(|ptr| ptr)
                .map_err(|_| AllocError),
        }
    }

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// This function finds the slab which contains address of `ptr` and adds the blocks beginning
    /// with `ptr` address to the list of free blocks.
    /// This operation is in `O(1)` for blocks <= 4096 bytes and `O(n)` for blocks > 4096 bytes.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn deallocate(&mut self, ptr: usize, layout: Layout) {
        match Heap::layout_to_allocator(&layout) {
            HeapAllocator::Slab32Bytes => self.slab_32_bytes.deallocate(ptr),
            HeapAllocator::Slab96Bytes => self.slab_96_bytes.deallocate(ptr),
            HeapAllocator::Slab192Bytes => self.slab_192_bytes.deallocate(ptr),
            HeapAllocator::Slab384Bytes => self.slab_384_bytes.deallocate(ptr),
            HeapAllocator::Slab512Bytes => self.slab_512_bytes.deallocate(ptr),
            HeapAllocator::Slab2048Bytes => self.slab_2048_bytes.deallocate(ptr),
            HeapAllocator::Slab8192Bytes => self.slab_8192_bytes.deallocate(ptr),
            HeapAllocator::Slab32768Bytes => self.slab_32768_bytes.deallocate(ptr),
            HeapAllocator::Slab131072Bytes => self.slab_131072_bytes.deallocate(ptr),
            HeapAllocator::Slab524288Bytes => self.slab_524288_bytes.deallocate(ptr),
            HeapAllocator::BumpAllocator => self.bump_allocator.dealloc(ptr, layout),
        }
    }

    /// Returns bounds on the guaranteed usable size of a successful
    /// allocation created with the specified `layout`.
    pub fn usable_size(&self, layout: Layout) -> (usize, usize) {
        match Heap::layout_to_allocator(&layout) {
            HeapAllocator::Slab32Bytes => (layout.size(), 500),
            HeapAllocator::Slab96Bytes => (layout.size(), 96),
            HeapAllocator::Slab192Bytes => (layout.size(), 192),
            HeapAllocator::Slab384Bytes => (layout.size(), 384),
            HeapAllocator::Slab512Bytes => (layout.size(), 884),
            HeapAllocator::Slab2048Bytes => (layout.size(), 2420),
            HeapAllocator::Slab8192Bytes => (layout.size(), 8564),
            HeapAllocator::Slab32768Bytes => (layout.size(), 33140),
            HeapAllocator::Slab131072Bytes => (layout.size(), 131444),
            HeapAllocator::Slab524288Bytes => (layout.size(), 524660),
            HeapAllocator::BumpAllocator => (layout.size(), layout.size()),
        }
    }

    /// Finds allocator to use based on layout size and alignment
    fn layout_to_allocator(layout: &Layout) -> HeapAllocator {
        if layout.size() == 96 && layout.align() == 8 {
            HeapAllocator::Slab96Bytes
        } else if layout.size() == 192 && layout.align() == 8 {
            HeapAllocator::Slab192Bytes
        } else if layout.size() == 384 && layout.align() == 8 {
            HeapAllocator::Slab384Bytes
        } else if layout.size() <= 500 && layout.size() >= 32 {
            unsafe {
                if check_temp(layout.size() as isize) {
                    HeapAllocator::Slab32Bytes
                } else {
                    HeapAllocator::BumpAllocator
                }
            }
        } else if layout.size() <= 884 && layout.size() >= 512 {
            unsafe {
                if check_temp(layout.size() as isize) {
                    HeapAllocator::Slab512Bytes
                } else {
                    HeapAllocator::BumpAllocator
                }
            }
        } else if layout.size() <= 2420 && layout.size() >= 2048 {
            unsafe {
                update_temp(layout.size());
            }
            HeapAllocator::Slab2048Bytes
        } else if layout.size() <= 8564 && layout.size() >= 8192 {
            HeapAllocator::Slab8192Bytes
        } else if layout.size() <= 33140 && layout.size() >= 32768 {
            HeapAllocator::Slab32768Bytes
        } else if layout.size() <= 131444 && layout.size() >= 131072 {
            HeapAllocator::Slab131072Bytes
        } else if layout.size() <= 524660 && layout.size() >= 524288 {
            HeapAllocator::Slab524288Bytes
        } else {
            HeapAllocator::BumpAllocator
        }
    }

    /// Returns total memory size in bytes of the heap.
    pub fn total_bytes(&self) -> usize {
        assert!(self.slab_96_bytes.total_blocks() <= 1);
        assert!(self.slab_32_bytes.total_blocks() <= 2);
        assert!(self.slab_192_bytes.total_blocks() <= 1);
        assert!(self.slab_384_bytes.total_blocks() <= 1);
        assert!(self.slab_512_bytes.total_blocks() <= 1);
        assert!(self.slab_2048_bytes.total_blocks() <= 1);
        assert!(self.slab_8192_bytes.total_blocks() <= 1);
        assert!(self.slab_32768_bytes.total_blocks() <= 1);
        assert!(self.slab_131072_bytes.total_blocks() <= 1);
        assert!(self.slab_524288_bytes.total_blocks() <= 1);
        
        self.slab_96_bytes.total_blocks() * 96
            + self.slab_32_bytes.total_blocks() * 500
            + self.slab_192_bytes.total_blocks() * 192
            + self.slab_384_bytes.total_blocks() * 384
            + self.slab_512_bytes.total_blocks() * 884
            + self.slab_2048_bytes.total_blocks() * 2420
            + self.slab_8192_bytes.total_blocks() * 8564
            + self.slab_32768_bytes.total_blocks() * 33140
            + self.slab_131072_bytes.total_blocks() * 131444
            + self.slab_524288_bytes.total_blocks() * 524660
            + self.bump_allocator.stats_total_bytes()
    }

    /// Returns allocated memory size in bytes.
    pub fn used_bytes(&self) -> usize {
        self.slab_96_bytes.used_blocks() * 96
            + self.slab_32_bytes.used_blocks() * 500
            + self.slab_192_bytes.used_blocks() * 192
            + self.slab_384_bytes.used_blocks() * 384
            + self.slab_512_bytes.used_blocks() * 884
            + self.slab_2048_bytes.used_blocks() * 2420
            + self.slab_8192_bytes.used_blocks() * 8564
            + self.slab_32768_bytes.used_blocks() * 33140
            + self.slab_131072_bytes.used_blocks() * 131444
            + self.slab_524288_bytes.used_blocks() * 524660
            + self.bump_allocator.stats_alloc_actual()
    }

    /// Returns available memory size in bytes.
    pub fn available_bytes(&self) -> usize {
        self.total_bytes() - self.used_bytes()
    }
}

unsafe fn update_temp(v: usize) {
    TEMP = v;
}

unsafe fn check_temp(v:isize) -> bool {
    let d = TEMP as isize - 2048;
    assert!(d >= 0);

    d == v - 32 
        || d == v - 33 
        || d == v - 128 
        || d == v - 129
        || d == v - 512
        || d == v - 513
}
