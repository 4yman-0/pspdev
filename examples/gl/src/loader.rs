use aligned_box::AlignedBox;
use psp_apis::fs::File;
use psp_apis::gfx::{
    color::Color32,
    index::IndexItem,
    texture::{Texture, TextureResult, texture_size},
    vertex::{Vertex, VertexSize},
};
use psp_apis::kernel;
use psp_sys::{dprint, sys};

unsafe fn read_u32(file: &mut File) -> u32 {
    let mut buf: [u8; 4] = [0u8; 4];
    file.read_all(&mut buf).unwrap();
    u32::from_le_bytes(buf)
}

unsafe fn read_u16(file: &mut File) -> u16 {
    let mut buf: [u8; 2] = [0u8; 2];
    file.read_all(&mut buf).unwrap();
    u16::from_le_bytes(buf)
}

pub fn load_texture(file_name: &core::ffi::CStr) -> TextureResult<Texture> {
    let mut file = File::open(file_name, sys::IoOpenFlags::RD_ONLY).unwrap();

    let _file_size = file.seek_i32(0, sys::IoWhence::End) as usize;
    file.seek_i32(0, sys::IoWhence::Set);

    let format: u32 = unsafe { read_u32(&mut file) };
    let format: sys::TexturePixelFormat =
        unsafe { core::mem::transmute(format) };

    // TODO: this should be `mut`
    let swizzled: u8 = 0;
    file.read_all(&mut [swizzled]).unwrap();
    let swizzled = swizzled == 1;

    let width: usize = unsafe { read_u32(&mut file) as usize };
    let height: usize = unsafe { read_u32(&mut file) as usize };

    let mut ram_texture = alloc::boxed::Box::<[u8]>::new_uninit_slice(
        texture_size(width * height, format),
    );
    file.read_all(unsafe { ram_texture.assume_init_mut() })
        .unwrap();
    let ram_texture = unsafe { ram_texture.assume_init() };
    kernel::data_cache_writeback_invalidate(&ram_texture);

    let mut texture =
        Texture::allocate(width as u16, height as u16, format, swizzled)?;

    texture.buffer_mut().copy_from_slice(&ram_texture);
    kernel::data_cache_writeback_invalidate(texture.buffer());

    drop(ram_texture);
    drop(file);

    Ok(texture)
}

pub struct Model {
    pub primitive: sys::GuPrimitive,
    pub vertex_type: sys::VertexType,
    pub vertex_size: VertexSize,
    pub vertices: AlignedBox<[u8]>,
    pub indices: AlignedBox<[u8]>,
}

pub fn load_model(file_name: &core::ffi::CStr) -> Model {
    let mut file = File::open(file_name, sys::IoOpenFlags::RD_ONLY).unwrap();

    let _file_size = file.seek_i32(0, sys::IoWhence::End) as usize;
    file.seek_i32(0, sys::IoWhence::Set);

    let mut primitive = [0u8; 4];
    file.read_all(&mut primitive).unwrap();
    let primitive: sys::GuPrimitive =
        unsafe { core::mem::transmute(primitive) };

    let mut vertex_type = [0u8; 4];
    file.read_all(&mut vertex_type).unwrap();
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
    file.read_all(&mut vertices).unwrap();

    let mut indices = AlignedBox::<[u8]>::slice_from_default(
        vertex_type.index_size().next_power_of_two(),
        indices_size,
    )
    .unwrap();
    file.read_all(&mut indices).unwrap();

    drop(file);

    Model {
        primitive,
        vertex_type,
        vertex_size,
        vertices,
        indices,
    }
}

pub struct Shading {
    pub light_components: sys::LightComponent,
    pub real_ambient: Option<Color32>,
    pub ambient: Option<Color32>,
    pub diffuse: Option<Color32>,
    pub specular: Option<Color32>,
    pub specular_coeff: Option<f32>,
}

pub fn load_shading(file_name: &core::ffi::CStr) -> Shading {
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
    file.read_all(&mut specular_coeff).unwrap();
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
