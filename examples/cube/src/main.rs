#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

extern crate alloc;

use psp_apis::fs::Directory;
use psp_apis::gfx::{
    Gfx,
    color::Color32,
    gl::{GlResult, Mat3By4, MatrixMode},
};

use glam::{EulerRot, Mat3, Mat4, Vec3};
use psp_sys::{dprint, enable_home_button, sys};

mod loader;
use loader::{load_model, load_texture};

mod frame_clock;
use frame_clock::FrameClock;

psp_sys::module!("cube", 0, 1);

fn warn_unwrap(result: GlResult<()>) {
    result.unwrap_or_else(|err| dprint!("Warning: {err:?}"));
}

fn deg_to_rad(deg: f32) -> f32 {
    deg * (core::f32::consts::PI / 180.0)
}

macro_rules! asset {
    ($emulated:expr, $file_name:literal) => {
        if $emulated {
            unsafe {
                ::core::ffi::CStr::from_bytes_with_nul_unchecked(
                    concat!("ms0:/PSP/GAME/PSPDEV_EMU/", $file_name, "\0",)
                        .as_bytes(),
                )
            }
        } else {
            unsafe {
                ::core::ffi::CStr::from_bytes_with_nul_unchecked(
                    concat!("ms0:/PSP/GAME/PSPDEV/", $file_name, "\0",)
                        .as_bytes(),
                )
            }
        }
    };
}

fn psp_main() {
    const ROTATION_SPEED: Vec3 = Vec3::new(0.015, 0.03, 0.02);

    enable_home_button();
    let mut gfx = Gfx::init_default().unwrap();

    warn_unwrap(gfx.start_frame_with(|frame| {
        let gl = frame.gl_mut();
        gl.texture_filter(
            sys::TextureFilter::Linear,
            sys::TextureFilter::Linear,
        );

        let perspective =
            Mat4::perspective_rh_gl(deg_to_rad(90.0), 16.0 / 9.0, 1.0, -1.0);
        gl.overwrite_projection_matrix(perspective);
        Ok(())
    }));
    let emulated = Directory::open(c"ms0:/PSP/GAME/PSPDEV_EMU").is_ok();

    if emulated {
        dprint!("PSPDEV_EMU detected!");
    } else {
        dprint!("Real PSP!");
    }

    let cube = load_model(asset!(emulated, "cube.pspm"));
    let texture = load_texture(asset!(emulated, "cube.pspt")).unwrap();

    let mut frame_clock = FrameClock::default();
    let mut rotation = Vec3::ZERO;

    loop {
        frame_clock = frame_clock.update();
        rotation += ROTATION_SPEED;
        // retain precision
        rotation %= ::core::f32::consts::PI * 2.0;

        warn_unwrap(gfx.start_frame_with(|frame| {
            let gl = frame.gl_mut();
            gl.clear_color(Color32::BLUE);
            gl.clear(
                sys::ClearFlags::DEPTH_BUFFER_BIT
                    | sys::ClearFlags::COLOR_BUFFER_BIT,
                1,
            )?;

            {
                // Mild Gran Turismo jittering
                const JITTER: f32 = 0.5 / 272.0;
                let mut view = Mat3By4::IDENTITY;
                if frame_clock.edge_clock(2) {
                    view.w_axis.x += JITTER;
                }
                if frame_clock.offset(1).edge_clock(2) {
                    view.w_axis.y += JITTER;
                }
                gl.set_matrix(MatrixMode::View, &view)
            }

            gl.clear_color(Color32::WHITE);
            gl.set_matrix(
                MatrixMode::Model,
                &Mat3By4::from_mat3_vec3(
                    Mat3::from_euler(
                        EulerRot::XYZEx,
                        rotation.x,
                        rotation.y,
                        rotation.z,
                    ),
                    Vec3::new(0.0, 0.0, -2.0),
                ),
            );
            gl.texture(sys::MipmapLevel::None, &texture);
            gl.set_state(sys::GuState::Texture2D, true);
            let v = gl.bind_indexed_vertices(
                cube.vertex_type,
                cube.vertex_size,
                &cube.vertices[..],
                &cube.indices[..],
            )?;
            gl.draw_primitives(v, cube.primitive);
            gl.set_state(sys::GuState::Texture2D, false);
            Ok(())
        }));

        psp_apis::display::wait_vblank_start();
    }
}
