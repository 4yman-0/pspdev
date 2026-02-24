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
pub mod thread;

#[cfg(feature = "critical-section")]
mod critical_section;

// // not tested
// pub mod audio_codec;

#[repr(align(16))]
pub struct Align16<T: ?Sized>(T);
