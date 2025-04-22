use core::alloc::Layout;

use axlog::debug;

#[derive(Debug)]
pub struct BigHeap {
    start: usize,
    end: usize,
    used: usize,
    used_actual: usize,
}


impl BigHeap {
    pub fn new(start: usize, size: usize) -> Self {
        debug!("Big Heap new : start : {}, size : {}", start, size);
        Self {
            start,
            end: start + size,
            used: 0,
            used_actual: 0,
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
        debug!("layout size = {}", layout.size());
        if ptr + layout.size() == self.start + self.used {
            self.used -= layout.size();
            self.used_actual -= layout.size();
        }
    } 

    pub fn stats_total_bytes(&self) -> usize {
        self.end
    }

    pub fn stats_alloc_actual(&self) -> usize {
        self.used_actual
    }
}
