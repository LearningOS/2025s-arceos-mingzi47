#![no_std]

use core::{cmp::max, ptr::NonNull};

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
///

///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    base: usize,
    total_size: usize,
    byte_ptr_end: usize,   // 开区间
    page_ptr_start: usize, // 开区间
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            base: 0,
            total_size: 0,
            byte_ptr_end: 0,
            page_ptr_start: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());

        self.base = start;
        self.total_size = size;
        self.page_ptr_start = size;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> allocator::AllocResult {
        return Err(AllocError::NoMemory); // unsupported
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        if self.available_bytes() < layout.size() {
            return Err(AllocError::NoMemory);
        }

        let ptr = self.base + self.byte_ptr_end;
        self.byte_ptr_end += layout.size();

        unsafe {
            Ok(NonNull::new_unchecked(ptr as *mut u8))
        }
    }

    fn dealloc(&mut self, _pos: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        return // unsupported
    }

    fn total_bytes(&self) -> usize {
        self.total_size
    }

    fn available_bytes(&self) -> usize {
        max(self.page_ptr_start - self.byte_ptr_end + 1, 0)
    }

    fn used_bytes(&self) -> usize {
        self.byte_ptr_end
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;
    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }

        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        let req_size = num_pages * PAGE_SIZE;
        if self.available_pages() < req_size {
            return Err(AllocError::NoMemory);
        }


        let ptr = self.page_ptr_start - req_size + 1;
        self.page_ptr_start -= req_size;

        Ok(ptr)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        return // unsupported
    }

    fn total_pages(&self) -> usize {
        self.total_size / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        let available_pages_size = self.page_ptr_start - self.byte_ptr_end + 1;

        available_pages_size / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        let used_pages_size = self.total_size - self.page_ptr_start;

        used_pages_size / PAGE_SIZE
    }
}
