//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use axlog::debug;
use core::alloc::Layout;
use core::ptr::NonNull;

extern crate slab_allocator;
use slab_allocator::Heap;

pub struct LabByteAllocator {
    inner: Option<Heap>,
}

impl LabByteAllocator {
    /// Creates a new empty `SlabByteAllocator`.
    pub const fn new() -> Self {
        Self { inner: None }
    }

    fn inner_mut(&mut self) -> &mut Heap {
        self.inner.as_mut().unwrap()
    }

    fn inner(&self) -> &Heap {
        self.inner.as_ref().unwrap()
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.inner = unsafe { Some(Heap::new(start, size)) };
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        debug!("add memory : start : {}, size: {}", start, size);
        unsafe {
            self.inner_mut().add_memory(start, size);
        }
        Ok(())
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        debug!("alloc : {}, algin : {}", layout.size(), layout.align());
        self.inner_mut()
            .allocate(layout)
            .map(|addr| unsafe { NonNull::new_unchecked(addr as *mut u8) })
            .map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        debug!("dealloc : {}", layout.size());
        unsafe { self.inner_mut().deallocate(pos.as_ptr() as usize, layout) }
    }

    fn total_bytes(&self) -> usize {
        debug!("total_bytes : {}", self.inner().total_bytes());
        self.inner().total_bytes()
    }

    fn used_bytes(&self) -> usize {
        debug!("used_bytes : {}", self.inner().used_bytes());
        self.inner().used_bytes()
    }

    fn available_bytes(&self) -> usize {
        debug!("available_bytes : {}", self.inner().available_bytes());
        self.inner().available_bytes()
    }
}
