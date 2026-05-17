use crate::sys::{self, SceSysMemBlockTypes, SceSysMemPartitionId, SceUid};
use core::alloc::{GlobalAlloc, Layout};

/// An allocator that hooks directly into the PSP OS memory allocator.
struct SystemAlloc;

unsafe impl GlobalAlloc for SystemAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        use core::mem::size_of;
        #[allow(unused_imports)]
        use core::ptr;

        // Ensure valid alignment
        debug_assert!(layout.align().is_power_of_two());

        // Compute total size safely
        let size = match layout
            .size()
            .checked_add(size_of::<SceUid>())
            .and_then(|s| s.checked_add(layout.align()))
        {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        // Convert to u32 safely
        let size_u32 = match u32::try_from(size) {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };

        // Allocate from system
        let id = unsafe {
            sys::sceKernelAllocPartitionMemory(
                SceSysMemPartitionId::SceKernelPrimaryUserPartition,
                c"mem".as_ptr().cast(),
                SceSysMemBlockTypes::Low,
                size_u32,
                ptr::null_mut(),
            )
        };
        if id.0 < 0 {
            return ptr::null_mut();
        }

        let base_ptr = unsafe { sys::sceKernelGetBlockHeadAddr(id) as *mut u8 };
        if base_ptr.is_null() {
            return ptr::null_mut();
        }

        // Store Uid at the beginning (unaligned-safe)
        unsafe {
            (base_ptr as *mut SceUid).write_unaligned(id);
        };

        // Move past Uid
        let ptr = unsafe { base_ptr.add(size_of::<SceUid>()) };

        // Compute alignment padding safely
        let offset = unsafe { ptr.add(1).align_offset(layout.align()) };
        if offset == usize::MAX {
            return ptr::null_mut();
        }

        let align_padding = 1 + offset;

        // Store padding size in the last padding byte
        unsafe {
            ptr.add(align_padding - 1).write(align_padding as u8);
        };

        // Return aligned pointer
        unsafe { ptr.add(align_padding) }
    }
    #[inline(never)]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        use core::mem::size_of;
        #[allow(unused_imports)]
        use core::ptr;

        if ptr.is_null() {
            return;
        }

        // Read stored padding (last byte before aligned pointer)
        let align_padding = unsafe { ptr.sub(1).read() as usize };

        // Compute pointer to start of allocation (where Uid is stored)
        let uid_ptr = unsafe {
            ptr.sub(align_padding) // back to after Uid
                .sub(size_of::<SceUid>()) // back to Uid
        };

        // Read Uid (unaligned-safe)
        let id = unsafe { (uid_ptr as *const SceUid).read_unaligned() };

        // Free using the ID
        unsafe {
            sys::sceKernelFreePartitionMemory(id);
        };
    }
}

#[global_allocator]
static ALLOC: SystemAlloc = SystemAlloc;

#[alloc_error_handler]
fn aeh(layout: Layout) -> ! {
    panic!("Allocator error: {layout:?}")
}

const WORD_SIZE: usize = core::mem::size_of::<usize>();
const WORD_SIZE_SIGNED: isize = WORD_SIZE as isize;

#[unsafe(no_mangle)]
unsafe extern "C" fn memset(ptr: *mut u8, value: u32, num: usize) -> *mut u8 {
    let mut i = 0;
    let value = value as u8;

    while i < num {
        unsafe { *ptr.byte_add(i) = value };
        i += 1;
    }

    ptr
}

#[unsafe(no_mangle)]
#[allow(clippy::manual_memcpy)]
unsafe extern "C" fn memcpy(
    dst: *mut u8,
    src: *const u8,
    num: isize,
) -> *mut u8 {
    let num = num as usize;

    let mut i = 0usize;
    if num >= WORD_SIZE {
        // Machine word copy
        while i < num - WORD_SIZE {
            unsafe {
                let src: *const usize = src.add(i).cast();
                let dst: *mut usize = dst.add(i).cast();
                dst.write_unaligned(src.read_unaligned());
            };
            i += WORD_SIZE;
        }
    }

    while i < num {
        unsafe {
            *dst.add(i) = *src.add(i);
        };
        i += 1;
    }

    dst
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memmove(
    dst: *mut u8,
    src: *const u8,
    num: isize,
) -> *mut u8 {
    if dst.addr() < src.addr() {
        let num = num as usize;

        let mut i = 0usize;
        if num >= WORD_SIZE {
            // Machine word move
            while i < num - WORD_SIZE {
                unsafe {
                    let src: *const usize = src.add(i).cast();
                    let dst: *mut usize = dst.add(i).cast();
                    dst.write_unaligned(src.read_unaligned());
                };
                i += WORD_SIZE;
            }
        }

        while i < num {
            unsafe {
                *dst.add(i) = *src.add(i);
            };
            i += 1;
        }
    } else {
        let mut i = num;
        // Machine word move
        while i >= WORD_SIZE_SIGNED {
            unsafe {
                let src: *const usize = src.offset(i).cast();
                let dst: *mut usize = dst.offset(i).cast();
                dst.write_unaligned(src.read_unaligned());
            };
            i -= WORD_SIZE_SIGNED;
        }

        while i > 0 {
            unsafe {
                *dst.offset(i) = *src.offset(i);
            };
            i -= 1;
        }
    }

    dst
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memcmp(lhs: *const u8, rhs: *const u8, num: usize) -> i32 {
    let mut i = 0usize;
    if num >= WORD_SIZE {
        // Machine word compare
        while i < num - WORD_SIZE {
            unsafe {
                let lhs: *const usize = lhs.add(i).cast();
                let rhs: *const usize = rhs.add(i).cast();
                if lhs.read_unaligned() != rhs.read_unaligned() {
                    break;
                }
            };
            i += WORD_SIZE;
        }
    }

    while i < num {
        let a = unsafe { *lhs.add(i) };
        let b = unsafe { *rhs.add(i) };
        if a != b {
            return i32::from(a) - i32::from(b);
        }
        i += 1;
    }

    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn strlen(s: *mut u8) -> usize {
    let mut len = 0usize;

    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }

    len
}
