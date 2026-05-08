use alloc::{vec, vec::Vec};
use psp_sys::sys;

pub fn vram_size() -> usize {
    (unsafe { psp_sys::sys::sceGeEdramGetSize() }) as usize
}

fn vram_base() -> *mut u8 {
    unsafe { psp_sys::sys::sceGeEdramGetAddr().cast() }
}

fn align_up(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}

#[derive(Clone, Debug)]
struct VramNode {
    addr: *mut u8,
    size: usize,
}

impl VramNode {
    fn new(addr: *mut u8, size: usize) -> Self {
        Self { addr, size }
    }

    fn start(&self) -> usize {
        self.addr as usize
    }

    fn end(&self) -> usize {
        self.start() + self.size
    }
}

struct VramAllocator {
    free: Vec<VramNode>,
}

impl VramAllocator {
    unsafe fn init() -> Self {
        Self {
            free: vec![VramNode::new(vram_base(), vram_size())],
        }
    }

    unsafe fn alloc(&mut self, size: usize, align: usize) -> Option<VramChunk> {
        for i in 0..self.free.len() {
            let node = self.free[i].clone();

            let alloc_start = align_up(node.start(), align);

            let alloc_end = alloc_start.checked_add(size)?;

            if alloc_end > node.end() {
                continue;
            }

            // Remove current free node
            self.free.remove(i);

            // Left remainder
            if alloc_start > node.start() {
                self.free
                    .push(VramNode::new(node.addr, alloc_start - node.start()));
            }

            // Right remainder
            if alloc_end < node.end() {
                self.free.push(VramNode::new(
                    alloc_end as *mut u8,
                    node.end() - alloc_end,
                ));
            }

            self.coalesce();

            return Some(VramChunk::new(alloc_start as *mut u8, size));
        }

        None
    }

    unsafe fn dealloc(&mut self, chunk: &VramChunk) {
        self.free.push(VramNode::new(chunk.addr, chunk.size));

        self.coalesce();
    }

    fn coalesce(&mut self) {
        self.free.sort_by_key(|n| n.start());

        let mut merged: Vec<VramNode> = Vec::new();

        for node in self.free.drain(..) {
            if let Some(last) = merged.last_mut() {
                if last.end() == node.start() {
                    last.size += node.size;
                    continue;
                }
            }

            merged.push(node);
        }

        self.free = merged;
    }
}

static mut VRAM_ALLOCATOR: Option<VramAllocator> = None;

pub struct VramChunk {
    addr: *mut u8,
    size: usize,
}

impl VramChunk {
    fn new(addr: *mut u8, size: usize) -> Self {
        Self { addr, size }
    }

    pub fn alloc(size: usize, align: usize) -> Self {
        #[allow(static_mut_refs)]
        unsafe {
            if VRAM_ALLOCATOR.is_none() {
                VRAM_ALLOCATOR = Some(VramAllocator::init());
            }

            VRAM_ALLOCATOR
                .as_mut()
                .unwrap()
                .alloc(size, align)
                .expect("VRAM allocation failed")
        }
    }

    fn dealloc(&mut self) {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(alloc) = VRAM_ALLOCATOR.as_mut() {
                alloc.dealloc(self);
            }
        }
    }
}

impl Drop for VramChunk {
    fn drop(&mut self) {
        self.dealloc();
    }
}

impl core::ops::Deref for VramChunk {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.addr, self.size) }
    }
}

impl core::ops::DerefMut for VramChunk {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.addr, self.size) }
    }
}
