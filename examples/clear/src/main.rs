//! PSP test application.
#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

use psp_apis::gfx::{
    Gfx,
    color::Color32,
};

use psp_sys::{enable_home_button, sys};

const CLEAR_COLOR: Color32 = Color32::from_rgb_channels(3, 10, 3);

psp_sys::module!("clear", 0, 1);

fn psp_main() {
    enable_home_button();
    let mut gfx = Gfx::init_default().unwrap();

    loop {
        gfx.start_frame_with(|frame| {
            let gl = frame.gl_mut();
            
            gl.clear_color(CLEAR_COLOR);
            gl.clear(
                sys::ClearFlags::DEPTH_BUFFER_BIT
                    | sys::ClearFlags::COLOR_BUFFER_BIT,
                0,
            )?;

            Ok(())
        }).unwrap();
        
        psp_apis::display::wait_vblank_start();
    }
}
