#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::{boxed::Box /*, vec::Vec*/};
use core::time::Duration;
use psp_apis::atrac::{
    AtracHandle,
    //AtracDecodeInfo,
};
use psp_apis::audio::{
	align_sample_count,
    AudioFormat,
    //AudioOutputFrequency,
    AudioChannel,
    Sample,
    Volume,
};
use psp_apis::display::wait_vblank_start;
use psp_apis::fs::{
    Directory,
    File,
    //self,
    //Path,
};
use psp_apis::thread;
use psp_sys::sys;

psp_sys::module!("music", 0, 1);

fn play_atrac(file_path: alloc::ffi::CString) -> thread::Thread {
    thread::spawn(c"playback", move || {
        loop {
            let volume = Volume::from_mono_f32(0.4);
            let mut channel = AudioChannel::reserve_next(
            	512,
				AudioFormat::Stereo,
            ).unwrap();
            let mut audio_file =
                File::open(&file_path, sys::IoOpenFlags::RD_ONLY).unwrap();
            let _ = audio_file.seek_i32(0, sys::IoWhence::Set);
            let mut audio_buffer: Box<[u8]> =
                unsafe { Box::<[u8]>::new_uninit_slice(1024).assume_init() };
            audio_file.read(&mut audio_buffer).unwrap();
            // FIXME: works in PPSSPP but not on PSP-3000
            let mut audio_handle =
                unsafe { AtracHandle::new(&mut audio_buffer).unwrap() };
            let _ = audio_handle.set_loops(None);

            let mut stream = unsafe { thread::spawn_unsafe(c"stream", || {
                loop {
                    let remaining = audio_handle.remaining_frames().unwrap();
                    if remaining.is_none() {
                        // all data is available
                        break;
                    } else if remaining.is_some_and(|r| r < 20)
                        && let Some((data_add, read_from)) =
                            audio_handle.start_data_add().unwrap()
                    {
                        let data_add_len = {
                            let _ = audio_file
                                .seek(read_from as i64, sys::IoWhence::Set);
                            audio_file.read(data_add).unwrap();
                            data_add.len()
                        };
                        audio_handle.notify_data_add(data_add_len).unwrap();
                    }
                    // The PSP has cooperative multithreading!?
                    thread::sleep(Duration::from_millis(3)).unwrap();
                }
                Ok(())
            })
            .unwrap() };
            let max_sample_count = audio_handle.max_sample_count().unwrap();
            let mut sample_buffer =
                Box::<[Sample]>::new_uninit_slice(max_sample_count);
            loop {
                let remaining = audio_handle.remaining_frames().unwrap();
                if let None | Some(0) = remaining {
                    break;
                }

                let info = audio_handle
                    .decode(unsafe { sample_buffer.assume_init_mut() })
                    .unwrap();
                assert!(
                    info.sample_count <= max_sample_count,
                    "Max sample count exceeded"
                );
                let sample_count = info.sample_count.min(sample_buffer.len());
                channel.set_sample_count(align_sample_count(sample_count as u32)).unwrap();
                channel
                    .output_blocking(&volume, unsafe {
                        sample_buffer.assume_init_ref()
                    })
                    .unwrap();
                if info.is_end {
                    break;
                }
            }
            if let Err(_) | Ok(false) = stream.has_exited() {
                let _ = stream.terminate();
            }
        }
    })
    .unwrap()
}

fn psp_main() {
    let emulated = Directory::open(c"ms0:/PSP/GAME/PSPDEV_EMU").is_ok();

	AudioChannel::init().unwrap();
    let /*mut*/ playback = play_atrac(
        (if emulated {
            c"ms0:/PSP/GAME/PSPDEV_EMU/music.at3"
        } else {
            c"ms0:/PSP/GAME/PSPDEV/music.at3"
        }).into(),
    );

    loop {
        if let Err(_) | Ok(true) = playback.has_exited() {
            break;
        }
        wait_vblank_start();
    }
}
