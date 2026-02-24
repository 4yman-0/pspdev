use core::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
pub type VramAllocatorResult<T> = Result<T, VramAllocatorError>;

#[derive(Debug, Clone)]
pub enum VramAllocatorError {
    OutOfVram,
}

impl core::fmt::Display for VramAllocatorError {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), core::fmt::Error> {
        write!(f, "VRAM allocator error: {self:?}")
    }
}
impl core::error::Error for VramAllocatorError {}

pub fn vram_size() -> usize {
    (unsafe { psp_sys::sys::sceGeEdramGetSize() }) as usize
}
fn vram_base() -> usize {
    (unsafe { psp_sys::sys::sceGeEdramGetAddr() }) as usize
}

pub struct VramAllocator {
    head: AtomicUsize,
}

impl VramAllocator {
    pub(crate) const fn default() -> Self {
        Self {
            head: AtomicUsize::new(0),
        }
    }

    /// # Safety
    /// The allocator acts like it can *just* give away VRAM pointers
    pub unsafe fn allocate(
        &mut self,
        size: usize,
    ) -> VramAllocatorResult<*mut u8> {
        let head = self.head.load(AtomicOrdering::Relaxed);
        if head + size >= vram_size() {
            return Err(VramAllocatorError::OutOfVram);
        }
        let ptr = (vram_base() + head) as *mut u8;
        self.head.store(head + size, AtomicOrdering::Relaxed);
        Ok(ptr)
    }

    /// # Note
    /// All VRAM pointers will become dangling
    pub fn deallocate_all(&mut self) {
        self.head.store(0, AtomicOrdering::Relaxed);
    }

    pub fn memory_used(&self) -> usize {
        self.head.load(AtomicOrdering::Relaxed)
    }
}
