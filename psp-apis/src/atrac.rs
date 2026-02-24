use crate::audio::Sample;
use crate::error::{
    /*NativeError, */ NativeResult, native_error, native_result,
};
use psp_sys::sys;

pub struct AtracDecodeInfo {
    pub sample_count: usize,
    pub remaining: usize,
    pub is_end: bool,
}

pub struct AtracHandle(i32);

impl AtracHandle {
    /// # Safety
    /// indirect raw ptr deref
    pub unsafe fn new(buffer: &mut [u8]) -> NativeResult<Self> {
        native_result(unsafe {
            sys::sceAtracSetDataAndGetID(
                buffer.as_mut_ptr().cast(),
                buffer.len(),
            )
        })
        .map(|id| Self(id as i32))
    }

    pub fn decode(
        &mut self,
        output: &mut [Sample],
    ) -> NativeResult<AtracDecodeInfo> {
        let mut sample_count = 0i32;
        let mut is_end = 0i32;
        let mut remaining = 0i32;
        native_error(unsafe {
            sys::sceAtracDecodeData(
                self.0,
                output.as_mut_ptr().cast(),
                &raw mut sample_count,
                &raw mut is_end,
                &raw mut remaining,
            )
        })?;
        Ok(AtracDecodeInfo {
            sample_count: sample_count as usize,
            remaining: remaining as usize,
            is_end: is_end == 1,
        })
    }
    pub fn remaining_frames(&self) -> NativeResult<Option<usize>> {
        let mut remaining_frames = 0i32;
        native_error(unsafe {
            sys::sceAtracGetRemainFrame(self.0, &raw mut remaining_frames)
        })?;
        Ok(if remaining_frames == -1 {
            None
        } else {
            Some(remaining_frames as usize)
        })
    }

    pub fn start_data_add(
        &mut self,
    ) -> NativeResult<Option<(&mut [u8], usize)>> {
        let mut destination: *mut u8 = core::ptr::null_mut();
        let mut to_write = 0;
        let mut source_position = 0;
        native_result(unsafe {
            sys::sceAtracGetStreamDataInfo(
                self.0,
                &raw mut destination,
                &raw mut to_write,
                &raw mut source_position,
            )
        })
        .map(|_| {
            if !destination.is_null() {
                Some((
                    unsafe {
                        core::slice::from_raw_parts_mut(
                            destination, /*.offset(source_position as isize)*/
                            to_write as usize,
                        )
                    },
                    source_position as usize,
                ))
            } else {
                None
            }
        })
    }

    pub fn notify_data_add(&mut self, bytes_to_add: usize) -> NativeResult<()> {
        native_error(unsafe {
            sys::sceAtracAddStreamData(self.0, bytes_to_add as u32)
        })
    }

    pub fn bitrate(&self) -> NativeResult<u32> {
        let mut bitrate = 0i32;
        native_error(unsafe {
            sys::sceAtracGetBitrate(self.0, &raw mut bitrate)
        })?;
        Ok(bitrate as u32)
    }

    pub fn set_loops(&mut self, loops: Option<u32>) -> NativeResult<()> {
        native_error(unsafe {
            sys::sceAtracSetLoopNum(
                self.0,
                loops.map(|l| l as i32).unwrap_or(-1),
            )
        })
    }

    fn close_non_consuming(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceAtracReleaseAtracID(self.0) })
    }

    pub fn next_sample_count(&self) -> NativeResult<usize> {
        let mut sample_count = 0i32;
        native_error(unsafe {
            sys::sceAtracGetNextSample(self.0, &raw mut sample_count)
        })?;
        Ok(sample_count as usize)
    }

    pub fn max_sample_count(&self) -> NativeResult<usize> {
        let mut sample_count = 0i32;
        native_error(unsafe {
            sys::sceAtracGetMaxSample(self.0, &raw mut sample_count)
        })?;
        Ok(sample_count as usize)
    }

    // TODO: etc
}

impl Drop for AtracHandle {
    fn drop(&mut self) {
        let _ = self.close_non_consuming();
    }
}
