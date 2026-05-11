//! PSP APIs
#![no_std]
#![cfg(target_os = "psp")]

extern crate alloc;

pub mod atrac;
pub mod audio;
pub mod display;
pub mod error;
pub mod fs;

#[cfg(feature = "gfx")]
pub mod gfx;

pub mod input;
pub mod kernel;
pub mod mp3;
pub mod power;
pub mod rtc;
pub mod thread;
pub mod wlan;

#[cfg(feature = "critical-section")]
mod critical_section;

// // not tested
// pub mod audio_codec;

#[repr(align(16))]
pub struct Align16<T: ?Sized>(T);

pub mod ptr {
	#[inline]
	pub fn make_uncached<T>(ptr: *const T) -> *const T {
		ptr.map_addr(|a| a | 0x40000000)
	}

	#[inline]
	pub fn make_uncached_mut<T>(ptr: *mut T) -> *mut T {
		ptr.map_addr(|a| a | 0x40000000)
	}
	
	#[inline]
	pub fn make_cached<T>(ptr: *const T) -> *const T {
		ptr.map_addr(|a| a & (!0x40000000))
	}

	#[inline]
	pub fn make_cached_mut<T>(ptr: *mut T) -> *mut T {
		ptr.map_addr(|a| a & (!0x40000000))
	}
}
