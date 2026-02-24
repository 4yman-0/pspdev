//! PSP test application.
#![cfg(target_os = "psp")]
#![no_std]
#![no_main]

extern crate alloc;

mod renderer;
use renderer::render_tex_aabb;

use psp_apis::fs::{
    Directory,
    File,
    //self,
    //Path,
};
use psp_apis::gfx::{
    Gfx,
    color::Color32,
    gl::{GlResult, Mat3By4, MatrixMode},
    index::IndexItem,
    texture::{Texture, texture_pixel_size},
    vertex::{Vertex, VertexSize},
    vram_alloc::VramAllocator,
};
use psp_apis::kernel;
use psp_apis::{
    display::wait_vblank_start,
    input::{Buttons, Input},
};

use aligned_box::AlignedBox;
//use alloc::{boxed::Box /*, vec::Vec*/};
use glam::{EulerRot, Mat3, Mat4, Vec2, Vec3};
use psp_sys::{dprint, enable_home_button, sys};

mod frame_clock;
use frame_clock::FrameClock;

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

unsafe fn read_u32(file: &mut File) -> u32 {
    let mut buf: [u8; 4] = [0u8; 4];
    file.read(&mut buf).unwrap();
    u32::from_le_bytes(buf)
}

unsafe fn read_u16(file: &mut File) -> u16 {
    let mut buf: [u8; 2] = [0u8; 2];
    file.read(&mut buf).unwrap();
    u16::from_le_bytes(buf)
}

fn load_texture(
    file_name: &core::ffi::CStr,
    vram_allocator: &mut VramAllocator,
) -> Texture {
    let mut file = File::open(file_name, sys::IoOpenFlags::RD_ONLY).unwrap();

    let _file_size = file.seek_i32(0, sys::IoWhence::End) as usize;
    file.seek_i32(0, sys::IoWhence::Set);

    let format: u32 = unsafe { read_u32(&mut file) };
    let format: sys::TexturePixelFormat =
        unsafe { core::mem::transmute(format) };

    // TODO: this should be `mut`
    let swizzled: u8 = 0;
    file.read(&mut [swizzled]).unwrap();
    let swizzled = swizzled == 1;

    let width: u32 = unsafe { read_u32(&mut file) };
    let height: u32 = unsafe { read_u32(&mut file) };

    // PPSSPP weirdness
    let mut ram_texture = alloc::boxed::Box::<[u8]>::new_uninit_slice(
        (width * height) as usize * texture_pixel_size(format),
    );
    file.read(unsafe { ram_texture.assume_init_mut() }).unwrap();
    let ram_texture = unsafe { ram_texture.assume_init() };
    kernel::data_cache_writeback_invalidate(&ram_texture);

    let mut texture = Texture::allocate(
        vram_allocator,
        width as u16,
        height as u16,
        format,
        swizzled,
    )
    .unwrap();

    texture.buffer_mut().copy_from_slice(&ram_texture);
    kernel::data_cache_writeback_invalidate(texture.buffer());

    drop(ram_texture);
    drop(file);

    texture
}

struct Model {
    pub primitive: sys::GuPrimitive,
    pub vertex_type: sys::VertexType,
    pub vertex_size: VertexSize,
    pub vertices: AlignedBox<[u8]>,
    pub indices: AlignedBox<[u8]>,
}

fn load_model(file_name: &core::ffi::CStr) -> Model {
    let mut file = File::open(file_name, sys::IoOpenFlags::RD_ONLY).unwrap();

    let _file_size = file.seek_i32(0, sys::IoWhence::End) as usize;
    file.seek_i32(0, sys::IoWhence::Set);

    let mut primitive = [0u8; 4];
    file.read(&mut primitive).unwrap();
    let primitive: sys::GuPrimitive =
        unsafe { core::mem::transmute(primitive) };

    let mut vertex_type = [0u8; 4];
    file.read(&mut vertex_type).unwrap();
    let vertex_type =
        sys::VertexType::from_bits(i32::from_le_bytes(vertex_type)).unwrap();
    let vertex_size = vertex_type.vertex_size();

    let vertices_size = (unsafe { read_u32(&mut file) }) as usize;
    dprint!("verts total size: {vertices_size}");

    let indices_size = (unsafe { read_u32(&mut file) }) as usize;
    dprint!("idxs total size: {indices_size}");

    let mut vertices = AlignedBox::<[u8]>::slice_from_default(
        vertex_size.get().next_power_of_two(),
        vertices_size,
    )
    .unwrap();
    file.read(&mut vertices).unwrap();

    let mut indices = AlignedBox::<[u8]>::slice_from_default(
        vertex_type.index_size().next_power_of_two(),
        indices_size,
    )
    .unwrap();
    file.read(&mut indices).unwrap();

    drop(file);

    Model {
        primitive,
        vertex_type,
        vertex_size,
        vertices,
        indices,
    }
}

struct Shading {
    pub light_components: sys::LightComponent,
    pub real_ambient: Option<Color32>,
    pub ambient: Option<Color32>,
    pub diffuse: Option<Color32>,
    pub specular: Option<Color32>,
    pub specular_coeff: Option<f32>,
}

fn load_shading(file_name: &core::ffi::CStr) -> Shading {
    unsafe fn read_u16_as_bool(f: &mut File) -> bool {
        let flag = unsafe { read_u16(f) };
        flag == 1
    }

    let mut file = File::open(file_name, sys::IoOpenFlags::RD_ONLY).unwrap();

    let _file_size = file.seek_i32(0, sys::IoWhence::End) as usize;
    file.seek_i32(0, sys::IoWhence::Set);

    let use_real_ambient = unsafe { read_u16_as_bool(&mut file) };

    let real_ambient = unsafe { read_u32(&mut file) };
    let real_ambient = Color32::from_rgba(real_ambient);

    let use_ambient = unsafe { read_u16_as_bool(&mut file) };

    let ambient = unsafe { read_u32(&mut file) };
    let ambient = Color32::from_rgba(ambient);

    let use_diffuse = unsafe { read_u16_as_bool(&mut file) };

    let diffuse = unsafe { read_u32(&mut file) };
    let diffuse = Color32::from_rgba(diffuse);

    let use_specular = unsafe { read_u16_as_bool(&mut file) };

    let specular = unsafe { read_u32(&mut file) };
    let specular = Color32::from_rgba(specular);

    let use_specular_coeff = unsafe { read_u16_as_bool(&mut file) };

    let mut specular_coeff = [0u8; 4];
    file.read(&mut specular_coeff).unwrap();
    let specular_coeff = f32::from_le_bytes(specular_coeff);

    drop(file);

    let mut light_components = sys::LightComponent::empty();

    //if use_ambient {
    light_components |= sys::LightComponent::AMBIENT;
    //}

    if use_diffuse {
        light_components |= sys::LightComponent::DIFFUSE;
    }

    if use_specular {
        light_components |= sys::LightComponent::SPECULAR;
    }

    Shading {
        light_components,
        real_ambient: use_real_ambient.then_some(real_ambient),
        ambient: use_ambient.then_some(ambient),
        diffuse: use_diffuse.then_some(diffuse),
        specular: use_specular.then_some(specular),
        specular_coeff: use_specular_coeff.then_some(specular_coeff),
    }
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
            0,
            0,
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
    dprint!("SETUP PASS SUCCESS");
    let mut input = Input::init(sys::CtrlMode::Analog);

    let emulated = Directory::open(c"ms0:/PSP/GAME/PSPDEV_EMU").is_ok();

    if emulated {
        dprint!("PSPDEV_EMU detected! This system is an illusion!");
    } else {
        dprint!("Real PSP!");
    }

    let model = load_model(asset!(emulated, "test.pspm"));

    let shading = load_shading(asset!(emulated, "test.psps"));

    let texture =
        load_texture(asset!(emulated, "test.pspt"), gfx.vram_allocator_mut());

    let lamp_texture =
        load_texture(asset!(emulated, "lamp.pspt"), gfx.vram_allocator_mut());

    let mut translation = Vec3::new(0.0, 0.0, -2.0);
    let mut rotation = Vec3::ZERO;
    let mut frame_clock = FrameClock::default();

    loop {
        wait_vblank_start();

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
                let mut view = matrix_3_by_4(&Mat3::IDENTITY, &Vec3::ZERO);
                if frame_clock.edge_clock(2) {
                    view.w.x += JITTER;
                }
                if frame_clock.offset(1).edge_clock(2) {
                    view.w.y += JITTER;
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
                &matrix_3_by_4(
                    &(Mat3::from_scale(Vec2::new(1.0, 1.0))
                        * Mat3::from_euler(
                            EulerRot::XYZEx,
                            rotation.x,
                            rotation.y,
                            rotation.z,
                        )),
                    &translation,
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
    }
}
