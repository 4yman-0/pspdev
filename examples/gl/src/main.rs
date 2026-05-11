//! PSP test application.
#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

extern crate alloc;

mod renderer;
use renderer::render_tex_aabb;

use psp_apis::fs::{
    Directory,
};
use psp_apis::gfx::{
    Gfx,
    color::Color32,
    gl::{GlResult, Mat3By4, MatrixMode},
};
use psp_apis::input::{Buttons, Input};

use glam::{EulerRot, Mat3, Mat4, Vec2, Vec3};
use psp_sys::{dprint, enable_home_button, sys};

mod loader;
use loader::*;

mod frame_clock;
use frame_clock::FrameClock;

psp_sys::module!("gl", 0, 1);

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

const LIGHT_POSITION: Vec3 = Vec3::new(0.0, 0.0, -3.0);

fn psp_main() {
    const NAV_SPEED: f32 = 0.07;
    const NAV_ROT_SPEED: f32 = NAV_SPEED;

    enable_home_button();
    let mut gfx = Gfx::init_default().unwrap();
    warn_unwrap(gfx.start_frame_with(|frame| {
        let gl = frame.gl_mut();

        let perspective = Mat4::perspective_rh_gl(
            deg_to_rad(90.0), //90º
            16.0 / 9.0,
            1.0,
            // it has to be negative otherwise it wont work
            -1.0,
        );
        gl.overwrite_projection_matrix(perspective);

        gl.shading_model(sys::ShadingModel::Flat);

        gl.set_state(sys::GuState::Lighting, true);
        gl.light_mode(sys::LightMode::SeparateSpecularColor);

        gl.set_state(sys::GuState::Light0, true);
        gl.light_position(0, &LIGHT_POSITION);
        gl.light_attenuation(0, 0.6, 0.05, 0.0, 0.0);
        gl.light_type_components(
            0,
            sys::LightType::Pointlight,
            sys::LightComponent::AMBIENT
                .union(sys::LightComponent::DIFFUSE)
                .union(sys::LightComponent::SPECULAR),
        );
        gl.light_color(
            0,
            sys::LightComponent::AMBIENT,
            Color32::from_rgba(0x999999aa),
        );
        gl.light_color(
            0,
            sys::LightComponent::DIFFUSE,
            Color32::from_rgb(0x0055ff),
        );
        gl.light_color(
            0,
            sys::LightComponent::SPECULAR,
            Color32::from_rgba(0x007755ff),
        );
        Ok(())
    }));
    let mut input = Input::init(sys::CtrlMode::Analog);

    let emulated = Directory::open(c"ms0:/PSP/GAME/PSPDEV_EMU").is_ok();

    if emulated {
        dprint!("PSPDEV_EMU detected!");
    } else {
        dprint!("Real PSP!");
    }

    let model = load_model(asset!(emulated, "test.pspm"));

    let shading = load_shading(asset!(emulated, "test.psps"));

    let texture = load_texture(asset!(emulated, "test.pspt"));

    let lamp_texture = load_texture(asset!(emulated, "lamp.pspt"));

    let mut translation = Vec3::new(0.0, 0.0, -2.0);
    let mut rotation = Vec3::ZERO;
    let mut frame_clock = FrameClock::default();

    loop {
        frame_clock = frame_clock.update();
        input.read_mut();

        if input.button_pressed(Buttons::DOWN) {
            translation.y -= NAV_SPEED;
        }
        if input.button_pressed(Buttons::UP) {
            translation.y += NAV_SPEED;
        }
        if input.button_pressed(Buttons::LEFT) {
            translation.x -= NAV_SPEED;
        }
        if input.button_pressed(Buttons::RIGHT) {
            translation.x += NAV_SPEED;
        }
        if input.button_pressed(Buttons::LTRIGGER) {
            translation.z -= NAV_SPEED;
        }
        if input.button_pressed(Buttons::RTRIGGER) {
            translation.z += NAV_SPEED;
        }
        if input.button_pressed(Buttons::CIRCLE) {
            rotation.x -= NAV_ROT_SPEED;
        }
        if input.button_pressed(Buttons::CROSS) {
            rotation.x += NAV_ROT_SPEED;
        }
        if input.button_pressed(Buttons::SQUARE) {
            rotation.y -= NAV_ROT_SPEED;
        }
        if input.button_pressed(Buttons::TRIANGLE) {
            rotation.y += NAV_ROT_SPEED;
        }

        warn_unwrap(gfx.start_frame_with(|frame| {
            let gl = frame.gl_mut();
            gl.clear_color(Color32::BLACK);
            gl.clear(
                sys::ClearFlags::DEPTH_BUFFER_BIT
                    | sys::ClearFlags::COLOR_BUFFER_BIT,
                1,
            )?;

            {
                // Gran Turismo jittering
                const JITTER: f32 = 1.0 / 272.0;
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
            render_tex_aabb(
                gl,
                &lamp_texture,
                LIGHT_POSITION,
                Vec2::new(0.4, -0.4),
            )?;

            if let Some(real_ambient) = shading.real_ambient {
                gl.ambient(real_ambient);
            }
            gl.color_material(shading.light_components);
            if let Some(ambient) = shading.ambient {
                gl.material_ambient(ambient);
            }
            if let Some(diffuse) = shading.diffuse {
                gl.material_diffuse(diffuse);
            }
            if let Some(specular) = shading.specular {
                gl.material_specular(specular);
            }
            if let Some(specular_coeff) = shading.specular_coeff {
                gl.specular_coeff(specular_coeff);
            }

            gl.set_matrix(
                MatrixMode::Model,
                &Mat3By4::from_mat3_vec3(
                    Mat3::from_scale(Vec2::new(1.0, 1.0))
                        * Mat3::from_euler(
                            EulerRot::XYZEx,
                            rotation.x,
                            rotation.y,
                            rotation.z,
                        ),
                    translation,
                ),
            );
            gl.texture(sys::MipmapLevel::None, &texture);
            //gl.texture_scale(1.0 / 64.0, 1.0 / 64.0);
            gl.set_state(sys::GuState::Texture2D, true);
            let v = gl.bind_indexed_vertices(
                model.vertex_type,
                model.vertex_size,
                &model.vertices[..],
                &model.indices[..],
            )?;
            gl.draw_primitives(v, model.primitive);
            gl.set_state(sys::GuState::Texture2D, false);
            Ok(())
        }));

        psp_apis::display::wait_vblank_start();
    }
}
