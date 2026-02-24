//! PSP test application.
#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

extern crate alloc;

use psp_apis::display::wait_vblank_start;
use psp_apis::fs::{
    Directory,
    //self,
    //Path,
};
use psp_apis::gfx::{
    Gfx,
    color::Color32,
    gl::{GlResult, Mat3By4, MatrixMode},
    index::IndexItem,
    texture::{Texture, texture_pixel_size},
    vertex::{Vertex, VertexSize, const_vt_size},
};

//use alloc::{boxed::Box /*, vec::Vec*/};
use glam::{EulerRot, Mat3, Mat4, Vec2, Vec3};
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
        position: [ 0.0, 0.5, 0.0],
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
const DISPLAYS_INDICES: &[u16] = &[
	0, 1, 2,
	0, 2, 3,
];

psp_sys::module!("gl", 0, 1);

const fn matrix_3_by_4(matrix: &Mat3, translation: &Vec3) -> Mat3By4 {
    Mat3By4 {
        x: matrix.x_axis,
        y: matrix.y_axis,
        z: matrix.z_axis,
        w: *translation,
    }
}

fn warn_unwrap(result: GlResult<()>) {
    result.unwrap_or_else(|err| dprint!("Warning: {err:?}"));
}

fn deg_to_rad(deg: f32) -> f32 {
    deg * (core::f32::consts::PI / 180.0)
}

fn psp_main() {
    enable_home_button();
    dprint!("ENABLED HOME BUTTON");
    let mut gfx = Gfx::init()
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
    dprint!("GFX INIT SUCCESS");
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

        let texture = matrix_3_by_4(&Mat3::ZERO, &Vec3::ZERO);
        gl.set_matrix(MatrixMode::Texture, &texture);

        gl.shading_model(sys::ShadingModel::Flat);

        gl.set_state(sys::GuState::Lighting, true);
        gl.light_mode(sys::LightMode::SeparateSpecularColor);
        Ok(())
    }));
    dprint!("SETUP PASS SUCCESS");
    let emulated = Directory::open(c"ms0:/PSP/GAME/PSPDEV_EMU").is_ok();

    if emulated {
        dprint!("PSPDEV_EMU detected! This system is an illusion!");
    } else {
        dprint!("Real PSP!");
    }

    let mut frame_clock = FrameClock::default();

	let mut texture = Texture::allocate(
		gfx.vram_allocator_mut(),
		64,
		32,
		sys::TexturePixelFormat::Psm5650,
		false,
	).unwrap();

	let mut yaw = 0_f32;

    loop {
        wait_vblank_start();

        frame_clock = frame_clock.update();
		yaw += 0.05;
		if yaw > ::core::f32::consts::PI * 2.0 {
			yaw = 0.0;
		}

		warn_unwrap((|| {
			let gl = gfx.gl_mut();
			gl.list_mut().start();

			let (tw, th) = (texture.width(), texture.height());
			gl.draw_region(0, 0, tw, th)?;
			gl.scissor_region(0, 0, texture.width(), texture.height())?;
	        gl.offset(
	            2048 - (tw as usize / 2),
	            2048 - (th as usize / 2),
	        );
	        gl.viewport(
	            2048.0,
	            2048.0,
				tw as _,
				th as _,
	        );

            gl.set_matrix(MatrixMode::View, &matrix_3_by_4(
            	&Mat3::IDENTITY,
            	&Vec3::ZERO,
            ));
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
                &matrix_3_by_4(
                	&Mat3::from_euler(EulerRot::XYZEx, 0.0, yaw, 0.0),
                	&Vec3::new(0.0, 0.0, -1.0)
                ),
            );
            let v = gl.bind_vertices(
                TRIANGLE_VERTEX_TYPE,
                TRIANGLE_VERTEX_SIZE,
                TRIANGLE_VERTICES,
            )?;
            gl.draw_primitives(v, TRIANGLE_PRIMITIVE);
            gl.set_state(sys::GuState::CullFace, true);

			gl.draw_region(0, 0, 480, 272)?;
			gl.scissor_region(0, 0, 480, 272)?;
	        gl.offset(
	            2048 - (480_usize / 2),
	            2048 - (272_usize / 2),
	        );
	        gl.viewport(
	            2048.0,
	            2048.0,
				480.0,
				272.0,
	        );

            gl.finish().unwrap();
            gl.list_mut().sync(sys::GuSyncMode::List, sys::GuSyncBehavior::Wait);
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

            {
                // Gran Turismo jittering
                const JITTER: f32 = 0.5 / 272.0;
                let mut view = matrix_3_by_4(&Mat3::IDENTITY, &Vec3::ZERO);
                if frame_clock.edge_clock(2) {
                    view.w.x += JITTER;
                //}
                //if frame_clock.offset(1).edge_clock(2) {
                    view.w.y += JITTER;
                }
                gl.set_matrix(MatrixMode::View, &view)
            }

			gl.texture_filter(
				sys::TextureFilter::Linear,
				sys::TextureFilter::Linear,
			);
            gl.clear_color(Color32::WHITE);
            gl.set_state(sys::GuState::Texture2D, true);
            gl.texture(sys::MipmapLevel::None, &texture);
            for m3b4 in &[
				matrix_3_by_4(
                	&Mat3::from_scale(Vec2::new(2.0, 1.0)),
                	&Vec3::new(-1.0, -0.5, -1.0),
                ),
				matrix_3_by_4(
                	&Mat3::from_scale(Vec2::new(2.0, 1.0)),
                	&Vec3::new(-2.5, -1.7, -1.7),
                ),
				matrix_3_by_4(
                	&Mat3::from_scale(Vec2::new(2.0, 1.0)),
                	&Vec3::new(0.5, 0.5, -1.5),
                ),
            ] {
	            gl.set_matrix(
	                MatrixMode::Model,
	                m3b4,
	            );
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
    }
}
