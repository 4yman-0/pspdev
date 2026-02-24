use crate::error::{NativeError, NativeResult, native_error, native_result};
use psp_sys::sys;

pub type Path = core::ffi::CStr;
pub type PathBuf = alloc::ffi::CString;

pub type FsResult<T> = Result<T, FsError>;

#[derive(Clone, Debug)]
pub enum FsError {
    FileNotFound,
    NoMoreDirectoryEntries,
    UnexpectedBytesWritten,
    Native(NativeError),
}

impl core::fmt::Display for FsError {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), core::fmt::Error> {
        match self {
            Self::Native(err) => {
                write!(f, "Native file system error: {err}")
            }
            _ => write!(f, "File system error: {self:?}"),
        }
    }
}
impl core::error::Error for FsError {}

impl From<NativeError> for FsError {
    fn from(from: NativeError) -> Self {
        Self::Native(from)
    }
}

pub struct File {
    uid: sys::SceUid,
}

impl File {
    const DEFAULT_PERMISSIONS: u32 = 0o0764;

    /// # Safety
    /// `uid` must be a valid file UID, we can't check (yet).
    pub unsafe fn from_uid(uid: sys::SceUid) -> NativeResult<Self> {
        native_result(uid.0).map(|raw_uid| Self {
            uid: (raw_uid as i32).into(),
        })
    }

    pub fn open(
        path: &Path,
        open_flags: sys::IoOpenFlags,
    ) -> NativeResult<Self> {
        let uid = unsafe {
            sys::sceIoOpen(
                path.as_ptr().cast(),
                open_flags,
                Self::DEFAULT_PERMISSIONS as _,
            )
        };
        native_error(uid.0).map(|_| Self { uid })
    }

    pub fn open_async(
        path: &Path,
        open_flags: sys::IoOpenFlags,
    ) -> NativeResult<Self> {
        let uid = unsafe {
            sys::sceIoOpenAsync(
                path.as_ptr().cast(),
                open_flags,
                Self::DEFAULT_PERMISSIONS as _,
            )
        };
        native_error(uid.0).map(|_| Self { uid })
    }

    fn close_non_consuming(&self) -> NativeResult<()> {
        let result = unsafe { sys::sceIoClose(self.uid) };
        native_error(result)
    }

    pub fn close_async_non_consuming(&self) -> NativeResult<()> {
        let result = unsafe { sys::sceIoCloseAsync(self.uid) };
        native_error(result)
    }

    pub fn close_async(self) -> NativeResult<()> {
        self.close_non_consuming()?;
        let _ = core::mem::ManuallyDrop::new(self);
        Ok(())
    }

    pub fn write(&self, data: &[u8]) -> FsResult<usize> {
        let bytes_written = unsafe {
            sys::sceIoWrite(self.uid, data.as_ptr().cast(), data.len())
        } as usize;
        if bytes_written != data.len() {
            Err(FsError::UnexpectedBytesWritten)
        } else {
            Ok(bytes_written)
        }
    }
    pub fn write_async(&self, data: &[u8]) -> FsResult<()> {
        let bytes_written = unsafe {
            sys::sceIoWriteAsync(
                self.uid,
                data.as_ptr().cast(),
                data.len() as u32,
            )
        } as usize;
        if bytes_written != data.len() {
            Err(FsError::UnexpectedBytesWritten)
        } else {
            Ok(())
        }
    }

    pub fn read(&mut self, data: &mut [u8]) -> FsResult<usize> {
        let bytes_written = unsafe {
            sys::sceIoRead(
                self.uid,
                data.as_mut_ptr().cast(),
                data.len() as u32,
            )
        } as usize;
        if bytes_written != data.len() {
            Err(FsError::UnexpectedBytesWritten)
        } else {
            Ok(bytes_written)
        }
    }
    pub fn read_async(&mut self, data: &mut [u8]) -> FsResult<()> {
        let bytes_written = unsafe {
            sys::sceIoReadAsync(
                self.uid,
                data.as_mut_ptr().cast(),
                data.len() as u32,
            )
        } as usize;
        if bytes_written != data.len() {
            Err(FsError::UnexpectedBytesWritten)
        } else {
            Ok(())
        }
    }

    pub fn seek(&mut self, offset: i64, whence: sys::IoWhence) -> i64 {
        unsafe { sys::sceIoLseek(self.uid, offset, whence) }
    }
    pub fn seek_async(
        &mut self,
        offset: i64,
        whence: sys::IoWhence,
    ) -> NativeResult<()> {
        native_error(unsafe { sys::sceIoLseekAsync(self.uid, offset, whence) })
    }
    pub fn seek_i32(&mut self, offset: i32, whence: sys::IoWhence) -> i32 {
        unsafe { sys::sceIoLseek32(self.uid, offset, whence) }
    }
    pub fn seek_i32_async(
        &mut self,
        offset: i32,
        whence: sys::IoWhence,
    ) -> NativeResult<()> {
        native_error(unsafe {
            sys::sceIoLseek32Async(self.uid, offset, whence)
        })
    }

    pub fn wait_async(&self) -> NativeResult<i64> {
        let mut result = 0i64;
        native_error(unsafe { sys::sceIoWaitAsync(self.uid, &raw mut result) })
            .map(|_| result)
    }

    // TODO: sceIoWaitAsyncCB

    pub fn poll_async(&self) -> NativeResult<i64> {
        let mut result = 0i64;
        native_error(unsafe { sys::sceIoPollAsync(self.uid, &raw mut result) })
            .map(|_| result)
    }

    pub fn cancel_async(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceIoCancel(self.uid) })
    }

    pub fn device_type(&self) -> NativeResult<u32> {
        native_result(unsafe { sys::sceIoGetDevType(self.uid) })
    }

    pub fn set_async_priority(&mut self, pri: i32) -> NativeResult<()> {
        native_error(unsafe { sys::sceIoChangeAsyncPriority(self.uid, pri) })
    }

    // TODO: sceIoSetAsyncCallback
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = self.close_non_consuming();
    }
}

#[cfg(feature = "no_std_io")]
use no_std_io::io::{
    Error as IoError, ErrorKind as IoErrorKind, Result as IoResult,
};

#[cfg(feature = "no_std_io")]
impl no_std_io::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        File::read(self, buf)
            .map_err(|_| IoError::new(IoErrorKind::Other, "native error"))
    }
}

#[cfg(feature = "no_std_io")]
impl no_std_io::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        File::write(self, buf)
            .map_err(|_| IoError::new(IoErrorKind::Other, "native error"))
    }
    fn flush(&mut self) -> IoResult<()> {
        File::wait_async(self)
            .map_err(|_| IoError::new(IoErrorKind::Other, "native error"))
            .and(Ok(()))
    }
}

#[cfg(feature = "no_std_io")]
impl no_std_io::io::Seek for File {
    fn seek(&mut self, pos: no_std_io::io::SeekFrom) -> IoResult<u64> {
        use no_std_io::io::SeekFrom;
        Ok((match pos {
            SeekFrom::Start(pos) => {
                File::seek(self, pos as i64, sys::IoWhence::Set)
            }
            SeekFrom::End(pos) => File::seek(self, pos, sys::IoWhence::End),
            SeekFrom::Current(pos) => File::seek(self, pos, sys::IoWhence::Cur),
        }) as u64)
    }
}

pub struct Directory {
    uid: sys::SceUid,
}

impl Directory {
    pub fn open(path: &Path) -> NativeResult<Self> {
        let uid = unsafe { sys::sceIoDopen(path.as_ptr().cast()) };
        native_error(uid.0).map(|_| Self { uid })
    }

    fn close_non_consuming(&mut self) -> NativeResult<()> {
        let result = unsafe { sys::sceIoDclose(self.uid) };
        native_error(result)
    }

    pub fn read(&self) -> FsResult<(sys::SceIoDirent, bool)> {
        let mut dir_entry = sys::SceIoDirent {
            d_stat: default_io_stat(),
            d_name: [48; 256],
            d_private: Default::default(),
            dummy: Default::default(),
        };
        let result = unsafe { sys::sceIoDread(self.uid, &raw mut dir_entry) };
        native_result(result)
            .map(|result| (dir_entry, result == 0))
            .map_err(|e| e.into())
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = self.close_non_consuming();
    }
}

#[must_use]
fn default_io_stat() -> sys::SceIoStat {
    use sys::ScePspDateTime as PspDateTime;
    sys::SceIoStat {
        st_mode: sys::IoStatMode::IFREG,
        st_attr: sys::IoStatAttr::IFREG,
        st_size: 0,
        st_ctime: PspDateTime::default(),
        st_atime: PspDateTime::default(),
        st_mtime: PspDateTime::default(),
        st_private: Default::default(),
    }
}

pub fn get_stat(path: &Path) -> NativeResult<sys::SceIoStat> {
    let mut stat = default_io_stat();
    let result =
        unsafe { sys::sceIoGetstat(path.as_ptr().cast(), &raw mut stat) };
    native_error(result).map(|_| stat)
}
pub fn set_stat(
    path: &Path,
    stat: &sys::SceIoStat,
    mask: u32,
) -> NativeResult<()> {
    native_error(unsafe {
        // SAFETY: it's not actually mutable
        // It's just not const * const _
        sys::sceIoChstat(
            path.as_ptr().cast(),
            (&raw const *stat) as *mut sys::SceIoStat,
            mask.cast_signed(),
        )
    })
}

pub fn chdir(path: &Path) -> NativeResult<()> {
    native_error(unsafe { sys::sceIoChdir(path.as_ptr().cast()) })
}

pub fn remove(path: &Path) -> NativeResult<()> {
    native_error(unsafe { sys::sceIoRemove(path.as_ptr().cast()) })
}

pub fn remove_dir(path: &Path) -> Option<()> {
    let result = unsafe { sys::sceIoRmdir(path.as_ptr().cast()) };
    if result == 0 { Some(()) } else { None }
}

pub fn mkdir(path: &Path, mode: sys::IoPermissions) -> NativeResult<()> {
    native_error(unsafe { sys::sceIoMkdir(path.as_ptr().cast(), mode) })
}

pub fn rename(old: &Path, new: &Path) -> NativeResult<()> {
    native_error(unsafe {
        sys::sceIoRename(old.as_ptr().cast(), new.as_ptr().cast())
    })
}

pub fn stdout() -> NativeResult<File> {
    unsafe { File::from_uid(sys::sceKernelStdout()) }
}

pub fn stdin() -> NativeResult<File> {
    unsafe { File::from_uid(sys::sceKernelStdout()) }
}

pub fn stderr() -> NativeResult<File> {
    unsafe { File::from_uid(sys::sceKernelStdout()) }
}

// TODO: Weird SceIoDev*/SceIo(Un)Assign

pub fn sync(device: &Path, unknown: u32) -> i32 {
    unsafe { sys::sceIoSync(device.as_ptr().cast(), unknown) }
}
