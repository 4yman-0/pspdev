// ## Convention
// - GL uses pointers from system memory to refer to VRAM.
// - GL uses `usize` whenever possible.

use super::{
    color::Color32,
    index::IndexItem,
    texture::Texture,
    vertex::{VertexSize, const_vt_size},
};
use crate::error::{
    NativeError, NativeResult, native_error, /*native_result*/
};
use alloc::boxed::Box;
use core::ffi::c_void;
use core::mem::MaybeUninit;
use psp_sys::sys::{self, DisplayPixelFormat, GeCommand as GeCmd};

pub type GlResult<T> = Result<T, GlError>;

#[derive(Clone, Debug)]
pub enum GlError {
    Native(NativeError),

    InvalidScissorRegion,
    InvalidDrawRegion,
    VertexTypeContainsIndex,
    //VertexTypeDoesNotContainIndex,
    InvalidVertexArraySize,
    InvalidIndexArraySize,
    InvalidClutFormat,
    InvalidFramebuffer,
    //ListEmpty,
    ListNotEmpty,
}

impl core::fmt::Display for GlError {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), core::fmt::Error> {
        match self {
            Self::Native(err) => write!(f, "Gl error: {err}"),
            _ => write!(f, "GL error: {self:?}"),
        }
    }
}
impl core::error::Error for GlError {}

impl From<NativeError> for GlError {
    fn from(from: NativeError) -> Self {
        Self::Native(from)
    }
}

use glam::{I8Vec4, Mat4, Vec3, Vec4};

#[derive(Clone)]
pub struct Mat3By4 {
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
    pub w_axis: Vec3,
}

impl Mat3By4 {
    pub const IDENTITY: Self = Self {
        x_axis: Vec3::new(1.0, 0.0, 0.0),
        y_axis: Vec3::new(0.0, 1.0, 0.0),
        z_axis: Vec3::new(0.0, 0.0, 1.0),
        w_axis: Vec3::ZERO,
    };
    pub const ZERO: Self = Self {
        x_axis: Vec3::ZERO,
        y_axis: Vec3::ZERO,
        z_axis: Vec3::ZERO,
        w_axis: Vec3::ZERO,
    };
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    #[must_use]
    const fn new(
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
        m30: f32,
        m31: f32,
        m32: f32,
    ) -> Self {
        Self {
            x_axis: Vec3::new(m00, m01, m02),
            y_axis: Vec3::new(m10, m11, m12),
            z_axis: Vec3::new(m20, m21, m22),
            w_axis: Vec3::new(m30, m31, m32),
        }
    }
    #[inline(always)]
    #[must_use]
	pub const fn from_cols(
		x_axis: Vec3,
		y_axis: Vec3,
		z_axis: Vec3,
		w_axis: Vec3,
	) -> Self {
		Self {
			x_axis,
			y_axis,
			z_axis,
			w_axis,
		}
	}
    #[inline(always)]
    #[must_use]
    pub const fn from_scale(scale: Vec3) -> Self {
    	Self::new(
    		scale.x, 0.0, 0.0,
    		0.0, scale.y, 0.0,
    		0.0, 0.0, scale.z,
    		0.0, 0.0, 0.0,
    	)
    }
    #[inline(always)]
    #[must_use]
    const fn v4_to_v3(v: Vec4) -> Vec3 {
    	Vec3 {
    		x: v.x,
    		y: v.y,
    		z: v.z,
    	}
    }
    #[inline(always)]
    #[must_use]
    pub const fn from_mat4(m: Mat4) -> Self {
    	Self::from_cols(
			Self::v4_to_v3(m.x_axis),
			Self::v4_to_v3(m.y_axis),
			Self::v4_to_v3(m.z_axis),
			Self::v4_to_v3(m.w_axis),
    	)
    }
}

#[derive(Clone)]
pub struct I8Mat4 {
    pub x: I8Vec4,
    pub y: I8Vec4,
    pub z: I8Vec4,
    pub w: I8Vec4,
}

impl AsRef<[f32; 12]> for Mat3By4 {
    #[inline]
    fn as_ref(&self) -> &[f32; 12] {
        unsafe { &*(&raw const *self).cast() }
    }
}
impl AsMut<[f32; 12]> for Mat3By4 {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 12] {
        unsafe { &mut *(&raw mut *self).cast() }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MatrixMode {
    View,
    Model,
    Texture,
}

/// splits a 32-bit address into:
/// - the least significant 24 bits
/// - the most significant 8 bits
#[inline]
#[must_use]
const fn split_address(ptr: usize) -> (usize, u8) {
    (ptr & 0xffffff, (ptr >> 24) as u8)
}

fn into_bytes<T>(slice: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            slice.as_ptr().cast::<u8>(),
            size_of_val(slice),
        )
    }
}

pub struct ListHandle {
    // `list` should not re-allocate while being written to or
    // its address will change silently during rendering
    list: Box<[MaybeUninit<usize>]>,
}

impl core::fmt::Debug for ListHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ListHandle;")
    }
}

const GE_LIST_SIZE: usize = 0x4000;

impl ListHandle {
    #[must_use]
    pub fn new() -> Self {
        let list = unsafe {
            let raw = alloc::alloc::alloc(
                core::alloc::Layout::from_size_align(
                    GE_LIST_SIZE * size_of::<usize>(),
                    16,
                )
                .unwrap(),
            );
            Box::<[MaybeUninit<usize>]>::from_raw({
                &raw mut *core::slice::from_raw_parts_mut(
                    raw.cast(),
                    GE_LIST_SIZE,
                )
            })
        };

        Self { list }
    }
    pub fn start(&mut self) {
        unsafe {
            sys::sceGuStart(
                sys::GuContextType::Direct,
                self.list.as_mut_ptr().cast(),
            );
        };
    }
    pub fn stall(&mut self) {
        unsafe {
            sys::sceGuCommandStall();
        };
    }
    pub fn sync(
        &mut self,
        sync_mode: sys::GuSyncMode,
        sync_behavior: sys::GuSyncBehavior,
    ) {
        unsafe {
            sys::sceGuSync(sync_mode, sync_behavior);
        };
    }
    pub fn ge_break(&mut self, reset: bool) -> NativeResult<()> {
        let mut unused_break_param =
            core::mem::MaybeUninit::<sys::GeBreakParam>::uninit();
        native_error(unsafe {
            sys::sceGeBreak(
                i32::from(reset),
                unused_break_param.assume_init_mut(),
            )
        })
    }
    pub fn ge_continue(&mut self) {
        unsafe {
            sys::sceGeContinue();
        };
    }
    #[must_use]
    #[inline]
    pub fn list_mut(&mut self) -> &mut [usize] {
        unsafe { self.list.assume_init_mut() }
    }
    #[must_use]
    #[inline]
    pub fn list(&mut self) -> &[usize] {
        unsafe { self.list.assume_init_ref() }
    }
    #[inline]
    pub fn send(&mut self, id: GeCmd, data: usize) {
        unsafe {
            sys::sceGuSendCommandi(id, data.cast_signed() as i32);
        };
    }
    #[inline]
    pub fn send_parts(&mut self, id: GeCmd, data1: u8, data2: u16) {
        self.send(id, ((data1 as usize) << 16) | data2 as usize);
    }
    #[inline]
    pub fn send_float(&mut self, id: GeCmd, data: f32) {
        unsafe {
            sys::sceGuSendCommandf(id, data);
        };
    }
}

impl Default for ListHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
const fn state_to_command(state: sys::GuState) -> GeCmd {
    use sys::GuState;
    match state {
        GuState::AlphaTest => GeCmd::AlphaTestEnable,
        GuState::DepthTest => GeCmd::ZTestEnable,
        GuState::StencilTest => GeCmd::StencilTestEnable,
        GuState::Blend => GeCmd::AlphaBlendEnable,
        GuState::CullFace => GeCmd::CullFaceEnable,
        GuState::Dither => GeCmd::DitherEnable,
        GuState::Fog => GeCmd::FogEnable,
        GuState::ClipPlanes => GeCmd::DepthClampEnable,
        GuState::Texture2D => GeCmd::TextureMapEnable,
        GuState::Lighting => GeCmd::LightingEnable,
        GuState::Light0 => GeCmd::LightEnable0,
        GuState::Light1 => GeCmd::LightEnable1,
        GuState::Light2 => GeCmd::LightEnable2,
        GuState::Light3 => GeCmd::LightEnable3,
        GuState::LineSmooth => GeCmd::AntiAliasEnable,
        GuState::PatchCullFace => GeCmd::PatchCullEnable,
        GuState::ColorTest => GeCmd::ColorTestEnable,
        GuState::ColorLogicOp => GeCmd::LogicOpEnable,
        GuState::FaceNormalReverse => GeCmd::ReverseNormal,
        GuState::PatchFace => GeCmd::PatchFacing,
        GuState::ScissorTest => {
            panic!("Do not enable ScissorTest this way")
        }
        GuState::Fragment2X => panic!("Do not enable Fragment2X this way"),
    }
}

pub struct MatrixCommandCache {
    projection: Mat4,
    view: Mat3By4,
    model: Mat3By4,
    texture: Mat3By4,
}

impl Default for MatrixCommandCache {
    fn default() -> Self {
        Self {
            projection: Mat4::IDENTITY,
            view: Mat3By4::IDENTITY,
            model: Mat3By4::IDENTITY,
            texture: Mat3By4::IDENTITY,
        }
    }
}

impl MatrixCommandCache {
    pub fn test(
        &mut self,
        matrix_mode: MatrixMode,
        index: usize,
        value: f32,
    ) -> bool {
        let matrix: &mut [f32] = match matrix_mode {
            MatrixMode::View => &mut self.view,
            MatrixMode::Model => &mut self.model,
            MatrixMode::Texture => &mut self.texture,
        }
        .as_mut();
        if matrix[index] != value {
            matrix[index] = value;
            true
        } else {
            false
        }
    }
    pub fn test_projection(&mut self, index: usize, value: f32) -> bool {
        let matrix = self.projection.as_mut();
        if matrix[index] != value {
            matrix[index] = value;
            true
        } else {
            false
        }
    }
    pub fn overwrite(&mut self, matrix_mode: MatrixMode, matrix: Mat3By4) {
        match matrix_mode {
            MatrixMode::View => self.view = matrix,
            MatrixMode::Model => self.model = matrix,
            MatrixMode::Texture => self.texture = matrix,
        }
    }
    pub fn overwrite_projection(&mut self, matrix: Mat4) {
        self.projection = matrix;
    }
}

#[derive(Clone, Copy)]
pub struct VertexCount(u16);

impl VertexCount {
    pub fn get(self) -> u16 {
        self.0
    }
}

pub struct Gl {
    list: ListHandle,
    display_size: Option<(u16, u16)>,
    matrix_command_cache: MatrixCommandCache,
}

impl Default for Gl {
    fn default() -> Self {
        Self::new()
    }
}

impl Gl {
    #[must_use]
    pub fn new() -> Self {
        Self {
            list: ListHandle::new(),
            display_size: None,
            matrix_command_cache: MatrixCommandCache::default(),
        }
    }
    #[inline]
    pub fn list(&self) -> &ListHandle {
        &self.list
    }
    #[inline]
    pub fn list_mut(&mut self) -> &mut ListHandle {
        &mut self.list
    }
    /// # Safety
    /// This function uses a raw pointer (`depth.buffer`)
    pub unsafe fn set_depth_buffer(
        &mut self,
        depth: &mut Texture,
    ) -> GlResult<()> {
        use sys::TexturePixelFormat as TexelFmt;
        if !matches!(
            depth.format(),
            TexelFmt::Psm4444 | TexelFmt::Psm5650 | TexelFmt::Psm5551
        ) {
            return Err(GlError::InvalidFramebuffer);
        }
        /*let (depth1, depth2) = split_address(unsafe {
            depth
                .buffer
                .byte_offset_from_unsigned(sys::sceGeEdramGetAddr())
        });
        self.list.send(GeCmd::ZBufPtr, depth1);
        self.list.send_parts(GeCmd::ZBufWidth, depth2, depth.width);*/
        unsafe {
            sys::sceGuDepthBuffer(
                depth.buffer_mut().as_mut_ptr().cast(),
                depth.width() as i32,
            );
        };
        Ok(())
    }
    /// # Safety
    /// This function uses a raw pointer (`frame.buffer`)
    pub unsafe fn set_frame_buffer(
        &mut self,
        frame: &mut Texture,
    ) -> GlResult<()> {
        if !frame.can_be_framebuffer() {
            return Err(GlError::InvalidFramebuffer);
        }
        let (frame1, frame2) = split_address(unsafe {
            frame
                .buffer()
                .as_ptr()
                .byte_offset_from_unsigned(sys::sceGeEdramGetAddr())
        });
        self.list
            .send(GeCmd::FramebufPixFormat, frame.format() as usize);
        self.list.send(GeCmd::FrameBufPtr, frame1);
        self.list
            .send_parts(GeCmd::FrameBufWidth, frame2, frame.width());
        Ok(())
    }
    pub fn set_display_buffer(&mut self, display: &Texture) -> GlResult<()> {
        if !display.can_be_framebuffer() {
            return Err(GlError::InvalidFramebuffer);
        }
        unsafe {
            sys::sceDisplaySetFrameBuf(
                display.buffer().as_ptr(),
                display.width() as usize,
                core::mem::transmute::<
                    sys::TexturePixelFormat,
                    sys::DisplayPixelFormat,
                >(display.format()),
                sys::DisplaySetBufSync::NextFrame,
            );
        };
        Ok(())
    }
    pub fn set_display_size(&mut self, width: u16, height: u16) {
        unsafe {
            sys::sceDisplaySetMode(
                sys::DisplayMode::Lcd,
                width as usize,
                height as usize,
            );
        };
        self.display_size = Some((width, height));
    }
    #[must_use]
    #[inline]
    pub fn display_size(&self) -> &Option<(u16, u16)> {
        &self.display_size
    }
    pub fn remove_display_buffer(&mut self) {
        // Safety: sceDisplaySetFrameBuf interprets `null_mut`
        // as disabling the display buffer
        unsafe {
            sys::sceDisplaySetFrameBuf(
                core::ptr::null_mut(),
                0,
                DisplayPixelFormat::Psm5650,
                sys::DisplaySetBufSync::NextFrame,
            );
        }
    }
    pub fn draw_region(
        &mut self,
        start_x: u16,
        start_y: u16,
        end_x: u16,
        end_y: u16,
    ) -> GlResult<()> {
        // Both these command take 10-bit integers
        // x: 0-9, y: 10-19
        // Scissor1 and 2 are similar

        if !(start_x >> 10 == 0
            && start_y >> 10 == 0
            && end_x >> 10 == 0
            && end_y >> 10 == 0)
        {
            return Err(GlError::InvalidDrawRegion);
        }
        self.list.send(
            GeCmd::Region1,
            ((start_y as usize) << 10) | start_x as usize,
        );
        let (end_x, end_y) = (end_x as usize - 1, end_y as usize - 1); // offset by one
        self.list.send(GeCmd::Region2, (end_y << 10) | end_x);
        Ok(())
    }
    pub fn scissor_region(
        &mut self,
        start_x: u16,
        start_y: u16,
        end_x: u16,
        end_y: u16,
    ) -> GlResult<()> {
        if !(start_x >> 10 == 0
            && start_y >> 10 == 0
            && end_x >> 10 == 0
            && end_y >> 10 == 0)
        {
            return Err(GlError::InvalidScissorRegion);
        }
        self.list.send(
            GeCmd::Scissor1,
            ((start_y as usize) << 10) | start_x as usize,
        );
        let (end_x, end_y) = (end_x as usize - 1, end_y as usize - 1);
        self.list.send(GeCmd::Scissor2, (end_y << 10) | end_x);
        Ok(())
    }
    pub fn depth_test_function(&mut self, function: sys::DepthFunc) {
        self.list.send(GeCmd::ZTest, function as usize);
    }
    pub fn mask_depth_writes(&mut self, mask: bool) {
        self.list.send(GeCmd::ZWriteDisable, usize::from(mask));
    }
    pub fn depth_range(&mut self, near: u16, far: u16, offset: usize) {
        // lower means farther, higher means closer
        let (near, far) = (near as usize, far as usize);
        /*assert!(
            far < near,
            "Depth far limit should not be higher than near limit"
        );*/
        let max = near + far;
        let val = ((max >> 31) + max) as f32;
        let z = val / 2.0;

        self.list.send_float(GeCmd::ViewportZScale, z - near as f32);
        self.list
            .send_float(GeCmd::ViewportZCenter, z + offset as f32);

        self.list.send(GeCmd::MaxZ, near);
        self.list.send(GeCmd::MinZ, far);
    }
    pub fn viewport_z(&mut self, scale: f32, center: f32) {
        self.list.send_float(GeCmd::ViewportZScale, scale);
        self.list.send_float(GeCmd::ViewportZCenter, center);
    }
    pub fn front_face_direction(&mut self, direction: sys::FrontFaceDirection) {
        use sys::FrontFaceDirection;
        self.list.send(
            GeCmd::Cull,
            match direction {
                FrontFaceDirection::CounterClockwise => 0,
                FrontFaceDirection::Clockwise => 1,
            },
        );
    }
    /// # Panics
    /// panics if a division-by-zero error occurs when calculating
    pub fn fog(&mut self, near: f32, far: f32, color: Color32) {
        let distance = 1.0 / (far - near);

        let color = color.as_abgr() as usize;
        self.list.send(GeCmd::FogColor, color & 0xffffff);
        self.list.send_float(GeCmd::Fog1, far);
        self.list.send_float(GeCmd::Fog2, distance);
    }
    pub fn finish(&mut self) -> NativeResult<()> {
        unsafe {
            sys::sceGuFinish();
        };
        crate::kernel::data_cache_writeback_invalidate(&self.list);
        Ok(())
    }
    /// ## Safety
    /// The called list must eventually return
    /// otherwise we have a problem
    pub unsafe fn call_list(&mut self, list: &'static [usize]) {
        let list_addr = list.as_ptr().addr();
        let (ptr1, ptr2) = split_address(list_addr);
        self.list.send(GeCmd::Base, ptr2 as usize);
        self.list.send(GeCmd::Call, ptr1);
    }
    pub fn bind_vertices<T>(
        &mut self,
        vertex_type: sys::VertexType,
        vertex_size: VertexSize,
        vertices: &[T],
    ) -> GlResult<VertexCount> {
        let vertex_size = vertex_size.get();
        let vertices = into_bytes(vertices);
        if !vertices.len().is_multiple_of(vertex_size) || vertex_size == 0 {
            psp_sys::dprint!("should be {vertex_size}");
            return Err(GlError::InvalidVertexArraySize);
        }
        if vertex_type.intersects(
            sys::VertexType::INDEX_8BIT.union(sys::VertexType::INDEX_16BIT),
        ) {
            return Err(GlError::VertexTypeContainsIndex);
        }
        self.list.send(
            GeCmd::VertexType,
            vertex_type.bits().cast_unsigned() as usize,
        );

        let vertices_addr = vertices.as_ptr().addr();
        // 4 most significant bits for address (28 total)
        self.list.send(GeCmd::Base, (vertices_addr >> 8) & 0xf0000);
        self.list.send(GeCmd::Vaddr, vertices_addr & 0xffffff);
        Ok(VertexCount((vertices.len() / vertex_size) as u16))
    }

    pub fn bind_indexed_vertices<T, U>(
        &mut self,
        vertex_type: sys::VertexType,
        vertex_size: VertexSize,
        vertices: &[T],
        indices: &[U],
    ) -> GlResult<VertexCount> {
        let vertex_size = vertex_size.get();
        let vertices = into_bytes(vertices);
        let indices = into_bytes(indices);
        if !vertices.len().is_multiple_of(vertex_size) || vertex_size == 0 {
            psp_sys::dprint!("should be {vertex_size}");
            return Err(GlError::InvalidVertexArraySize);
        }
        let index_size = vertex_type.index_size();
        if !indices.len().is_multiple_of(index_size) || index_size == 0 {
            return Err(GlError::InvalidIndexArraySize);
        }

        self.list.send(
            GeCmd::VertexType,
            vertex_type.bits().cast_unsigned() as usize,
        );

        let indices_addr = indices.as_ptr().addr();
        self.list.send(GeCmd::Base, (indices_addr >> 8) & 0xf0000);
        self.list.send(GeCmd::Iaddr, indices_addr & 0xffffff);
        let vertices_addr = vertices.as_ptr().addr();
        self.list.send(GeCmd::Base, (vertices_addr >> 8) & 0xf0000);
        self.list.send(GeCmd::Vaddr, vertices_addr & 0xffffff);

        Ok(VertexCount((indices.len() / index_size) as u16))
    }
    pub fn draw_primitives(
        &mut self,
        vertex_count: VertexCount,
        prim: sys::GuPrimitive,
    ) {
        self.list
            .send_parts(GeCmd::Prim, prim as u8, vertex_count.get());
        self.list.stall();
    }
    pub fn draw_bounding_box(&mut self, vertex_count: VertexCount) {
        self.list
            .send(GeCmd::BoundingBox, vertex_count.get().into());
        self.list.stall();
    }
    pub fn end_bounding_box(&mut self) {
        /*let (ptr1, ptr2) = split_address(unsafe {
            (&raw const self.list.last()).addr()
        });
        self.list.send(GeCmd::Base, ptr2.into());
        self.list.send(GeCmd::BJump, ptr1);*/
        todo!();
    }
    pub fn draw_bezier(&mut self, u_vertices: u8, v_vertices: u8) {
        self.list.send(
            GeCmd::Bezier,
            ((v_vertices as usize) << 8) | (u_vertices as usize),
        );
    }
    pub fn draw_spline(
        &mut self,
        u_vertices: u8,
        v_vertices: u8,
        u_edge: u8,
        v_edge: u8,
    ) {
        self.list.send(
            GeCmd::Spline,
            ((v_edge as usize) << 18)
                | ((u_edge as usize) << 16)
                | ((v_vertices as usize) << 8)
                | (u_vertices as usize),
        );
    }
    pub fn set_state(&mut self, state: sys::GuState, enable: bool) {
        self.list.send(state_to_command(state), usize::from(enable));
    }
    /// # Panics
    /// panics if the light index is higher than 3 (lights start from zero)
    pub fn light_position(&mut self, light: u8, vec: &Vec3) {
        self.list.send_float(
            match light {
                0 => GeCmd::Light0X,
                1 => GeCmd::Light1X,
                2 => GeCmd::Light2X,
                3 => GeCmd::Light3X,
                _ => panic!("Only four lights are supported"),
            },
            vec.x,
        );
        self.list.send_float(
            match light {
                0 => GeCmd::Light0Y,
                1 => GeCmd::Light1Y,
                2 => GeCmd::Light2Y,
                3 => GeCmd::Light3Y,
                _ => unreachable!(),
            },
            vec.y,
        );
        self.list.send_float(
            match light {
                0 => GeCmd::Light0Z,
                1 => GeCmd::Light1Z,
                2 => GeCmd::Light2Z,
                3 => GeCmd::Light3Z,
                _ => unreachable!(),
            },
            vec.z,
        );
    }
    pub fn light_direction(&mut self, light: u8, vec: &Vec3) {
        self.list.send_float(
            match light {
                0 => GeCmd::Light0DirectionX,
                1 => GeCmd::Light1DirectionX,
                2 => GeCmd::Light2DirectionX,
                3 => GeCmd::Light3DirectionX,
                _ => panic!("Only four lights are supported"),
            },
            vec.x,
        );
        self.list.send_float(
            match light {
                0 => GeCmd::Light0DirectionY,
                1 => GeCmd::Light1DirectionY,
                2 => GeCmd::Light2DirectionY,
                3 => GeCmd::Light3DirectionY,
                _ => unreachable!(),
            },
            vec.y,
        );
        self.list.send_float(
            match light {
                0 => GeCmd::Light0DirectionZ,
                1 => GeCmd::Light1DirectionZ,
                2 => GeCmd::Light2DirectionZ,
                3 => GeCmd::Light3DirectionZ,
                _ => unreachable!(),
            },
            vec.z,
        );
    }
    /// # Panics
    /// panics if the light index is higher than 3 (lights start from zero)
    pub fn light_type_components(
        &mut self,
        light: u8,
        light_type: sys::LightType,
        components: sys::LightComponent,
    ) {
        let kind = if components.bits() == 8 {
            2
        } else {
            usize::from(components.bits() ^ 6 < 1)
        };
        self.list.send(
            match light {
                0 => GeCmd::LightType0,
                1 => GeCmd::LightType1,
                2 => GeCmd::LightType2,
                3 => GeCmd::LightType3,
                _ => panic!("Only four lights are supported"),
            },
            ((light_type as usize & 0x03) << 8) | kind,
        );
    }
    /// # Panics
    /// panics if the light index is higher than 3 (lights start from zero)
    pub fn light_color(
        &mut self,
        light: u8,
        component: sys::LightComponent,
        color: Color32,
    ) {
        use sys::LightComponent;
        let color = (color.as_abgr() & 0xffffff) as usize;
        if component.intersects(LightComponent::AMBIENT) {
            self.list.send(
                match light {
                    0 => GeCmd::Light0Ambient,
                    1 => GeCmd::Light1Ambient,
                    2 => GeCmd::Light2Ambient,
                    3 => GeCmd::Light3Ambient,
                    _ => panic!("Only four lights are supported"),
                },
                color,
            );
        }
        if component.intersects(LightComponent::DIFFUSE) {
            self.list.send(
                match light {
                    0 => GeCmd::Light0Diffuse,
                    1 => GeCmd::Light1Diffuse,
                    2 => GeCmd::Light2Diffuse,
                    3 => GeCmd::Light3Diffuse,
                    _ => unreachable!(),
                },
                color,
            );
        }
        if component.intersects(LightComponent::SPECULAR) {
            self.list.send(
                match light {
                    0 => GeCmd::Light0Specular,
                    1 => GeCmd::Light1Specular,
                    2 => GeCmd::Light2Specular,
                    3 => GeCmd::Light3Specular,
                    _ => unreachable!(),
                },
                color,
            );
        }
    }
    /// # Panics
    /// panics if the light index is higher than 3 (lights start from zero)
    pub fn light_attenuation(
        &mut self,
        light: u8,
        constant: f32,
        linear: f32,
        quadratic: f32,
        exponent: f32,
    ) {
        self.list.send_float(
            match light {
                0 => GeCmd::Light0ConstantAtten,
                1 => GeCmd::Light1ConstantAtten,
                2 => GeCmd::Light2ConstantAtten,
                3 => GeCmd::Light3ConstantAtten,
                _ => panic!("Only four lights are supported"),
            },
            constant,
        );
        self.list.send_float(
            match light {
                0 => GeCmd::Light0LinearAtten,
                1 => GeCmd::Light1LinearAtten,
                2 => GeCmd::Light2LinearAtten,
                3 => GeCmd::Light3LinearAtten,
                _ => unreachable!(),
            },
            linear,
        );
        self.list.send_float(
            match light {
                0 => GeCmd::Light0QuadtraticAtten,
                1 => GeCmd::Light1QuadtraticAtten,
                2 => GeCmd::Light2QuadtraticAtten,
                3 => GeCmd::Light3QuadtraticAtten,
                _ => unreachable!(),
            },
            quadratic,
        );
        self.list.send_float(
            match light {
                0 => GeCmd::Light0ExponentAtten,
                1 => GeCmd::Light1ExponentAtten,
                2 => GeCmd::Light2ExponentAtten,
                3 => GeCmd::Light3ExponentAtten,
                _ => unreachable!(),
            },
            exponent,
        );
    }
    /// This seems to only apply to spot-lights
    pub fn light_cutoff(&mut self, light: u8, cutoff: f32) {
        self.list.send_float(
            match light {
                0 => GeCmd::Light0CutoffAtten,
                1 => GeCmd::Light1CutoffAtten,
                2 => GeCmd::Light2CutoffAtten,
                3 => GeCmd::Light3CutoffAtten,
                _ => panic!("Only 4 Lights are supported"),
            },
            cutoff,
        );
    }
    pub fn light_mode(&mut self, mode: sys::LightMode) {
        self.list.send(GeCmd::LightMode, mode as usize);
    }
    // TODO: SceGuLightSpot
    /// You can think of this as a utility function rather than an actual GE command
    pub fn clear(
        &mut self,
        flags: sys::ClearFlags,
        depth: u16,
    ) -> GlResult<()> {
        use sys::VertexType as VertType;
        static mut VERTICES: [[i16; 3]; 2] = [[0; 3]; 2];
        const VERTEX_TYPE: VertType =
            VertType::VERTEX_16BIT.union(VertType::TRANSFORM_2D);
        const VERTEX_SIZE: VertexSize = const_vt_size(VERTEX_TYPE);

        let (width_s, height_s) = self.display_size.unwrap();
        let (width_s, height_s) = (width_s as i16, height_s as i16);
        let depth_s = depth as i16;
        unsafe {
            VERTICES[1] = [0, 0, depth_s];
            VERTICES[0] = [width_s, height_s, depth_s];
        }
        self.clear_mode(flags);
        let v = self.bind_vertices(VERTEX_TYPE, VERTEX_SIZE, unsafe {
            #[allow(static_mut_refs)]
            &VERTICES
        })?;
        self.draw_primitives(v, sys::GuPrimitive::Sprites);
        self.disable_clear();
        Ok(())
    }
    pub fn clear_mode(&mut self, flags: sys::ClearFlags) {
        let relevant_flags = flags
            & (sys::ClearFlags::COLOR_BUFFER_BIT
                .union(sys::ClearFlags::STENCIL_BUFFER_BIT)
                .union(sys::ClearFlags::DEPTH_BUFFER_BIT));
        self.list.send(
            GeCmd::ClearMode,
            (relevant_flags.bits() << 8) as usize | 0x01,
        );
    }
    pub fn disable_clear(&mut self) {
        self.list.send(GeCmd::ClearMode, 0);
    }
    pub fn mask_pixel(&mut self, mask: Color32) {
        self.list.send(GeCmd::MaskRgb, mask.as_bgr() as usize);
        self.list.send(GeCmd::MaskAlpha, mask.a() as usize);
    }
    pub fn clear_color(&mut self, color: Color32) {
        self.material_update(sys::LightComponent::AMBIENT);
        self.material_ambient(color);
        self.ambient(color);
        //self.material_diffuse(color);
        //self.material_specular(color);
    }
    pub fn color_function(
        &mut self,
        func: sys::ColorFunc,
        color: Color32,
        mask: usize,
    ) {
        self.list.send(GeCmd::ColorTest, func as usize & 0x03);
        self.list.send(GeCmd::ColorRef, color.as_bgr() as usize);
        self.list.send(GeCmd::ColorTestmask, mask);
    }
    pub fn color_material(&mut self, components: sys::LightComponent) {
        self.list.send(
            GeCmd::MaterialUpdate,
            components.bits().cast_unsigned() as usize,
        );
    }
    pub fn alpha_function(
        &mut self,
        func: sys::AlphaFunc,
        value: usize,
        mask: usize,
    ) {
        let arg = func as usize | ((value & 0xff) << 8) | ((mask & 0xff) << 16);
        self.list.send(GeCmd::AlphaTest, arg);
    }
    pub fn ambient(&mut self, color: Color32) {
        self.list.send(GeCmd::AmbientColor, color.as_bgr() as usize);
        self.list.send(GeCmd::AmbientAlpha, color.a() as usize);
    }
    pub fn material_emissive(&mut self, color: Color32) {
        self.list
            .send(GeCmd::MaterialEmissive, color.as_bgr() as usize);
    }
    pub fn material_ambient(&mut self, color: Color32) {
        self.list
            .send(GeCmd::MaterialAmbient, color.as_bgr() as usize);
        self.list.send(GeCmd::MaterialAlpha, color.a() as usize);
    }
    pub fn material_diffuse(&mut self, color: Color32) {
        self.list
            .send(GeCmd::MaterialDiffuse, color.as_bgr() as usize);
    }
    pub fn material_specular(&mut self, color: Color32) {
        self.list
            .send(GeCmd::MaterialSpecular, color.as_bgr() as usize);
    }
    pub fn material_update(&mut self, update: sys::LightComponent) {
        // TODO: use bitflags
        self.list
            .send(GeCmd::MaterialUpdate, update.bits() as usize & 0b111);
    }
    pub fn stencil_function(
        &mut self,
        func: sys::StencilFunc,
        ref_: u8,
        mask: u8,
    ) {
        self.list.send(
            GeCmd::StencilTest,
            func as usize | ((ref_ as usize) << 8) | ((mask as usize) << 16),
        );
    }
    pub fn stencil_operation(
        &mut self,
        fail: sys::StencilOperation,
        zfail: sys::StencilOperation,
        zpass: sys::StencilOperation,
    ) {
        self.list.send(
            GeCmd::StencilOp,
            fail as usize | ((zfail as usize) << 8) | ((zpass as usize) << 16),
        );
    }
    /// # Panics
    /// Panics if the `src` or `dst` constants are bigger than 24 bits
    pub fn blend_function(
        &mut self,
        op: sys::BlendOp,
        src: sys::BlendFactor,
        dst: sys::BlendFactor,
        src_fix: usize,
        dst_fix: usize,
    ) {
        assert!(src_fix >> 24 == 0);
        assert!(dst_fix >> 24 == 0);
        self.list.send(
            GeCmd::BlendMode,
            ((op as usize) << 8) | ((dst as usize) << 4) | src as usize,
        );
        self.list.send(GeCmd::BlendFixedA, src_fix);
        self.list.send(GeCmd::BlendFixedB, dst_fix);
    }
    pub fn logical_operation(&mut self, op: sys::LogicalOperation) {
        self.list.send(GeCmd::LogicOp, op as usize);
    }
    pub fn specular_coeff(&mut self, power: f32) {
        self.list.send_float(GeCmd::MaterialSpecularCoef, power);
    }
    pub fn viewport(
        &mut self,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
    ) {
        self.list.send_float(GeCmd::ViewportXCenter, center_x);
        self.list.send_float(GeCmd::ViewportYCenter, center_y);
        self.list.send_float(GeCmd::ViewportXScale, width / 2.0);
        self.list.send_float(GeCmd::ViewportYScale, -height / 2.0);
    }
    pub fn offset(&mut self, x: usize, y: usize) {
        self.list.send(GeCmd::OffsetX, x << 4);
        self.list.send(GeCmd::OffsetY, y << 4);
    }
    const fn matrix_to_commands(matrix_mode: MatrixMode) -> (GeCmd, GeCmd) {
        match matrix_mode {
            MatrixMode::View => {
                (GeCmd::ViewMatrixNumber, GeCmd::ViewMatrixData)
            }
            MatrixMode::Model => {
                (GeCmd::WorldMatrixNumber, GeCmd::WorldMatrixData)
            }
            MatrixMode::Texture => {
                (GeCmd::TGenMatrixNumber, GeCmd::TGenMatrixData)
            }
        }
    }
    pub fn set_matrix(&mut self, matrix_mode: MatrixMode, matrix: &Mat3By4) {
        let (number_cmd, data_cmd) = Self::matrix_to_commands(matrix_mode);
        let mut last_index = 0;
        for (index, scalar) in matrix.as_ref().iter().enumerate() {
            let scalar = *scalar;
            if self.matrix_command_cache.test(matrix_mode, index, scalar) {
                if index != last_index + 1 {
                    self.list.send(number_cmd, index);
                }
                self.list.send_float(data_cmd, scalar);
                last_index = index;
            }
        }
    }
    pub fn overwrite_matrix(
        &mut self,
        matrix_mode: MatrixMode,
        matrix: Mat3By4,
    ) {
        let (number_cmd, data_cmd) = Self::matrix_to_commands(matrix_mode);
        self.list.send(number_cmd, 0);
        for scalar in matrix.as_ref() {
            self.list.send_float(data_cmd, *scalar);
        }
        self.matrix_command_cache.overwrite(matrix_mode, matrix);
    }
    pub fn set_projection_matrix(&mut self, matrix: &Mat4) {
        let mut last_index = 0;
        for (index, scalar) in matrix.as_ref().iter().enumerate() {
            let scalar = *scalar;
            if self.matrix_command_cache.test_projection(index, scalar) {
                if index != last_index + 1 {
                    self.list.send(GeCmd::ProjMatrixNumber, index);
                }
                self.list.send_float(GeCmd::ProjMatrixData, scalar);
                last_index = index;
            }
        }
    }
    pub fn overwrite_projection_matrix(&mut self, matrix: Mat4) {
        self.list.send_float(GeCmd::ProjMatrixNumber, 0.0);
        for scalar in matrix.as_ref() {
            self.list.send_float(GeCmd::ProjMatrixData, *scalar);
        }
        self.matrix_command_cache.overwrite_projection(matrix);
    }
    pub fn set_bone_matrix(&mut self, index: u8, matrix: &Mat3By4) {
        self.list
            .send(GeCmd::BoneMatrixNumber, (index as usize) * 12);
        // 3 * 4 matrix
        for scalar in matrix.as_ref() {
            self.list.send_float(GeCmd::BoneMatrixData, *scalar);
        }
    }
    pub fn texture_env_color(&mut self, color: Color32) {
        self.list.send(GeCmd::TexEnvColor, color.as_bgr() as usize);
    }
    pub fn texture_flush(&mut self) {
        self.list.send_float(GeCmd::TexFlush, 0.0);
    }
    pub fn texture_filter(
        &mut self,
        min: sys::TextureFilter,
        mag: sys::TextureFilter,
    ) {
        self.list
            .send(GeCmd::TexFilter, ((mag as usize) << 8) | (min as usize));
    }
    pub fn texture_function(
        &mut self,
        tfx: sys::TextureEffect,
        tcc: sys::TextureColorComponent,
        fragment_2x: bool,
    ) {
        self.list.send(
            GeCmd::TexFunc,
            ((tcc as usize) << 8) | (tfx as usize) | usize::from(fragment_2x),
        );
    }
    /// # Safety
    /// This functions dereferences a raw pointer (`buffer`)
    /// # Panics
    /// panics if `mipmap` is higher than 7 (only 8 mipmaps are supported)
    pub unsafe fn texture_image(
        &mut self,
        mipmap: sys::MipmapLevel,
        buffer: *const c_void,
        size: (u16, u16),
        stride: u16,
    ) {
        use sys::MipmapLevel;
        assert!((mipmap as usize) < 8);
        let (tbp1, tbp2) = split_address(buffer.addr());
        self.list.send(
            match mipmap {
                MipmapLevel::None => GeCmd::TexAddr0,
                MipmapLevel::Level1 => GeCmd::TexAddr1,
                MipmapLevel::Level2 => GeCmd::TexAddr2,
                MipmapLevel::Level3 => GeCmd::TexAddr3,
                MipmapLevel::Level4 => GeCmd::TexAddr4,
                MipmapLevel::Level5 => GeCmd::TexAddr5,
                MipmapLevel::Level6 => GeCmd::TexAddr6,
                MipmapLevel::Level7 => GeCmd::TexAddr7,
            },
            tbp1,
        );
        self.list.send_parts(
            match mipmap {
                MipmapLevel::None => GeCmd::TexBufWidth0,
                MipmapLevel::Level1 => GeCmd::TexBufWidth1,
                MipmapLevel::Level2 => GeCmd::TexBufWidth2,
                MipmapLevel::Level3 => GeCmd::TexBufWidth3,
                MipmapLevel::Level4 => GeCmd::TexBufWidth4,
                MipmapLevel::Level5 => GeCmd::TexBufWidth5,
                MipmapLevel::Level6 => GeCmd::TexBufWidth6,
                MipmapLevel::Level7 => GeCmd::TexBufWidth7,
            },
            tbp2 & 0x0f,
            stride,
        );
        self.list.send(
            match mipmap {
                MipmapLevel::None => GeCmd::TexSize0,
                MipmapLevel::Level1 => GeCmd::TexSize1,
                MipmapLevel::Level2 => GeCmd::TexSize2,
                MipmapLevel::Level3 => GeCmd::TexSize3,
                MipmapLevel::Level4 => GeCmd::TexSize4,
                MipmapLevel::Level5 => GeCmd::TexSize5,
                MipmapLevel::Level6 => GeCmd::TexSize6,
                MipmapLevel::Level7 => GeCmd::TexSize7,
            },
            (size.1.ilog2() << 8 | size.0.ilog2()) as usize,
        );
        self.texture_flush();
    }
    pub fn texture_level_mode(
        &mut self,
        mode: sys::TextureLevelMode,
        bias: i8,
    ) {
        // PSPSDK: mipmap bias?
        let offset = (bias as isize * 16).clamp(-128, 128);
        self.list.send(
            GeCmd::TexLevel,
            (offset.cast_unsigned() << 16) | mode as usize,
        );
    }
    pub fn texture_map_mode(
        &mut self,
        mode: sys::TextureMapMode,
        proj_mode: sys::TextureProjectionMapMode,
    ) {
        self.list.send(
            GeCmd::TexMapMode,
            ((proj_mode as usize) << 8) | mode as usize,
        );
    }
    pub fn texture_env_map_matrix(&mut self, u: usize, v: usize) {
        self.list.send(GeCmd::TexShadeLs, (v << 8) | (u & 0x03));
    }
    pub fn texture_mode(
        &mut self,
        max_mips: sys::MipmapLevel,
        multi_clut: bool,
        swizzle: bool,
    ) {
        self.list.send(
            GeCmd::TexMode,
            ((max_mips as usize) << 16)
                | (usize::from(multi_clut) << 8)
                | usize::from(swizzle),
        );
    }
    pub fn texture_format(&mut self, format: sys::TexturePixelFormat) {
        self.list.send(GeCmd::TexFormat, format as usize);
    }
    /// Set texture offset
    ///
    /// # Note
    ///
    /// Only used by the 3D T&L pipe, renders done with `VertexType::TRANSFORM_2D`
    /// are not affected by this.
    pub fn texture_offset(&mut self, u: f32, v: f32) {
        self.list.send_float(GeCmd::TexOffsetU, u);
        self.list.send_float(GeCmd::TexOffsetV, v);
    }
    /// Set texture scale
    ///
    /// # Note
    ///
    /// Only used by the 3D T&L pipe, renders done with `VertexType::TRANSFORM_2D`
    /// are not affected by this.
    pub fn texture_scale(&mut self, u: f32, v: f32) {
        self.list.send_float(GeCmd::TexScaleU, u);
        self.list.send_float(GeCmd::TexScaleV, v);
    }
    pub fn texture_lod_slope(&mut self, slope: f32) {
        self.list.send_float(GeCmd::TexLodSlope, slope);
    }
    pub fn texture_wrap(
        &mut self,
        u: sys::GuTexWrapMode,
        v: sys::GuTexWrapMode,
    ) {
        self.list.send(
            GeCmd::TexWrap,
            ((v as usize & 0xff) << 8) | (u as usize & 0xff),
        );
    }
    pub fn texture(&mut self, mipmap: sys::MipmapLevel, texture: &Texture) {
        self.texture_format(texture.format());
        unsafe {
            self.texture_mode(
                mipmap,
                false, // TODO: enable multi-clut?
                texture.swizzled(),
            );
            self.texture_image(
                mipmap,
                texture.buffer().as_ptr().cast(),
                texture.size(),
                texture.width(),
            );
        }
    }

    // TODO
    pub fn set_dither(&mut self, matrix: &I8Mat4) {
        self.list.send(
            GeCmd::Dith0,
            ((matrix.x.w as usize) << 12)
                | ((matrix.x.z as usize) << 8)
                | ((matrix.x.y as usize) << 4)
                | (matrix.x.x as usize),
        );
        self.list.send(
            GeCmd::Dith1,
            ((matrix.y.w as usize) << 12)
                | ((matrix.y.z as usize) << 8)
                | ((matrix.y.y as usize) << 4)
                | (matrix.y.x as usize),
        );
        self.list.send(
            GeCmd::Dith2,
            ((matrix.z.w as usize) << 12)
                | ((matrix.z.z as usize) << 8)
                | ((matrix.z.y as usize) << 4)
                | (matrix.z.x as usize),
        );
        self.list.send(
            GeCmd::Dith3,
            ((matrix.w.w as usize) << 12)
                | ((matrix.w.z as usize) << 8)
                | ((matrix.w.y as usize) << 4)
                | (matrix.w.x as usize),
        );
    }
    pub fn patch_division(&mut self, u_divisions: u8, v_divisions: u8) {
        self.list.send(
            GeCmd::PatchDivision,
            ((v_divisions as usize) << 8) | u_divisions as usize,
        );
    }
    pub fn patch_front_face(&mut self, unknown: usize) {
        self.list.send(GeCmd::PatchFacing, unknown);
    }
    pub fn patch_primitive(&mut self, primitive: sys::PatchPrimitive) {
        use sys::PatchPrimitive;
        self.list.send(
            GeCmd::PatchPrimitive,
            match primitive {
                PatchPrimitive::Points => 2,
                PatchPrimitive::LineStrip => 1,
                PatchPrimitive::TriangleStrip => 0,
            },
        );
    }
    pub fn shading_model(&mut self, mode: sys::ShadingModel) {
        use sys::ShadingModel;
        self.list.send(
            GeCmd::ShadeMode,
            match mode {
                ShadingModel::Smooth => 1,
                ShadingModel::Flat => 0,
            },
        );
    }
    /// # Panics
    /// panics if `index >= 8` (only 8 bones are supported)
    pub fn morph_weight(&mut self, index: u8, weight: f32) {
        self.list.send_float(
            match index {
                0 => GeCmd::MorphWeight0,
                1 => GeCmd::MorphWeight1,
                2 => GeCmd::MorphWeight2,
                3 => GeCmd::MorphWeight3,
                4 => GeCmd::MorphWeight4,
                5 => GeCmd::MorphWeight5,
                6 => GeCmd::MorphWeight6,
                7 => GeCmd::MorphWeight7,
                _ => panic!("Invalid index"),
            },
            weight,
        );
    }

    pub fn transfer(
        &mut self,
        _dst: &mut Texture,
        _src: &Texture,
        // etc...
    ) {
        todo!();
        /*        let (src1, src2) = split_address(src.buffer().as_ptr().addr());
        self.list.send(GeCmd::TransferSrc, src1);
        self.list.send(
            GeCmd::TransferSrcW,
            ((src2 as usize) << 16) | src.stride() as usize,
        );
        self.list.send(GeCmd::TransferSrcPos, 0); // It's already off-set

        let (dst1, dst2) = split_address(dst.buffer().as_ptr().addr());
        self.list.send(GeCmd::TransferDst, dst1);
        self.list.send(
            GeCmd::TransferDstW,
            ((dst2 as usize) << 16) | dst.stride() as usize,
        );
        self.list.send(GeCmd::TransferDstPos, 0); // It's already off-set

        self.list.send(
            GeCmd::TransferSize,
            (dst.height() as usize - 1) << 10 | (dst.width() as usize - 1),
        );
        self.list
            .send(GeCmd::TransferStart, usize::from(size_of::<T>() == 4));*/
    }

    pub fn transfer_sync(&mut self) {
        self.list.send(GeCmd::TexSync, 0);
    }
    pub fn load_clut(
        &mut self,
        clut: &Texture,
        shift: u8,
        mask: u8,
        unknown: u8,
    ) -> GlResult<()> {
        use sys::TexturePixelFormat as TexelFmt;
        match clut.format() {
            TexelFmt::PsmT4
            | TexelFmt::PsmT8
            | TexelFmt::PsmT16
            | TexelFmt::PsmT32 => {
                return Err(GlError::InvalidClutFormat);
            }
            _ => {}
        }

        let arg = (clut.format() as usize)
            | ((shift as usize) << 2)
            | ((mask as usize) << 8)
            | ((unknown as usize) << 16);
        self.list.send(GeCmd::ClutFormat, arg);
        let (ptr1, ptr2) = split_address(clut.buffer().as_ptr().addr());
        self.list.send(GeCmd::ClutAddr, ptr1);
        self.list.send(GeCmd::ClutAddrUpper, ptr2 as usize);
        self.list
            .send(GeCmd::LoadClut, (clut.width() * clut.height() / 8) as usize);
        Ok(())
    }
}
