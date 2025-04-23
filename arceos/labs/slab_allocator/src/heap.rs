use core::alloc::Layout;

use axlog::{debug, info};

const VEC_MAX_SIZE: usize = 90000;

#[derive(Debug)]
pub struct BigHeap {
    start: usize,
    end: usize,
    used: usize,
    used_actual: usize,

    vec_start: usize,
    vec_used: usize,
    vec_end: usize,
}


impl BigHeap {
    pub fn new(start: usize, size: usize) -> Self {
        debug!("Big Heap new : start : {}, size : {}", start, size);
        Self {
            start,
            end: start + size,
            used: 0,
            used_actual: 0,

            vec_start: 0,
            vec_used: 0,
            vec_end: 0,
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
        if flag == 1 && layout.align() == 8 {
            let d = self.vec_end - self.vec_start;
            if d < VEC_MAX_SIZE {
                if self.end - self.used < VEC_MAX_SIZE - d {
                    return Err(());
                }

                self.vec_start = self.start + self.used;
                self.vec_end = self.vec_start + VEC_MAX_SIZE;
                self.used += VEC_MAX_SIZE;
            }


            let ret = self.vec_start + self.vec_used;
            self.vec_used += layout.size();
            info!("alloc layout size = {}, align = {}", layout.size(), layout.align());
            assert!(self.vec_start + self.vec_used <= self.vec_end);

            return Ok(ret);
        }

        let ret = self.start + self.used;
        if ret + layout.size() > self.end {
            debug!("not can big heap alloc ptr = {}, size = {}, end = {}",
                ret,
                layout.size(),
                self.end,
            );
            return Err(());
        }
        // debug!("can big heap alloc ptr = {}, used = {}, end = {}",
        //     ret,
        //     self.start + self.used,
        //     self.end,
        // );

        self.used += layout.size();
        if flag == 1 {
            self.used_actual += layout.size();
        }

        Ok(ret)
    }

    pub fn dealloc(&mut self, ptr: usize, layout: Layout) {
        info!("delloc layout size = {}, align = {}", layout.size(), layout.align());

        if layout.align() == 8 {
            assert_eq!(self.vec_start + self.vec_used, ptr + layout.size());
            self.vec_used -= layout.size();
        }
    } 

    pub fn stats_total_bytes(&self) -> usize {
        self.end
    }

    pub fn stats_alloc_actual(&self) -> usize {
        self.used_actual
    }
}
