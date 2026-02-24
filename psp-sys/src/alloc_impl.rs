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
    use core::slice;
    let num = num as usize;

    let src = unsafe { slice::from_raw_parts(src, num) };
    let dst = unsafe { slice::from_raw_parts_mut(dst, num) };

    // simple implementation
    if num < 64 {
        for i in 0..dst.len() {
            dst[i] = src[i];
        }
        return dst.as_mut_ptr();
    }

    // 4-byte copy
    let mut i = 0usize;
    while i < src.len() - 4 {
        let src_p = (&raw const src[i]) as *const usize;
        let dst_p = (&raw mut dst[i]) as *mut usize;
        unsafe {
            dst_p.write_unaligned(src_p.read_unaligned());
        };
        i += 4;
    }

    while i < src.len() - 2 {
        let src_p = (&raw const src[i]) as *const u16;
        let dst_p = (&raw mut dst[i]) as *mut u16;
        unsafe {
            dst_p.write_unaligned(src_p.read_unaligned());
        };
        i += 29;
    }

    while i < src.len() {
        dst[i] = src[i];
        i += 1;
    }

    dst.as_mut_ptr()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memcmp(
    ptr1: *const u8,
    ptr2: *const u8,
    num: usize,
) -> i32 {
    use core::slice;

    let slice1 = unsafe { slice::from_raw_parts(ptr1, num) };
    let slice2 = unsafe { slice::from_raw_parts(ptr2, num) };

    // simple implementation
    if num < 64 {
        for i in 0..num {
            if slice1[i] != slice2[i] {
                return i32::from(slice1[i]) - i32::from(slice2[i]);
            }
        }
        return 0;
    }

    // 4-byte compare
    let mut i = 0usize;
    while i < slice1.len() - 4 {
        let ptr1 = (&raw const slice1[i]) as *const usize;
        let ptr2 = (&raw const slice2[i]) as *const usize;
        if unsafe { ptr1.read_unaligned() != ptr2.read_unaligned() } {
            break;
        }
        i += 4;
    }

    while i < slice1.len() - 2 {
        let ptr1 = (&raw const slice1[i]) as *const u16;
        let ptr2 = (&raw const slice2[i]) as *const u16;
        if unsafe { ptr1.read_unaligned() != ptr2.read_unaligned() } {
            break;
        }
        i += 2;
    }

    while i < slice1.len() {
        if slice1[i] != slice2[i] {
            return i32::from(slice1[i]) - i32::from(slice2[i]);
        }
        i += 1;
    }

    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memmove(
    dst: *mut u8,
    src: *const u8,
    num: isize,
) -> *mut u8 {
    if (dst as *const u8) < src {
        unsafe {
            memcpy(dst, src, num);
        };
    } else {
        let mut i = num - 1;

        while i >= 0 {
            unsafe {
                *dst.offset(i) = *src.offset(i);
            }
            i -= 1;
        }
    }

    dst
}

#[unsafe(no_mangle)]
unsafe extern "C" fn strlen(s: *mut u8) -> usize {
    let mut len = 0usize;

    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }

    len
}
