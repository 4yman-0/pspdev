use super::vram_alloc::VramChunk;
use psp_sys::sys::TexturePixelFormat;

pub fn texture_pixel_size(psm: TexturePixelFormat) -> usize {
    match psm {
        //TexturePixelFormat::PsmT4 => length >> 1,
        TexturePixelFormat::PsmT8 => 1,

        TexturePixelFormat::Psm5650
        | TexturePixelFormat::Psm5551
        | TexturePixelFormat::Psm4444
        | TexturePixelFormat::PsmT16 => 2,

        TexturePixelFormat::Psm8888 | TexturePixelFormat::PsmT32 => 4,

        // TODO: support other texture types (Dxt1/3/5)
        _ => unimplemented!(),
    }
}

// // TODO: figure out why RAM texture don't work
pub struct Texture {
    size: (u16, u16),
    format: TexturePixelFormat,
    swizzled: bool,
    chunk: VramChunk,
}

impl Texture {
    /// Allocates a new `Texture`.
    // TODO: documentation is unclear.
    pub fn allocate(
        width: u16,
        height: u16,
        format: TexturePixelFormat,
        swizzled: bool,
    ) -> Self {
        Self {
            chunk: {
                let pixel_size = texture_pixel_size(format);
                let size = width as usize * height as usize * pixel_size;
                VramChunk::alloc(size, pixel_size)
            },
            size: (width, height),
            format,
            swizzled,
        }
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }
    pub fn width(&self) -> u16 {
        self.size.0
    }
    pub fn height(&self) -> u16 {
        self.size.1
    }
    pub fn format(&self) -> TexturePixelFormat {
        self.format
    }
    pub fn swizzled(&self) -> bool {
        self.swizzled
    }
    pub fn set_swizzled(&mut self, swizzled: bool) {
        self.swizzled = swizzled;
    }
    pub fn buffer(&self) -> &[u8] {
        self.chunk.as_ref()
    }
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        self.chunk.as_mut()
    }
    pub fn can_be_framebuffer(&self) -> bool {
        use TexturePixelFormat as TexelFmt;
        if self.swizzled {
            return false;
        }
        matches!(
            self.format,
            TexelFmt::Psm8888
                | TexelFmt::Psm4444
                | TexelFmt::Psm5650
                | TexelFmt::Psm5551
        )
    }

    pub fn swizzle_from_slice(&mut self, src: &[u8]) -> Option<()> {
        let (width, height) = (self.width() as usize, self.height() as usize);
        if src.len() != width * height * texture_pixel_size(self.format) {
            return None;
        }
        if !width.is_multiple_of(16) || !height.is_multiple_of(8) {
            return None;
        }
        todo!();
        //Some(())
    }
}
