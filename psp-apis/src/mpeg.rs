//! WIP

//use crate::audio::Sample;
use crate::error::{
    /*NativeError, */ NativeResult, native_error, native_result,
};
use psp_sys::sys;

pub struct MpegHandle(sys::SceMpeg);

impl MpegHandle {
	pub fn init() -> NativeResult<()> {
		native_error(unsafe {
			sys::sceMpegInit()
		})
	}
	pub fn finish() -> NativeResult<()> {
		native_error(unsafe {
			sys::sceMpegFinish()
		})
	}
}

pub struct MpegRingbuffer(sys::SceMpegRingBuffer);

impl MpegRingbuffer {
	pub fn ring_buffer_size(packets: usize) -> usize {
		native_error(unsafe {
			sys::sceMpegRingbufferQueryMemSize(packets as i32)
		}).map(|s| s as usize)
	}
	pub fn new(
		packets: usize,
		data: &mut [u8],
		callback: sys::SceMpegRingbufferCB,
		callback_param: *mut core::ffi::c_void,
	) -> NativeResult<Self> {
		let mut ring_buffer =
		 core::mem::MaybeUninit::<sys::sceMpegRingbuffer>::uninit();
		native_error(unsafe {
			sys::sceMpegRingbufferConstruct(
				&mut ring_buffer,
				packets as i32,
				data.as_mut_ptr(),
				data.len(),
				callback,
				callback_param,
			)
		}).map(|_| unsafe {
			Self(ring_buffer.assume_init())
		})
	}
	fn close_non_consuming(&mut self) {
		unsafe {
			sys::sceMpegRingbufferDestruct(&mut self.0);
		}
	}
	pub fn available_size(&self) -> NativeResult<usize> {
		native_result(unsafe {
			sys::sceMpegRingbufferAvailableSize(&mut self.0)
		}).map(|s| s as usize)
	}
	pub fn put(&mut self, packets: usize, available: usize) -> NativeResult<usize> {
		native_result(unsafe {
			sys::sceMpegRingbufferPut(&mut self.0, packets as i32, available as i32)
		}).map(|s| s as usize)		
	}
}

impl Drop for MpegRingbuffer {
	fn drop(&mut self) {
		self.close_non_consuming();
	}
}

impl MpegHandle {
	fn init(
		&mut self,
		data: &mut [u8],
		ring_buffer: &mut SceMpegRingbuffer,
		frame_width: usize,
	) -> NativeResult<()> {
		native_error(unsafe {
			sys::sceMpegCreate(
				self.0,
				data.as_ptr_mut(),
				data.len(),
				ring_buffer,
				frame_width as i32,
				0,
				0,
			)
		})
	}
	fn close_non_consuming(&mut self) {
		unsafe {
			sys::sceMpegDelete(self.0);
		};
	}
	// streamoffset
}

impl Drop for MpegHandle {
	fn drop(&mut self) {
		self.close_non_consuming();
	}
}
