use crate::error::{NativeResult, native_error};
use psp_sys::sys;

pub struct InterruptCtrlState(pub(crate) u32);

pub fn suspend_interrupts() -> InterruptCtrlState {
    InterruptCtrlState(unsafe { sys::sceKernelCpuSuspendIntr() })
}

pub fn resume_interrupts(state: InterruptCtrlState) {
    unsafe {
        sys::sceKernelCpuResumeIntr(state.0);
    };
}
pub fn resume_interrupts_with_sync(state: InterruptCtrlState) {
    unsafe {
        sys::sceKernelCpuResumeIntrWithSync(state.0);
    };
}

pub fn interrupt_enabled() -> bool {
    unsafe { sys::sceKernelIsCpuIntrEnable() == 1 }
}

pub fn time_since_epoch() -> core::time::Duration {
    let mut secs = 0i32;
    unsafe {
        let _ = sys::sceKernelLibcTime(&mut secs);
    }
    core::time::Duration::from_secs(secs as u64)
}

use core::ptr;

pub fn data_cache_writeback<T: ?Sized>(sized: &T) {
    unsafe {
        sys::sceKernelDcacheWritebackRange(
            ptr::from_ref::<T>(sized).cast(),
            size_of_val(sized) as u32,
        )
    }
}

pub fn data_cache_writeback_invalidate<T: ?Sized>(sized: &T) {
    unsafe {
        sys::sceKernelDcacheWritebackInvalidateRange(
            ptr::from_ref::<T>(sized).cast(),
            size_of_val(sized) as u32,
        )
    }
}

pub fn data_cache_invalidate<T: ?Sized>(sized: &T) {
    unsafe {
        sys::sceKernelDcacheInvalidateRange(
            ptr::from_ref::<T>(sized).cast(),
            size_of_val(sized) as u32,
        )
    }
}

pub fn instruction_cache_invalidate<T: ?Sized>(sized: &T) {
    unsafe {
        sys::sceKernelIcacheInvalidateRange(
            ptr::from_ref::<T>(sized).cast(),
            size_of_val(sized) as u32,
        )
    }
}

// TODO: other *cache function, but those are pretty trivial

pub fn utility_load_module(module: sys::Module) -> NativeResult<()> {
    native_error(unsafe { sys::sceUtilityLoadModule(module) })
}

pub fn utility_unload_module(module: sys::Module) -> NativeResult<()> {
    native_error(unsafe { sys::sceUtilityUnloadModule(module) })
}
