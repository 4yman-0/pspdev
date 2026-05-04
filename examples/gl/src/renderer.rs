use psp_apis::gfx::{
    gl::{Gl, GlResult, Mat3By4, MatrixMode},
    texture::Texture,
    vertex::{VertexSize, const_vt_size},
};

use glam::{Mat3, Vec2, Vec3};
use psp_sys::sys;

pub(crate) const fn matrix_3_by_4(matrix: Mat3, translation: Vec3) -> Mat3By4 {
    Mat3By4 {
        x_axis: matrix.x_axis,
        y_axis: matrix.y_axis,
        z_axis: matrix.z_axis,
        w_axis: translation,
    }
}

#[repr(C)]
#[allow(dead_code)]
struct Vert16 {
    vertex: [i16; 3],
}

impl Vert16 {
    const VERT_TYPE: sys::VertexType = sys::VertexType::VERTEX_16BIT;
    const VERT_SIZE: VertexSize = const_vt_size(Self::VERT_TYPE);
}

static RENDER_AABB_VERTS: &[Vert16] = &[
    // front
    Vert16 { vertex: [0, 0, 0] },
    Vert16 {
        vertex: [i16::MAX, i16::MAX, 0],
    },
];

#[repr(C)]
#[allow(dead_code)]
//#[repr(C)]
struct Vert16Tex8 {
    uv: [u8; 2],
    vertex: [i16; 3],
}

impl Vert16Tex8 {
    const VERT_TYPE: sys::VertexType =
        sys::VertexType::VERTEX_16BIT.union(sys::VertexType::TEXTURE_8BIT);
    const VERT_SIZE: VertexSize = const_vt_size(Self::VERT_TYPE);
}

static RENDER_AABB_TEX_VERTS: &[Vert16Tex8] = &[
    // front
    Vert16Tex8 {
        vertex: [0, 0, 0],
        uv: [0, 0],
    },
    Vert16Tex8 {
        vertex: [i16::MAX, i16::MAX, 0],
        uv: [128, 128],
    },
];

pub(crate) fn render_aabb(
    gl: &mut Gl,
    position: Vec3,
    size: Vec2,
) -> GlResult<()> {
    gl.set_matrix(
        MatrixMode::Model,
        &matrix_3_by_4(Mat3::from_scale(size), position),
    );
    let v = gl.bind_vertices(
        Vert16::VERT_TYPE,
        Vert16::VERT_SIZE,
        RENDER_AABB_VERTS,
    )?;
    gl.draw_primitives(v, sys::GuPrimitive::Sprites);
    Ok(())
}

pub(crate) fn render_tex_aabb(
    gl: &mut Gl,
    texture: &Texture,
    position: Vec3,
    size: Vec2,
) -> GlResult<()> {
    gl.set_state(sys::GuState::Texture2D, true);
    gl.texture(sys::MipmapLevel::None, texture);
    gl.set_matrix(
        MatrixMode::Model,
        &matrix_3_by_4(Mat3::from_scale(size), position),
    );
    let v = gl.bind_vertices(
        Vert16Tex8::VERT_TYPE,
        Vert16Tex8::VERT_SIZE,
        RENDER_AABB_TEX_VERTS,
    )?;
    gl.draw_primitives(v, sys::GuPrimitive::Sprites);
    gl.set_state(sys::GuState::Texture2D, false);
    Ok(())
}
