//! PSP test application.
#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

extern crate alloc;

use psp_apis::fs::{
    Directory,
    //self,
    //Path,
};
use psp_apis::gfx::{
    Gfx,
    color::Color32,
    gl::{GlResult, Mat3By4, MatrixMode},
    vertex::{VertexSize, const_vt_size},
};

//use alloc::{boxed::Box /*, vec::Vec*/};
use glam::{Mat4, Vec3};
use psp_sys::{dprint, enable_home_button, sys};

mod frame_clock;
use frame_clock::FrameClock;

#[repr(C)]
struct Vert {
    weight: [f32; 2],
    position: [f32; 3],
}

const MODEL_PRIMITIVE: sys::GuPrimitive = sys::GuPrimitive::Triangles;
const MODEL_VERTEX_TYPE: sys::VertexType = sys::VertexType::VERTEX_32BITF
    .union(sys::VertexType::WEIGHT_32BITF)
    .union(sys::VertexType::WEIGHTS2)
    .union(sys::VertexType::INDEX_16BIT);
const MODEL_VERTEX_SIZE: VertexSize = const_vt_size(MODEL_VERTEX_TYPE);

const MODEL_VERTICES: &[Vert] = &[
    Vert {
        weight: [1.0, 0.0],
        position: [0.5, -0.5, 0.0],
    },
    Vert {
        weight: [0.0, 1.0],
        position: [0.0, 0.5, 0.0],
    },
    Vert {
        weight: [1.0, 0.0],
        position: [-0.5, -0.5, 0.0],
    },
];
const MODEL_INDICES: &[u16] = &[0, 1, 2];

psp_sys::module!("anim", 0, 1);

fn warn_unwrap(result: GlResult<()>) {
    result.unwrap_or_else(|err| dprint!("Warning: {err:?}"));
}

fn deg_to_rad(deg: f32) -> f32 {
    deg * (core::f32::consts::PI / 180.0)
}

fn psp_main() {
    enable_home_button();
    let mut gfx = Gfx::init(sys::TexturePixelFormat::Psm8888)
        .unwrap()
        .depth_test()
        .double_buffering()
        .culling()
        .scissor_test()
        .unwrap()
        .clip_planes()
        .texture_2d()
        .build()
        .unwrap();
    warn_unwrap(gfx.start_frame_with(|frame| {
        // Initial setup pass
        let gl = frame.gl_mut();
        gl.patch_division(2, 2);
        gl.blend_function(
            sys::BlendOp::Add,
            sys::BlendFactor::SrcAlpha,
            sys::BlendFactor::OneMinusSrcAlpha,
            u8::MAX as _,
            u8::MAX as _,
        );
        gl.set_state(sys::GuState::Blend, true);

        //gl.depth_test_function(sys::DepthFunc::);
        let mut perspective = Mat4::perspective_rh_gl(
            deg_to_rad(90.0), //90º
            16.0 / 9.0,
            0.8,
            // it has to be negative otherwise it wont work
            -0.8,
        );
        perspective.w_axis.z *= 0.9;
        gl.overwrite_projection_matrix(perspective);

        gl.set_matrix(MatrixMode::Texture, &Mat3By4::ZERO);

        gl.shading_model(sys::ShadingModel::Flat);

        gl.set_state(sys::GuState::Lighting, true);
        gl.light_mode(sys::LightMode::SeparateSpecularColor);
        Ok(())
    }));
    let emulated = Directory::open(c"ms0:/PSP/GAME/PSPDEV_EMU").is_ok();

    if emulated {
        dprint!("PSPDEV_EMU detected!");
    } else {
        dprint!("Real PSP!");
    }

    let mut frame_clock = FrameClock::default();

    loop {
        frame_clock = frame_clock.update();

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

            gl.set_matrix(
                MatrixMode::Model,
                &Mat3By4::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            );
            gl.set_bone_matrix(
                1,
                &Mat3By4::from_translation(Vec3::new(
                    f32::from(frame_clock.continous_clock(30)) * 0.2,
                    0.0,
                    0.0,
                )),
            );
            gl.morph_weight(0, 1.0);
            gl.morph_weight(1, 1.0);
            let v = gl.bind_indexed_vertices(
                MODEL_VERTEX_TYPE,
                MODEL_VERTEX_SIZE,
                MODEL_VERTICES,
                MODEL_INDICES,
            )?;
            gl.draw_primitives(v, MODEL_PRIMITIVE);
            Ok(())
        }));

        psp_apis::display::wait_vblank();
    }
}
