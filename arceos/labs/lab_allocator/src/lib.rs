//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use axlog::warn;
use core::ptr::NonNull;
use core::alloc::Layout;

use rlsf::Tlsf;


pub struct LabByteAllocator {
    inner: Tlsf<'static, u32, u32, 28, 32>,
    total_bytes: usize,
    used_bytes: usize,
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self {
            inner: Tlsf::new(),
            total_bytes: 0,
            used_bytes: 0,
        }
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        unsafe {
            let pool = core::slice::from_raw_parts_mut(start as *mut u8, size);
            self.inner
                .insert_free_block_ptr(NonNull::new(pool).unwrap())
                .unwrap();
        }
        self.total_bytes = size;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unsafe {
            let pool = core::slice::from_raw_parts_mut(start as *mut u8, size);
            self.inner
                .insert_free_block_ptr(NonNull::new(pool).unwrap())
                .ok_or(AllocError::InvalidParam)?;
        }
        self.total_bytes += size;
        Ok(())
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        warn!("alloc : {}", layout.size());
        let ptr = self.inner.allocate(layout).ok_or(AllocError::NoMemory)?;
        self.used_bytes += layout.size();
        Ok(ptr)
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        warn!("dealloc : {}", layout.size());
        unsafe { self.inner.deallocate(pos, layout.align()) }
        self.used_bytes -= layout.size();
    }

    fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    fn available_bytes(&self) -> usize {
        self.total_bytes - self.used_bytes
    }
}
