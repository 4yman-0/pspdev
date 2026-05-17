use crate::error::{NativeResult, native_error, native_result};
use core::ffi::c_void;
use core::ptr;
use psp_sys::sys;

pub struct InterruptCtrlState(pub(crate) u32);

impl InterruptCtrlState {
    pub fn interrupts_suspended(&self) -> bool {
        unsafe { sys::sceKernelIsCpuIntrSuspended(self.0) != 1 }
    }
}

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

/// The time is only returned in seconds.
pub fn time_since_epoch() -> core::time::Duration {
    let mut secs = 0i32;
    unsafe {
        let _ = sys::sceKernelLibcTime(&mut secs);
    }
    core::time::Duration::from_secs(secs as u64)
}

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

pub fn exit_game() {
    unsafe {
        sys::sceKernelExitGame();
    }
}

pub fn load_exec(
    path: &core::ffi::CStr,
    param: Option<&sys::SceKernelLoadExecParam>,
) -> NativeResult<()> {
    native_error(unsafe {
        // SAFETY: [`sceKernelLoadExec`] interprets null as no params
        sys::sceKernelLoadExec(
            path.as_ptr().cast(),
            if let Some(param) = param {
                (&raw const *param) as *mut _
            } else {
                ptr::null_mut()
            },
        )
    })
}

pub struct MemoryBlock {
    uid: sys::SceUid,
}

impl MemoryBlock {
    pub fn alloc_from_partition(
        partition: sys::SceSysMemPartitionId,
        name: &core::ffi::CStr,
        block_type: sys::SceSysMemBlockTypes,
        size: usize,
        address: Option<usize>,
    ) -> NativeResult<Self> {
        native_result(
            (unsafe {
                sys::sceKernelAllocPartitionMemory(
                    partition,
                    name.as_ptr().cast(),
                    block_type,
                    size as u32,
                    if let Some(addr) = address {
                        addr as *mut c_void
                    } else {
                        ptr::null_mut()
                    },
                )
            })
            .0,
        )
        .map(|uid| Self {
            uid: (uid as i32).into(),
        })
    }

    pub fn head_addr(&self) -> *const c_void {
        unsafe { sys::sceKernelGetBlockHeadAddr(self.uid) as *const c_void }
    }

    pub fn head_addr_mut(&mut self) -> *mut c_void {
        unsafe { sys::sceKernelGetBlockHeadAddr(self.uid) }
    }

    pub fn free(self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelFreePartitionMemory(self.uid) })
    }
}

pub fn total_free_memory() -> usize {
    unsafe { sys::sceKernelTotalFreeMemSize() }
}

pub fn largest_free_memory_block() -> usize {
    unsafe { sys::sceKernelMaxFreeMemSize() }
}

pub fn firmware_version() -> u32 {
    unsafe { sys::sceKernelDevkitVersion() }
}

pub fn set_sdk_version(version: u32) -> NativeResult<()> {
    native_error(unsafe { sys::sceKernelSetCompiledSdkVersion(version) })
}

pub fn sdk_version() -> u32 {
    unsafe { sys::sceKernelGetCompiledSdkVersion() }
}

pub struct ModuleUid {
    uid: sys::SceUid,
}

impl ModuleUid {
    pub fn load(
        path: &core::ffi::CStr,
        options: Option<&sys::SceKernelLMOption>,
    ) -> NativeResult<Self> {
        native_result(
            (unsafe {
                sys::sceKernelLoadModule(
                    path.as_ptr().cast(),
                    0,
                    if let Some(option) = options {
                        (&raw const *option) as *mut _
                    } else {
                        ptr::null_mut()
                    },
                )
            })
            .0,
        )
        .map(|uid| Self {
            uid: (uid as i32).into(),
        })
    }
    pub fn load_from_ms(
        path: &core::ffi::CStr,
        options: Option<&sys::SceKernelLMOption>,
    ) -> NativeResult<Self> {
        native_result(
            (unsafe {
                sys::sceKernelLoadModuleMs(
                    path.as_ptr().cast(),
                    0,
                    if let Some(option) = options {
                        (&raw const *option) as *mut _
                    } else {
                        ptr::null_mut()
                    },
                )
            })
            .0,
        )
        .map(|uid| Self {
            uid: (uid as i32).into(),
        })
    }
    pub fn load_from_file(
        file: &mut crate::fs::File,
        options: Option<&sys::SceKernelLMOption>,
    ) -> NativeResult<Self> {
        native_result(
            (unsafe {
                sys::sceKernelLoadModuleByID(
                    file.uid,
                    0,
                    if let Some(option) = options {
                        (&raw const *option) as *mut _
                    } else {
                        ptr::null_mut()
                    },
                )
            })
            .0,
        )
        .map(|uid| Self {
            uid: (uid as i32).into(),
        })
    }
    // TODO: sceKernelLoadModuleBufferUsbWlan
    pub fn start<T: ?Sized>(
        &mut self,
        arg: &mut T,
        options: Option<&sys::SceKernelSMOption>,
    ) -> NativeResult<i32> {
        let mut start_status = 0i32;
        native_error(unsafe {
            sys::sceKernelStartModule(
                self.uid,
                core::mem::size_of_val(arg),
                (&raw mut *arg).cast(),
                &mut start_status,
                if let Some(option) = options {
                    (&raw const *option) as *mut _
                } else {
                    ptr::null_mut()
                },
            )
        })
        .map(|_| start_status)
    }
    pub fn stop<T: ?Sized>(
        &mut self,
        arg: &mut T,
        options: Option<&sys::SceKernelSMOption>,
    ) -> NativeResult<i32> {
        let mut start_status = 0i32;
        native_error(unsafe {
            sys::sceKernelStopModule(
                self.uid,
                core::mem::size_of_val(arg),
                (&raw mut *arg).cast(),
                &mut start_status,
                if let Some(option) = options {
                    (&raw const *option) as *mut _
                } else {
                    ptr::null_mut()
                },
            )
        })
        .map(|_| start_status)
    }
    pub fn unload(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelUnloadModule(self.uid) })
    }
    pub fn info(&self) -> NativeResult<sys::SceKernelModuleInfo> {
        use core::mem::MaybeUninit as MayUninit;
        let mut info = MayUninit::<sys::SceKernelModuleInfo>::uninit();
        native_error(unsafe {
            sys::sceKernelQueryModuleInfo(self.uid, info.assume_init_mut())
        })?;
        Ok(unsafe { info.assume_init() })
    }
}

pub fn stop_unload_current_module<T: ?Sized>(
    unknown: i32,
    arg: &mut T,
) -> NativeResult<()> {
    native_error(unsafe {
        sys::sceKernelSelfStopUnloadModule(
            unknown,
            core::mem::size_of_val(arg),
            (&raw mut *arg).cast(),
        )
    })
}

pub fn stop_unload_self_module<T: ?Sized>(
    arg: &mut T,
    options: Option<&sys::SceKernelSMOption>,
) -> NativeResult<i32> {
    let mut start_status = 0i32;
    native_error(unsafe {
        sys::sceKernelStopUnloadSelfModule(
            core::mem::size_of_val(arg),
            (&raw mut *arg).cast(),
            &mut start_status,
            if let Some(option) = options {
                (&raw const *option) as *mut _
            } else {
                ptr::null_mut()
            },
        )
    })
    .map(|_| start_status)
}

// TODO: sceKernelGetModuleIdList
