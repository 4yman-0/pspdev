//! NOT TESTED, WIP!
//! AFAIK this can only decode frames of encoded audio, so you need a parser

use crate::error::{
    /*NativeError,*/ NativeResult, native_error, /*, native_result*/
};
use core::mem::size_of_val;
use psp_sys::sys;
//use alloc::boxed::Box;

pub use sys::AudioCodec as AudioCodecType;

pub struct AudioCodec {
    pub codec: sys::SceAudiocodecCodec,
    codec_type: AudioCodecType,
}

impl AudioCodec {
    pub fn new(codec_type: AudioCodecType) -> NativeResult<Self> {
        let mut codec =
            core::mem::MaybeUninit::<sys::SceAudiocodecCodec>::uninit();
        native_error(unsafe {
            sys::sceAudiocodecInit(codec.assume_init_mut(), codec_type as i32)
        })?;
        Ok(Self {
            codec: unsafe { codec.assume_init() },
            codec_type,
        })
    }

    pub fn set_input(&mut self, input: &[u8]) {
        self.codec.input_buffer = input.as_ptr();
        self.codec.input_bytes_read = input.len() as i32;
    }

    pub fn check_error(&self) -> NativeResult<()> {
        native_error(self.codec.error)
    }

    pub fn decode(&mut self, output: &mut [i16]) -> NativeResult<()> {
        self.codec.output_buffer = output.as_mut_ptr();
        self.codec.alloc_mem = size_of_val(output) as u32;

        native_error(unsafe {
            sys::sceAudiocodecDecode(&mut self.codec, self.codec_type as i32)
        })
    }

    fn close_non_consuming(&mut self) -> NativeResult<()> {
        native_error(unsafe { sys::sceAudiocodecReleaseEDRAM(&mut self.codec) })
    }

    pub fn get_edram(&mut self) -> NativeResult<()> {
        native_error(unsafe {
            sys::sceAudiocodecGetEDRAM(&mut self.codec, self.codec_type as i32)
        })
    }

    pub fn check_needed_memory(&self) -> NativeResult<()> {
        native_error(unsafe {
            sys::sceAudiocodecCheckNeedMem(
                (&raw const self.codec) as *mut _,
                self.codec_type as i32,
            )
        })
    }
}

impl Drop for AudioCodec {
    fn drop(&mut self) {
        let _ = self.close_non_consuming();
    }
}

//unsafe impl core::marker::Send for AudioCodec {}
