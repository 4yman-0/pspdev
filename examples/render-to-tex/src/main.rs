//! PSP test application.
#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

extern crate alloc;

use psp_apis::fs::{
    Directory,
	//File,
};
use psp_apis::gfx::{
    Gfx,
    color::Color32,
    gl::{Gl, GlResult, Mat3By4, MatrixMode},
    texture::Texture,
    vertex::{VertexSize, const_vt_size},
};

use glam::{Mat3, Mat4, Vec3};
use psp_sys::{dprint, enable_home_button, sys};

mod frame_clock;
use frame_clock::FrameClock;

#[repr(C)]
struct Vert {
    position: [f32; 3],
}

const TRIANGLE_PRIMITIVE: sys::GuPrimitive = sys::GuPrimitive::Triangles;
const TRIANGLE_VERTEX_TYPE: sys::VertexType = sys::VertexType::VERTEX_32BITF;
const TRIANGLE_VERTEX_SIZE: VertexSize = const_vt_size(TRIANGLE_VERTEX_TYPE);

const TRIANGLE_VERTICES: &[Vert] = &[
    Vert {
        position: [0.5, -0.5, 0.0],
    },
    Vert {
        position: [0.0, 0.5, 0.0],
    },
    Vert {
        position: [-0.5, -0.5, 0.0],
    },
];

#[repr(C)]
struct Vert16Tex8 {
    uv: [u8; 2],
    pos: [i16; 3],
}

const DISPLAYS_PRIMITIVE: sys::GuPrimitive = sys::GuPrimitive::Triangles;
const DISPLAYS_VERTEX_TYPE: sys::VertexType = sys::VertexType::VERTEX_16BIT
    .union(sys::VertexType::TEXTURE_8BIT)
    .union(sys::VertexType::INDEX_16BIT);
const DISPLAYS_VERTEX_SIZE: VertexSize = const_vt_size(DISPLAYS_VERTEX_TYPE);

const DISPLAYS_VERTICES: &[Vert16Tex8] = &[
    Vert16Tex8 {
        pos: [0, 0, 0],
        uv: [0, 128],
    },
    Vert16Tex8 {
        pos: [i16::MAX, 0, 0],
        uv: [128, 128],
    },
    Vert16Tex8 {
        pos: [i16::MAX, i16::MAX, 0],
        uv: [128, 0],
    },
    Vert16Tex8 {
        pos: [0, i16::MAX, 0],
        uv: [0, 0],
    },
];
const DISPLAYS_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

psp_sys::module!("gl", 0, 1);

const DISPLAY_TRANSFORMS: &[Mat3By4] = &[
    Mat3By4::from_scale_translation(
        Vec3::new(2.0, 1.0, 0.0),
        Vec3::new(-1.0, -0.5, -1.0),
    ),
    Mat3By4::from_scale_translation(
        Vec3::new(2.0, 1.0, 0.0),
        Vec3::new(-2.5, -1.7, -1.7),
    ),
    Mat3By4::from_scale_translation(
        Vec3::new(2.0, 1.0, 0.0),
        Vec3::new(0.5, 0.5, -1.5),
    ),
];

fn warn_unwrap(result: GlResult<()>) {
    result.unwrap_or_else(|err| dprint!("Warning: {err:?}"));
}

fn deg_to_rad(deg: f32) -> f32 {
    deg * (core::f32::consts::PI / 180.0)
}

fn set_frame_size(gl: &mut Gl, width: u16, height: u16) -> GlResult<()> {
    gl.draw_region(0, 0, width, height)?;
    gl.scissor_region(0, 0, width, height)?;
    gl.offset(2048 - (width as u32 / 2), 2048 - (height as u32 / 2));
    gl.viewport(2048.0, 2048.0, width as _, height as _);
    Ok(())
}

fn psp_main() {
    enable_home_button();
    let mut gfx = Gfx::init_default().unwrap();
    warn_unwrap(gfx.start_frame_with(|frame| {
        let gl = frame.gl_mut();
        let mut perspective = Mat4::perspective_rh_gl(
            deg_to_rad(90.0),
            16.0 / 9.0,
            1.0,
            -1.0,
        );
        gl.overwrite_projection_matrix(perspective);

	    gl.texture_filter(
	        sys::TextureFilter::Linear,
	        sys::TextureFilter::Linear,
	    );

        gl.shading_model(sys::ShadingModel::Flat);
        Ok(())
    }));
    let emulated = Directory::open(c"ms0:/PSP/GAME/PSPDEV_EMU").is_ok();

    if emulated {
        dprint!("PSPDEV_EMU detected!");
    } else {
        dprint!("Real PSP!");
    }

    let mut frame_clock = FrameClock::default();

    let mut texture =
        Texture::allocate(64, 32, sys::TexturePixelFormat::Psm5650, false);

    let mut yaw = 0_f32;

    loop {
        frame_clock = frame_clock.update();
        yaw += 0.07;
        if yaw > ::core::f32::consts::PI * 2.0 {
            yaw -= ::core::f32::consts::PI * 2.0;
        }

        warn_unwrap((|| {
            let gl = gfx.gl_mut();
            gl.list_mut().start();

            set_frame_size(gl, texture.width(), texture.height())?;
            gl.set_matrix(MatrixMode::View, &Mat3By4::IDENTITY);
            unsafe {
                gl.set_frame_buffer(&mut texture).unwrap();
            }
            gl.clear_color(Color32::DARK_GRAY);
            gl.clear(
                sys::ClearFlags::DEPTH_BUFFER_BIT
                    | sys::ClearFlags::COLOR_BUFFER_BIT,
                1,
            )?;

            gl.set_state(sys::GuState::CullFace, false);
            gl.clear_color(Color32::WHITE);
            gl.set_matrix(
                MatrixMode::Model,
                &Mat3By4::from_mat3_vec3(
                    Mat3::from_rotation_y(yaw)
                     * (Mat3::IDENTITY * 2.0),
                    Vec3::new(0.0, 0.3, -1.5),
                ),
            );
            let v = gl.bind_vertices(
                TRIANGLE_VERTEX_TYPE,
                TRIANGLE_VERTEX_SIZE,
                TRIANGLE_VERTICES,
            )?;
            gl.draw_primitives(v, TRIANGLE_PRIMITIVE);
            gl.set_state(sys::GuState::CullFace, true);

            set_frame_size(gl, 480, 272)?;

            gl.finish().unwrap();
            gl.list_mut()
                .sync(sys::GuSyncMode::List, sys::GuSyncBehavior::Wait);
            Ok(())
        })());

        warn_unwrap(gfx.start_frame_with(|frame| {
            let gl = frame.gl_mut();
            gl.clear_color(Color32::BLACK);
            gl.clear(
                sys::ClearFlags::DEPTH_BUFFER_BIT
                    | sys::ClearFlags::COLOR_BUFFER_BIT,
                1,
            )?;

            /*{
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
            }*/

            gl.clear_color(Color32::WHITE);
            gl.set_state(sys::GuState::Texture2D, true);
            gl.texture(sys::MipmapLevel::None, &texture);
            for m3b4 in DISPLAY_TRANSFORMS {
                gl.set_matrix(MatrixMode::Model, m3b4);
                let v = gl.bind_indexed_vertices(
                    DISPLAYS_VERTEX_TYPE,
                    DISPLAYS_VERTEX_SIZE,
                    DISPLAYS_VERTICES,
                    DISPLAYS_INDICES,
                )?;
                gl.draw_primitives(v, DISPLAYS_PRIMITIVE);
            }
            gl.set_state(sys::GuState::Texture2D, false);
            Ok(())
        }));

        psp_apis::display::wait_vblank_start();
    }
}
