//! Panic support for the PSP.

//use crate::sys;

use core::{any::Any, mem::ManuallyDrop, /*, panic::PanicPayload as BoxMeUp*/};

use alloc::{boxed::Box /*, string::String*/};

#[allow(improper_ctypes)]
unsafe extern "C" {
    #[rustc_std_internal_symbol]
    fn __rust_panic_cleanup(
        payload: *mut u8,
    ) -> *mut (dyn Any + Send + 'static);
}

/// Invoke a closure, capturing the cause of an unwinding panic if one occurs.
#[inline(never)]
#[doc(hidden)]
pub fn catch_unwind<R, F: FnOnce() -> R>(
    f: F,
) -> Result<R, Box<dyn Any + Send>> {
    // This whole function is directly lifted out of rustc. See comments there
    // for an explanation of how this actually works.

    union Data<F, R> {
        f: ManuallyDrop<F>,
        r: ManuallyDrop<R>,
        p: ManuallyDrop<Box<dyn Any + Send>>,
    }

    let mut data = Data {
        f: ManuallyDrop::new(f),
    };

    let data_ptr: *mut u8 = (&raw mut data).cast();

    return unsafe {
        if core::intrinsics::catch_unwind(
            do_call::<F, R>,
            data_ptr,
            do_catch::<F, R>,
        ) == 0
        {
            Ok(ManuallyDrop::into_inner(data.r))
        } else {
            Err(ManuallyDrop::into_inner(data.p))
        }
    };

    #[cold]
    unsafe fn cleanup(payload: *mut u8) -> Box<dyn Any + Send + 'static> {
        let obj = unsafe { Box::from_raw(__rust_panic_cleanup(payload)) };
        //update_panic_count(-1);
        obj
    }

    #[inline]
    fn do_call<F: FnOnce() -> R, R>(data: *mut u8) {
        unsafe {
            let data: &mut Data<F, R> = &mut *(data.cast());
            let f = ManuallyDrop::take(&mut data.f);
            data.r = ManuallyDrop::new(f());
        }
    }

    #[inline]
    fn do_catch<F: FnOnce() -> R, R>(data: *mut u8, payload: *mut u8) {
        let data: &mut Data<F, R> = unsafe { &mut *(data.cast()) };
        let obj = unsafe { cleanup(payload) };
        data.p = ManuallyDrop::new(obj);
    }
}
