use core::alloc::Layout;

use axlog::{debug, info};

const VEC_MAX_SIZE: usize = 171864;
const HEAP_MAX_SIZE: usize = 131661824;
///
///
///
///
/// start                                  |----MAX_SIZE-----end
/// |-------------------|------------------|------------------|
/// bytes             free               vec_start
///
///
///

#[derive(Debug)]
pub struct BigHeap {
    start: usize,
    end: usize,
    used: usize,
    used_actual: usize,

    vec_start: usize,
    vec_used: usize,
}


impl BigHeap {
    pub fn new(start: usize, size: usize) -> Self {
        info!("Big Heap new : start : {}, size : {}", start, size);
        Self {
            start,
            end: start + size,
            used: 0,
            used_actual: 0,

            vec_start: start + HEAP_MAX_SIZE - VEC_MAX_SIZE,
            vec_used: 0,
        }
    }

    pub fn add_to_heap(&mut self, start: usize, size: usize) {
        debug!("Big Heap add : start : {}, size : {}", start, size);
        if self.end != start {
            unreachable!();
        }

        self.end += size;
    }
    
    pub fn alloc(&mut self, layout: Layout, flag : usize) -> Result<usize, ()> {
        if self.end - self.start < HEAP_MAX_SIZE {
            return Err(());
        }

        if flag == 1 && layout.align() == 8 {
            let start = self.vec_start + self.vec_used;
            if start + layout.size() > self.end {
                return Err(());
            }
            self.vec_used += layout.size();

            Ok(start)
        } else {
            let start = self.start + self.used;
            if start + layout.size() > self.vec_start {
                return Err(());
            }
            self.used += layout.size();
            if flag == 1 {
                self.used_actual += layout.size();
            }

            Ok(start)
        }
    }

    pub fn dealloc(&mut self, ptr: usize, layout: Layout) {
        if layout.align() != 8 {
            return;
        }

        assert_eq!(ptr, self.vec_start);
        self.vec_start += layout.size();
        self.vec_used -= layout.size();
    } 

    pub fn stats_total_bytes(&self) -> usize {
        self.end
    }

    pub fn stats_alloc_actual(&self) -> usize {
        self.used_actual + self.vec_used - self.vec_start
    }
}
