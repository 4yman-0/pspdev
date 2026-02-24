//! NOT TESTED

use psp_sys::sys;
use crate::error::{NativeResult, NativeError, native_error/*, native_result*/};
//use alloc::boxed::Box;

pub type VideoCodecResult<T> = Result<T, VideoCodecError>;

pub enum VideoCodecError {
	Native(NativeError),

	//ContextTooSmall,
}

impl From<NativeError> for VideoCodecError {
	fn from(from: NativeError) -> Self {
		Self::Native(from)
	}
}

pub use sys::VideoCodec as VideoCodecType;

pub struct VideoCodec {
	codec: sys::SceVideocodecCodec,
	codec_type: VideoCodecType,
}

impl VideoCodec {
	pub fn new(codec_type: VideoCodecType) -> NativeResult<Self> {
		let mut codec = core::mem::MaybeUninit::<sys::SceVideocodecCodec>::uninit();
		native_error(unsafe {
			sys::sceVideocodecInit(&mut codec, codec_type as i32)
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

	pub fn decode(&mut self) -> NativeResult<&[i16]> {
		native_error(unsafe {
			sys::sceVideocodecDecode((&mut codec, self.codec_type as i32)
		}).map(|_| unsafe {
			core::slice::from_raw_parts(
				self.codec.output_buffer,
				self.codec.output_samples_written as usize,
			)
		})
	}

	fn close_non_consuming(&mut self) -> NativeResult<()> {
		native_error(unsafe {
			sys::sceVideocodecReleaseEDRAM(&mut self.codec)
		})
	}

	pub fn get_edram(&mut self) -> NativeResult<()> {
		native_error(unsafe {
			sys::sceVideocodecGetEDRAM(&mut self.codec, self.codec_type as i32)
		})
	}

	pub fn check_needed_memory(&self) -> NativeResult<()> {
		native_error(unsafe {
			sys::sceVideocodecCheckNeedMem((&raw const self.codec) as *mut u32, self.codec_type as i32)
		})
	}
}

impl Drop for VideoCodec {
	fn drop(&mut self) {
		let _ = self.close_non_consuming();
	}
}
