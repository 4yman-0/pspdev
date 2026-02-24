//! This is not tested

use crate::audio::Sample;
use crate::error::{
    /*NativeError, */ NativeResult, native_error, native_result,
};
use core::mem::size_of_val;
use psp_sys::sys;

pub struct Mp3Handle(sys::Mp3Handle);

impl Mp3Handle {
    /// # Safety
    /// This function dereferences raw pointers indirectly (after [`Mp3Handle`] is created)
    pub unsafe fn reserve(
        mp3_stream_start: usize,
        mp3_stream_end: usize,
        mp3_buf: &mut [u8],
        pcm_buf: &mut [Sample],
    ) -> NativeResult<Self> {
        let mut init_args = sys::SceMp3InitArg {
            mp3_stream_start: mp3_stream_start as u32,
            mp3_stream_end: mp3_stream_end as u32,
            mp3_buf: (&raw mut *mp3_buf).cast(),
            mp3_buf_size: size_of_val(mp3_buf) as i32,
            pcm_buf: (&raw mut *pcm_buf).cast(),
            pcm_buf_size: size_of_val(pcm_buf) as i32,
            unk1: 0,
            unk2: 0,
        };
        native_result(unsafe {
            sys::sceMp3ReserveMp3Handle(&raw mut init_args)
        })
        .map(|id| Self(sys::Mp3Handle(id as i32)))
    }

    pub fn close_non_consuming(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceMp3ReleaseMp3Handle(self.0) })
    }

    pub fn init_resource() -> NativeResult<()> {
        native_error(unsafe { sys::sceMp3InitResource() })
    }
    pub fn term_resource() -> NativeResult<()> {
        native_error(unsafe { sys::sceMp3TermResource() })
    }

    pub fn init(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceMp3Init(self.0) })
    }

    pub fn decode(&mut self) -> NativeResult<&mut [Sample]> {
        let mut destination = core::ptr::null_mut();
        native_result(unsafe {
            sys::sceMp3Decode(self.0, &raw mut destination)
        })
        .map(|decoded_bytes| unsafe {
            core::slice::from_raw_parts_mut(destination, decoded_bytes as usize)
        })
    }

    pub fn start_data_add(&mut self) -> NativeResult<Option<&mut [u8]>> {
        let mut destination: *mut u8 = core::ptr::null_mut();
        let mut to_write = 0;
        let mut source_position = 0;
        native_result(unsafe {
            sys::sceMp3GetInfoToAddStreamData(
                self.0,
                &raw mut destination,
                &raw mut to_write,
                &raw mut source_position,
            )
        })
        .map(|_| {
            if !destination.is_null() {
                Some(unsafe {
                    core::slice::from_raw_parts_mut(
                        destination.offset(source_position as isize),
                        to_write as usize,
                    )
                })
            } else {
                None
            }
        })
    }

    pub fn notify_data_add(&mut self, added_size: usize) -> NativeResult<()> {
        native_result(unsafe {
            sys::sceMp3NotifyAddStreamData(self.0, added_size as i32)
        })
        .and(Ok(()))
    }

    pub fn is_data_needed(&self) -> NativeResult<bool> {
        native_result(unsafe { sys::sceMp3CheckStreamDataNeeded(self.0) })
            .map(|i| i == 1)
    }

    pub fn set_loops(&mut self, loops: Option<usize>) -> NativeResult<()> {
        native_error(unsafe {
            sys::sceMp3SetLoopNum(self.0, loops.map(|l| l as i32).unwrap_or(-1))
        })
    }
    pub fn loops(&self) -> NativeResult<usize> {
        native_result(unsafe { sys::sceMp3GetLoopNum(self.0) })
            .map(|i| i as usize)
    }
    pub fn decoded_samples(&self) -> NativeResult<usize> {
        native_result(unsafe { sys::sceMp3GetSumDecodedSample(self.0) })
            .map(|i| i as usize)
    }
    pub fn max_sample_count(&self) -> NativeResult<usize> {
        native_result(unsafe { sys::sceMp3GetMaxOutputSample(self.0) })
            .map(|i| i as usize)
    }

    pub fn sample_rate(&self) -> NativeResult<usize> {
        native_result(unsafe { sys::sceMp3GetSamplingRate(self.0) })
            .map(|i| i as usize)
    }

    pub fn bit_rate(&self) -> NativeResult<usize> {
        native_result(unsafe { sys::sceMp3GetBitRate(self.0) })
            .map(|i| i as usize)
    }

    pub fn channel_count(&self) -> NativeResult<usize> {
        native_result(unsafe { sys::sceMp3GetMp3ChannelNum(self.0) })
            .map(|i| i as usize)
    }

    pub fn reset_play_position(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceMp3ResetPlayPosition(self.0) })
    }
}

impl Drop for Mp3Handle {
    fn drop(&mut self) {
        let _ = self.close_non_consuming();
    }
}
