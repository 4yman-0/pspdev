use psp_sys::sys;

pub const BUF_WIDTH: usize = 512;
pub const SCREEN_HEIGHT: usize = 272;
pub const SCREEN_WIDTH: usize = 480;

use glam::{I8Vec4, Mat4};

pub mod color;
pub mod gl;
pub mod index;
pub mod texture;
pub mod vertex;
pub mod vram_alloc;

use gl::{Gl, I8Mat4, Mat3By4};
use texture::Texture;
use vram_alloc::VramAllocator;

pub struct Gfx {
    gl: Gl,
    vram_allocator: VramAllocator,
    frame_buffer: Texture,
    depth_buffer: Option<Texture>,
    double_buffer: Option<(Texture, bool)>,
}

impl Gfx {
    pub const WIDTH: u16 = 480;
    pub const HEIGHT: u16 = 272;
    pub const BUFFER_WIDTH: u16 = 512;
    pub fn init_default() -> gl::GlResult<Self> {
        Self::init()?
            .depth_test()
            .double_buffering()
            .culling()
            .scissor_test()?
            .clip_planes()
            .texture_2d()
            .build()
    }

    pub fn scissor_test(mut self) -> gl::GlResult<Self> {
        let size = self.size();
        let gl = self.gl_mut();
        gl.scissor_region(0, 0, size.0, size.1)?;
        Ok(self)
    }
    #[must_use]
    pub fn depth_test(mut self) -> Self {
        let depth_buffer = Texture::allocate(
            self.vram_allocator_mut(),
            Self::BUFFER_WIDTH,
            Self::WIDTH,
            sys::TexturePixelFormat::Psm4444,
            false,
        )
        .unwrap();
        self.depth_buffer = Some(depth_buffer);
        self.gl.depth_range(u16::MAX, 0, 0);
        self.gl.depth_test_function(sys::DepthFunc::GreaterOrEqual);
        self.gl.set_state(sys::GuState::DepthTest, true);
        unsafe {
            self.gl
                .set_depth_buffer(self.depth_buffer.as_mut().unwrap())
                .unwrap();
        };
        self
    }
    #[must_use]
    pub fn double_buffering(mut self) -> Self {
        let double_buffer = Texture::allocate(
            self.vram_allocator_mut(),
            Self::BUFFER_WIDTH,
            Self::WIDTH,
            sys::TexturePixelFormat::Psm5551,
            false,
        )
        .unwrap();
        self.double_buffer = Some((double_buffer, false));
        self
    }
    #[must_use]
    pub fn culling(mut self) -> Self {
        let gl = self.gl_mut();
        gl.front_face_direction(sys::FrontFaceDirection::CounterClockwise);
        gl.set_state(sys::GuState::CullFace, true);
        self
    }
    #[must_use]
    pub fn clip_planes(mut self) -> Self {
        self.gl_mut().set_state(sys::GuState::ClipPlanes, true);
        self
    }
    #[must_use]
    pub fn texture_2d(mut self) -> Self {
        let gl = self.gl_mut();
        gl.texture_scale(1.0, 1.0);
        gl.texture_offset(0.0, 0.0);
        // The texture might not be valid
        //gl.set_state(sys::GuState::Texture2D, true);
        self
    }
    pub fn build(mut self) -> gl::GlResult<Self> {
        let gl = self.gl_mut();
        gl.finish()?;
        gl.list_mut()
            .sync(sys::GuSyncMode::List, sys::GuSyncBehavior::Wait);
        //gl.list_mut().dequeue();
        psp_sys::dprint!("SYNCED");
        let _ = gl;

        Ok(self)
    }

    /// It is strongly recommended to only call this once
    pub fn init() -> gl::GlResult<Self> {
        use gl::MatrixMode;
        let mut vram_allocator = VramAllocator::default();
        let mut frame_buffer = Texture::allocate(
            &mut vram_allocator,
            Self::BUFFER_WIDTH,
            Self::HEIGHT,
            sys::TexturePixelFormat::Psm5551,
            false,
        )
        .unwrap();
        unsafe {
            sys::sceGuInit();
        };
        let mut gl = Gl::new();
        gl.list_mut().start();
        unsafe {
            gl.set_frame_buffer(&mut frame_buffer)?;
            //self.gl.remove_display_buffer();
            gl.set_display_buffer(&frame_buffer)?;
            gl.set_display_size(Self::WIDTH, Self::HEIGHT);
        };
        gl.draw_region(0, 0, Self::WIDTH, Self::HEIGHT)?;
        gl.offset(
            2048 - (Self::WIDTH as usize / 2),
            2048 - (Self::HEIGHT as usize / 2),
        );
        gl.viewport(2048.0, 2048.0, Self::WIDTH.into(), Self::HEIGHT.into());
        // Chewing sceGum is not allowed
        gl.overwrite_projection_matrix(Mat4::IDENTITY);
        for mode in [MatrixMode::View, MatrixMode::Model, MatrixMode::Texture] {
            gl.overwrite_matrix(mode, Mat3By4::IDENTITY);
        }
        for bone_index in 0..8 {
            gl.set_bone_matrix(bone_index, &Mat3By4::IDENTITY);
            gl.morph_weight(bone_index, 0.0);
        }

        gl.set_dither(&I8Mat4 {
            x: I8Vec4::new(-4, 0, -3, 1),
            y: I8Vec4::new(2, -2, 3, -1),
            z: I8Vec4::new(-3, 1, -4, 0),
            w: I8Vec4::new(3, -1, 2, -2),
        });
        gl.shading_model(sys::ShadingModel::Smooth);
        gl.patch_division(16, 16);

        psp_sys::dprint!("initialized!");
        Ok(Self {
            gl,
            vram_allocator,
            frame_buffer,
            depth_buffer: None,
            double_buffer: None,
        })
    }

    fn close_non_consuming(&mut self) {
        let gl = self.gl_mut();
        gl.remove_display_buffer();
    }

    #[must_use]
    pub const fn size(&self) -> (u16, u16) {
        (SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16)
    }

    #[must_use]
    pub const fn frame_buffer(&self) -> &Texture {
        &self.frame_buffer
    }

    #[must_use]
    pub const fn frame_buffer_mut(&mut self) -> &mut Texture {
        &mut self.frame_buffer
    }

    #[must_use]
    pub const fn depth_buffer(&self) -> Option<&Texture> {
        self.depth_buffer.as_ref()
    }
    #[must_use]
    pub const fn depth_buffer_mut(&mut self) -> Option<&mut Texture> {
        self.depth_buffer.as_mut()
    }

    #[must_use]
    pub const fn double_buffer(&self) -> Option<&(Texture, bool)> {
        self.double_buffer.as_ref()
    }
    #[must_use]
    pub fn double_buffer_mut(&mut self) -> Option<&mut (Texture, bool)> {
        self.double_buffer.as_mut()
    }

    /// This is not recommended, use [`start_frame`] instead
    #[must_use]
    pub const fn gl(&self) -> &Gl {
        &self.gl
    }
    /// This is not recommended, use [`start_frame`] instead
    #[must_use]
    pub fn gl_mut(&mut self) -> &mut Gl {
        &mut self.gl
    }

    #[must_use]
    pub const fn vram_allocator(&self) -> &VramAllocator {
        &self.vram_allocator
    }
    #[must_use]
    pub fn vram_allocator_mut(&mut self) -> &mut VramAllocator {
        &mut self.vram_allocator
    }

    pub fn start_frame(&mut self) -> Frame<'_> {
        self.gl_mut().list_mut().start();
        if self.double_buffer.is_some() {
            let (double_buffer, is_drawn) =
                self.double_buffer.as_mut().unwrap();
            if *is_drawn {
                unsafe {
                    self.gl.set_frame_buffer(double_buffer).unwrap();
                };
                *is_drawn = false;
            } else {
                unsafe {
                    self.gl.set_frame_buffer(&mut self.frame_buffer).unwrap();
                };
                *is_drawn = true;
            }
        } else {
            unsafe {
                self.gl.set_frame_buffer(&mut self.frame_buffer).unwrap();
            };
        }
        Frame { gfx: self }
    }
    pub fn start_frame_with<F>(&mut self, function: F) -> gl::GlResult<()>
    where
        F: FnOnce(&mut Frame<'_>) -> gl::GlResult<()>,
    {
        {
            let mut frame = self.start_frame();
            function(&mut frame)?;
        }
        Ok(())
    }
    /*pub fn start_owning_frame(mut self) -> OwningFrame {
        self.gl_mut().list_mut().start();
        unsafe {
            self.gl.set_frame_buffer(&mut self.frame_buffer).unwrap();
        };
        OwningFrame { gfx: self }
    }*/
}

impl Drop for Gfx {
    fn drop(&mut self) {
        self.close_non_consuming();
    }
}

pub struct Frame<'frame> {
    gfx: &'frame mut Gfx,
}

impl<'frame> Frame<'frame> {
    /// Finish rendering and wait for v-blank.
    pub fn finish_non_consuming(&mut self) -> gl::GlResult<()> {
        let gl = self.gl_mut();
        gl.finish()?;
        gl.list_mut()
            .sync(sys::GuSyncMode::Finish, sys::GuSyncBehavior::Wait);
        if self.gfx.double_buffer.is_some() {
            let (double_buffer, is_drawn) =
                self.gfx.double_buffer.as_ref().unwrap();
            if !*is_drawn {
                self.gfx.gl.set_display_buffer(double_buffer)?;
            } else {
                self.gfx.gl.set_display_buffer(&self.gfx.frame_buffer)?;
            }
        }
        Ok(())
    }

    /// Finish rendering and wait for v-blank.
    ///
    /// ## Note
    /// You don't have to call this as the [`Frame`] is terminated automatically when it's dropped.
    pub fn finish(mut self) {
        self.finish_non_consuming().unwrap();
    }

    #[must_use]
    pub const fn size(&self) -> (u16, u16) {
        self.gfx.size()
    }

    #[must_use]
    pub const fn gfx(&self) -> &Gfx {
        self.gfx
    }

    #[must_use]
    pub const fn gfx_mut(&mut self) -> &mut Gfx {
        self.gfx
    }

    #[must_use]
    pub const fn gl(&self) -> &Gl {
        &self.gfx.gl
    }

    #[must_use]
    pub fn gl_mut(&mut self) -> &mut Gl {
        &mut self.gfx.gl
    }
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        self.finish_non_consuming().unwrap();
    }
}

/*
/// A [`Frame`] which contains [`Gfx`]
/// This may be useful in cases where storing a reference is not possible
pub struct OwningFrame {
    pub gfx: Gfx,
}

impl OwningFrame {
    /// Finish rendering and wait for v-blank.
    pub fn finish_non_consuming(&mut self) -> gl::GlResult<()> {
        let gl = self.gl_mut();
        gl.finish()?;
        gl.list_mut()
            .sync(sys::GuSyncMode::Finish, sys::GuSyncBehavior::Wait);
        Ok(())
    }

    /// Finish rendering and wait for v-blank.
    pub fn finish(mut self) -> Gfx {
        self.finish_non_consuming().unwrap();
        self.gfx
    }

    #[must_use]
    pub const fn gfx(&self) -> &Gfx {
        &self.gfx
    }

    #[must_use]
    pub const fn gfx_mut(&mut self) -> &mut Gfx {
        &mut self.gfx
    }

    #[must_use]
    pub const fn gl(&self) -> &Gl {
        &self.gfx.gl
    }

    #[must_use]
    pub fn gl_mut(&mut self) -> &mut Gl {
        &mut self.gfx.gl
    }
}
*/
