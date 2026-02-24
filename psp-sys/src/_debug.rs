//! Debug support.
//!
//! You should use the `dprintln!` and `dprint!` macros.

// SAFETY: no
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]

use crate::sys;

/// Like `print!`, but prints using firmware functions.
#[macro_export]
macro_rules! dprint {
    ($($arg:tt)*) => {{
        $crate::debug::print_args(core::format_args!($($arg)*))
    }}
}

#[doc(hidden)]
pub fn print_args(arg: core::fmt::Arguments<'_>) {
    if let Some(string) = arg.as_str() {
        unsafe {
            sys::sceIoWrite(
                sys::SceUid(1), // stdout
                string.as_ptr().cast(),
                string.len(),
            );
        };
    } else {
        let string = alloc::fmt::format(arg);
        unsafe {
            sys::sceIoWrite(
                sys::SceUid(1), // stdout
                string.as_str().as_ptr().cast(),
                string.len(),
            );
        };
    }
    /*let arg = alloc::ffi::CString::new(
        arg.as_str().unwrap_or("<No Message>")
    ).unwrap();
    unsafe {
        sys::sceKernelPrintf(arg.as_ptr().cast());
    };*/
}
