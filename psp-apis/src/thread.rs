use crate::error::{
    NativeResult, /*, NativeResult*/
    native_error, native_result,
};
use alloc::boxed::Box;
use core::ffi;
use core::mem::MaybeUninit;
use psp_sys::{dprint, sys};

pub fn exit(status: i32) -> i32 {
    unsafe { sys::sceKernelExitThread(status) }
}

pub fn exit_and_delete(status: i32) -> i32 {
    unsafe { sys::sceKernelExitDeleteThread(status) }
}

pub fn sleep(duration: core::time::Duration) -> NativeResult<()> {
    native_error(unsafe {
        sys::sceKernelDelayThread(duration.as_micros() as u32)
    })
}

pub fn sleep_sysclocks(sysclocks: &sys::SceKernelSysClock) -> NativeResult<()> {
    native_error(unsafe {
        sys::sceKernelDelaySysClockThread((&raw const *sysclocks) as *mut _)
    })
}

pub fn sleep_sysclocks_with_callback(
    sysclocks: &sys::SceKernelSysClock,
) -> NativeResult<()> {
    native_error(unsafe {
        sys::sceKernelDelaySysClockThreadCB((&raw const *sysclocks) as *mut _)
    })
}

pub fn make_sleeping() -> NativeResult<()> {
    native_error(unsafe { sys::sceKernelSleepThread() })
}

pub fn make_sleeping_with_callback() -> NativeResult<()> {
    native_error(unsafe { sys::sceKernelSleepThreadCB() })
}

pub fn set_attribute(attr: sys::ThreadAttributes) -> NativeResult<()> {
    native_error(unsafe { sys::sceKernelChangeCurrentThreadAttr(0, attr) })
}

pub fn rotate_ready_queue(priority: i32) -> NativeResult<()> {
    native_error(unsafe { sys::sceKernelRotateThreadReadyQueue(priority) })
}

pub fn current() -> Thread {
    unsafe {
        Thread::from_uid(sys::SceUid(sys::sceKernelGetThreadId()))
            .unwrap()
            .unwrap()
    }
}

pub fn current_priority() -> i32 {
    unsafe { sys::sceKernelGetThreadCurrentPriority() }
}

pub fn check_current() -> i32 {
    unsafe { sys::sceKernelCheckThreadStack() }
}

pub(crate) fn uid_threadman_type(
    uid: sys::SceUid,
) -> NativeResult<sys::SceKernelIdListType> {
    unsafe {
        let id_type = sys::sceKernelGetThreadmanIdType(uid);
        let result = (id_type as u32).cast_signed();
        if result < 0 {
            Err(result.try_into().unwrap())
        } else {
            Ok(id_type)
        }
    }
}

pub struct Thread(sys::SceUid);

impl Thread {
    // TODO: &mut _?
    pub fn from_uid(uid: sys::SceUid) -> NativeResult<Option<Self>> {
        use sys::SceKernelIdListType as ThreadmanType;
        Ok(match uid_threadman_type(uid)? {
            ThreadmanType::Thread
            | ThreadmanType::SleepThread
            | ThreadmanType::DelayThread
            | ThreadmanType::SuspendThread
            | ThreadmanType::DormantThread => Some(Self(uid)),
            _ => None,
        })
    }

    pub fn new(
        name: &ffi::CStr,
        entry: sys::SceKernelThreadEntry,
        priority: i32,
        stack_size: usize,
        attr: sys::ThreadAttributes,
        options: Option<&sys::SceKernelThreadOptParam>,
    ) -> NativeResult<Self> {
        unsafe {
            Ok(Self::from_uid(sys::sceKernelCreateThread(
                name.as_ptr().cast(),
                entry,
                priority,
                stack_size as i32,
                attr,
                if let Some(options) = options {
                    (&raw const *options) as *mut _
                } else {
                    core::ptr::null_mut()
                },
            ))?
            .unwrap())
        }
    }

    /// # Safety
    /// Raw pointer gymnastics
    pub unsafe fn start<T: ?Sized + Send>(
        &mut self,
        arg: Box<T>,
    ) -> NativeResult<()> {
        if core::mem::size_of_val(arg.as_ref()) == 0 {
            dprint!("Warning: the OS does not pass ZST arguments");
        }
        native_error(unsafe {
            sys::sceKernelStartThread(
                self.0,
                size_of_val(arg.as_ref()),
                Box::into_raw(arg).cast(),
            )
        })?;
        Ok(())
    }

    pub fn threadman_type(&self) -> NativeResult<sys::SceKernelIdListType> {
        uid_threadman_type(self.0)
    }

    pub fn wake_up(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelWakeupThread(self.0) })
    }

    pub fn cancel_wake_up(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelCancelWakeupThread(self.0) })
    }

    pub fn suspend(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelSuspendThread(self.0) })
    }

    pub fn resume(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelResumeThread(self.0) })
    }

    pub fn block_on_end(
        &mut self,
        timeout: core::time::Duration,
    ) -> NativeResult<()> {
        let micros = timeout.as_micros() as u32;
        native_error(unsafe {
            // SAFETY: It's not mutable, trust me!
            sys::sceKernelWaitThreadEnd(self.0, (&raw const micros) as *mut _)
        })
    }

    pub fn block_on_end_with_callback(
        &mut self,
        timeout: core::time::Duration,
    ) -> NativeResult<()> {
        let micros = timeout.as_micros() as u32;
        native_error(unsafe {
            // SAFETY: It's not mutable, trust me!
            sys::sceKernelWaitThreadEndCB(self.0, (&raw const micros) as *mut _)
        })
    }

    pub fn set_priority(&mut self, priority: i32) -> NativeResult<()> {
        native_error(unsafe {
            sys::sceKernelChangeThreadPriority(self.0, priority)
        })
    }

    pub fn release_waiting(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelReleaseWaitThread(self.0) })
    }

    pub fn exit_status(&self) -> NativeResult<u32> {
        native_result(unsafe { sys::sceKernelGetThreadExitStatus(self.0) })
    }

    pub fn has_exited(&self) -> NativeResult<bool> {
        Ok(match self.exit_status() {
            Ok(_) => true,
            Err(err) => {
                if err.inner().cast_unsigned() == 0x800201A4 {
                    false
                } else {
                    return Err(err);
                }
            }
        })
    }

    pub fn free_stack_size(&self) -> usize {
        (unsafe { sys::sceKernelGetThreadStackFreeSize(self.0) }) as usize
    }

    // TODO: default impls
    pub fn status_info(&self) -> NativeResult<sys::SceKernelThreadInfo> {
        let mut info = MaybeUninit::<sys::SceKernelThreadInfo>::uninit();
        native_error(unsafe {
            sys::sceKernelReferThreadStatus(self.0, (&raw mut info).cast())
        })
        .map(|_| unsafe { info.assume_init() })
    }
    pub fn runtime_status(
        &self,
    ) -> NativeResult<sys::SceKernelThreadRunStatus> {
        let mut info = MaybeUninit::<sys::SceKernelThreadRunStatus>::uninit();
        native_error(unsafe {
            sys::sceKernelReferThreadRunStatus(self.0, (&raw mut info).cast())
        })
        .map(|_| unsafe { info.assume_init() })
    }

    pub fn delete(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelDeleteThread(self.0) })
    }

    pub fn terminate(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelTerminateThread(self.0) })
    }

    /* This is not needed
    pub fn terminate_and_delete(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceKernelTerminateDeleteThread(self.0) })
    }*/
}

/*impl Drop for Thread {
    fn drop(&mut self) {
        // TODO: terminate?
    }
}*/

pub unsafe fn spawn_unsafe<F>(name: &ffi::CStr, entry: F) -> NativeResult<Thread>
where
    F: FnOnce() -> NativeResult<()> + Send,
{
    const DEFAULT_PRIORITY: i32 = 48;
    const DEFAULT_STACK_SIZE: usize = 4096;
    const DEFAULT_ATTRIBUTES: sys::ThreadAttributes =
        sys::ThreadAttributes::USER
            .union(sys::ThreadAttributes::VFPU)
            .union(sys::ThreadAttributes::NO_FILLSTACK);
    //.union(sys::ThreadAttributes::SCRATCH_SRAM);
    // prevent ILLEGAL_ATTR
    unsafe extern "C" fn function<F>(
        _arg_size: usize,
        arg: *mut ffi::c_void,
    ) -> i32
    where
        F: FnOnce() -> NativeResult<()> + Send,
    {
        if arg.is_null() {
            panic!("function arg should be non-null");
        }
        let entry: Box<(F, u8)> = unsafe { Box::from_raw(arg.cast()) };
        let (entry, _) = *entry;
        match entry() {
            Ok(()) => 0,
            Err(err) => err.inner(),
        }
    }
    let mut thread = Thread::new(
        name,
        function::<F>,
        DEFAULT_PRIORITY,
        DEFAULT_STACK_SIZE,
        DEFAULT_ATTRIBUTES,
        None,
    )?;
    unsafe {
        // This is absurd, if size_of::<F>() == 0
        // (and it is because regular functions are ZSTs)
        // The stupid OS will just throw away the pointer!
        thread.start(Box::new((entry, 0u8)))?;
    };
    Ok(thread)
}

pub fn spawn<F>(name: &ffi::CStr, entry: F) -> NativeResult<Thread>
where
    F: FnOnce() -> NativeResult<()> + Send + 'static,
{
	unsafe {
		spawn_unsafe(name, entry)
	}
}

// We won't implement semaphores, nor mutexes, nor lw-mutexes
// because the PSP kernel only supports one core.
